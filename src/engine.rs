use crate::{error::*, JsEngine, JsonValue};
use std::fmt;

use js::{Context, Function, Object, Promise, Tokio};
use snafu::ResultExt;
use tracing::debug;
impl JsEngine {
    #[cfg(feature = "dispatcher")]
    pub fn create() -> Result<(Self, flume::Receiver<crate::MsgChannel>)> {
        let (tx, rx) = flume::unbounded::<crate::MsgChannel>();
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

    #[cfg(not(feature = "dispatcher"))]
    pub fn create() -> Result<Self> {
        let rt = js::Runtime::new().context(JsRuntimeSnafu)?;
        rt.set_max_stack_size(256 * 1024);
        rt.set_memory_limit(2 * 1024 * 1024);

        let ctx = Context::full(&rt).context(JsContextSnafu)?;
        rt.spawn_executor(Tokio);

        let engine = Self {
            runtime: rt,
            context: ctx,
        };
        engine.init_globals()?;
        Ok(engine)
    }

    #[cfg(feature = "builtin_processor")]
    pub fn create_with_processors(
        processors: Vec<(&str, &str, Box<dyn crate::Processor>)>,
    ) -> Result<Self, Error> {
        let (engine, rx) = Self::create()?;

        run_processors(rx, processors);
        Ok(engine)
    }

    pub async fn run(&self, code: &str, req: JsonValue) -> Result<JsonValue, Error> {
        let ret: Result<Promise<JsonValue>, js::Error> = self.context.with(|ctx| {
            let src = format!(r#"export default async function(req) {{ {} }}"#, code);
            debug!("code to execute: {}", src);
            let m = ctx.compile("script", src)?;
            let fun = m.get::<_, Function>("default")?;

            fun.call((req,))
        });
        ret.context(JsExecuteSnafu)?.await.context(JsExecuteSnafu)
    }

    pub fn load_global_js(&self, name: &str, code: &str) -> Result<()> {
        let ret: Result<(), js::Error> = self.context.with(|ctx| {
            let global = ctx.globals();
            let m = ctx.compile(name, code)?;
            let obj = m.get::<_, Object>("default")?;
            for item in obj.into_iter() {
                let (k, v) = item?;
                global.set(k, v)?;
            }
            Ok(())
        });
        ret.context(JsExecuteSnafu)
    }

    fn init_globals(&self) -> Result<(), Error> {
        let ret: Result<(), js::Error> = self.context.with(|ctx| {
            let global = ctx.globals();
            #[cfg(feature = "console")]
            {
                use crate::builtins::{con::Console, Con};
                global.init_def::<Con>()?;
                global.set("console", Console)?;
            }
            #[cfg(feature = "fetch")]
            {
                use crate::builtins::Fetch;
                ctx.globals().init_def::<Fetch>()?;
            }

            #[cfg(feature = "dispatcher")]
            {
                use crate::builtins::{disp::Dispatcher, Disp};
                global.init_def::<Disp>()?;
                global.set("dispatcher", Dispatcher::new(self.sender.clone()))?;
            }
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

#[cfg(all(feature = "builtin_processor", feature = "dispatcher"))]
fn run_processors(
    rx: flume::Receiver<crate::MsgChannel>,
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
            tracing::info!("Received request for {name}: {:#?}", msg.args);
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
            .run(
                "let a = 1;console.log(`hello world ${a}`, a)",
                JsonValue::null(),
            )
            .await?;
        let ret = engine
            .run(
                "return dispatcher.dispatch('auth', 'create_token', {a: 1})",
                JsonValue::null(),
            )
            .await?;

        assert_eq!(ret.0, json!({"a": 1}));
        Ok(())
    }

    #[cfg(feature = "fetch")]
    #[cfg(not(feature = "dispatcher"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fetch_should_work() {
        let engine = JsEngine::create().expect("valid");
        let ret = engine
            .run(
                "return await fetch('https://httpbin.org/get');",
                JsonValue::null(),
            )
            .await
            .expect("valid");
        assert_eq!(ret.0["url"], "https://httpbin.org/get");
    }
}
