use crate::{
    builtins::{disp::Dispatcher, Disp},
    error::*,
    JsEngine, JsonValue, MsgChannel,
};
use std::fmt;

use js::{Context, Function, Promise, Tokio};
use snafu::ResultExt;
use tracing::debug;
impl JsEngine {
    pub fn create() -> Result<(Self, flume::Receiver<MsgChannel>)> {
        let (tx, rx) = flume::unbounded::<MsgChannel>();
        let rt = js::Runtime::new().context(JsRuntimeSnafu)?;
        rt.set_max_stack_size(256 * 1024);
        rt.set_memory_limit(2 * 1024 * 1024);

        let ctx = Context::full(&rt).context(JsContextSnafu)?;
        rt.spawn_executor(Tokio);

        let engine = Self {
            runtime: rt,
            context: ctx,
            sender: tx,
        };
        engine.init_globals()?;
        Ok((engine, rx))
    }

    #[cfg(feature = "builtin_processor")]
    pub fn create_with_processors(
        processors: Vec<(&str, &str, Box<dyn crate::Processor>)>,
    ) -> Result<Self, Error> {
        let (engine, rx) = Self::create()?;

        run_processors(rx, processors);
        Ok(engine)
    }

    pub async fn run(&self, code: &str) -> Result<JsonValue, Error> {
        let ret: Result<Promise<JsonValue>, js::Error> = self.context.with(|ctx| {
            let src = format!(r#"export default async function() {{ {} }}"#, code);
            debug!("code to execute: {}", src);
            let m = ctx.compile("script", src)?;
            let fun = m.get::<_, Function>("default")?;

            fun.call(())
        });
        ret.context(JsExecuteSnafu)?.await.context(JsExecuteSnafu)
    }

    fn init_globals(&self) -> Result<(), Error> {
        let ret: Result<(), js::Error> = self.context.with(|ctx| {
            let glob = ctx.globals();
            #[cfg(feature = "console")]
            {
                use crate::builtins::{con::Console, Con};
                glob.init_def::<Con>()?;
                glob.set("console", Console)?;
            }

            glob.init_def::<Disp>()?;
            glob.set("dispatcher", Dispatcher::new(self.sender.clone()))?;
            Ok(())
        });
        ret.context(JsExecuteSnafu)
    }
}

impl fmt::Debug for JsEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsEngine").finish()
    }
}

#[cfg(feature = "builtin_processor")]
fn run_processors(
    rx: flume::Receiver<MsgChannel>,
    processors: Vec<(&str, &str, Box<dyn crate::Processor>)>,
) {
    let processors: std::collections::HashMap<(String, String), Box<dyn crate::Processor>> =
        processors
            .into_iter()
            .map(|(ns, name, processor)| ((ns.to_owned(), name.to_owned()), processor))
            .collect();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv_async().await {
            let name = format!("{}.{}", msg.namespace, msg.name);
            tracing::info!("Got request for {name}: {:#?}", msg.args);
            let processor = match processors.get(&(msg.namespace, msg.name)) {
                Some(p) => p,
                None => {
                    if let Err(e) = msg.res.send(Err(format!("{} not found", name))) {
                        tracing::warn!("send error: {:?}", e);
                    };
                    continue;
                }
            };
            let ret = processor.call(msg.args).await;
            if let Err(e) = msg.res.send(ret) {
                tracing::warn!("send error: {:?}", e);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use anyhow::Result;
    #[allow(unused_imports)]
    use serde_json::json;

    #[cfg(feature = "builtin_processor")]
    fn auth_create_token(args: JsonValue) -> std::result::Result<JsonValue, String> {
        Ok(args)
    }

    #[cfg(feature = "builtin_processor")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn js_engine_should_work() -> Result<()> {
        tracing_subscriber::fmt::init();

        let engine = JsEngine::create_with_processors(vec![(
            "auth",
            "create_token",
            Box::new(auth_create_token) as Box<dyn crate::Processor>,
        )])?;
        #[cfg(feature = "console")]
        engine
            .run("let a = 1;console.log(`hello world ${a}`, a)")
            .await?;
        let ret = engine
            .run("return dispatcher.dispatch('auth', 'create_token', {a: 1})")
            .await?;

        assert_eq!(ret.0, json!({"a": 1}));
        Ok(())
    }
}
