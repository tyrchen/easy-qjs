use crate::{
    dispatcher::{disp::Dispatcher, Disp},
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

        let engine = Self {
            runtime: rt,
            context: ctx,
            sender: tx,
        };
        Ok(engine)
    }

    pub async fn run(&self, code: &str) -> Result<JsonValue, Error> {
        let ret: Result<Promise<JsonValue>, js::Error> = self.context.with(|ctx| {
            let glob = ctx.globals();
            glob.init_def::<Print>()?;
            glob.init_def::<Disp>()?;
            glob.set("dispatcher", Dispatcher::new(self.sender.clone()))?;

            let src = format!(r#"export default async function() {{ {} }}"#, code);
            debug!("code to execute: {}", src);
            let m = ctx.compile("script", src)?;
            let fun = m.get::<_, Function>("default")?;

            fun.call(())
        });
        ret.context(JsExecuteSnafu)?.await.context(JsExecuteSnafu)
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
        let engine = JsEngine::create(vec![(
            "auth".into(),
            "create_token".into(),
            Box::new(auth_create_token) as Box<dyn Processor>,
        )])?;
        engine.run("print('hello world')").await?;
        let ret = engine
            .run("return dispatcher.dispatch('auth', 'create_token', {a: 1})")
            .await?;

        assert_eq!(ret.0, json!({"a": 1}));
        Ok(())
    }
}
