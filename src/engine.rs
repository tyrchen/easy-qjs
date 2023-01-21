use crate::{
    builtins::{con::Console, disp::Dispatcher, Con, Disp},
    error::*,
    JsEngine, JsonValue, MsgChannel, Processor,
};
use std::collections::HashMap;

use js::{Context, Function, Promise, Tokio};
use snafu::ResultExt;
use tracing::{debug, info, warn};
impl JsEngine {
    pub fn create(processors: Vec<(String, String, Box<dyn Processor>)>) -> Result<Self, Error> {
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

        let names: Vec<(&str, &str)> = processors
            .iter()
            .map(|(ns, name, _)| (ns.as_str(), name.as_str()))
            .collect();
        engine.init_globals(&names)?;

        engine.run_processors(rx, processors);
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

    fn run_processors(
        &self,
        rx: flume::Receiver<MsgChannel>,
        processors: Vec<(String, String, Box<dyn Processor>)>,
    ) {
        let processors: HashMap<(String, String), Box<dyn Processor>> = processors
            .into_iter()
            .map(|(ns, name, processor)| ((ns, name), processor))
            .collect();

        tokio::spawn(async move {
            while let Ok(msg) = rx.recv_async().await {
                let name = format!("{}.{}", msg.namespace, msg.name);
                info!("Got request for {name}: {:#?}", msg.args);
                let processor = match processors.get(&(msg.namespace, msg.name)) {
                    Some(p) => p,
                    None => {
                        if let Err(e) = msg.res.send(Err(format!("{} not found", name))) {
                            warn!("send error: {:?}", e);
                        };
                        continue;
                    }
                };
                let ret = processor.call(msg.args).await;
                if let Err(e) = msg.res.send(ret) {
                    warn!("send error: {:?}", e);
                }
            }
        });
    }

    fn init_globals(&self, _names: &[(&str, &str)]) -> Result<(), Error> {
        let ret: Result<(), js::Error> = self.context.with(|ctx| {
            let glob = ctx.globals();
            glob.init_def::<Con>()?;
            glob.init_def::<Disp>()?;
            glob.set("console", Console)?;
            glob.set("dispatcher", Dispatcher::new(self.sender.clone()))?;
            Ok(())
        });
        ret.context(JsExecuteSnafu)
    }
}

#[js::bind(object, public)]
fn print(s: String) {
    println!("{}", s);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use serde_json::json;

    fn auth_create_token(args: JsonValue) -> std::result::Result<JsonValue, String> {
        Ok(args)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn js_engine_should_work() -> Result<()> {
        tracing_subscriber::fmt::init();
        let engine = JsEngine::create(vec![(
            "auth".into(),
            "create_token".into(),
            Box::new(auth_create_token) as Box<dyn Processor>,
        )])?;
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
