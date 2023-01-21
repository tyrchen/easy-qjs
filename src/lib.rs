mod dispatcher;
mod engine;
pub(crate) mod error;
mod msg_channel;
mod value;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use error::Error;

#[async_trait]
pub trait Processor: Send + Sync + 'static {
    async fn call(&self, args: JsonValue) -> Result<JsonValue, String>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonValue(serde_json::Value);

pub struct JsEngine {
    #[allow(dead_code)]
    runtime: js::Runtime,
    context: js::Context,
    sender: flume::Sender<MsgChannel>,
}

#[derive(Debug)]
pub struct MsgChannel {
    /// the namespace of the calling function
    pub namespace: String,
    /// calling function name
    pub name: String,
    /// args for the calling function
    pub args: JsonValue,
    /// the sender of the response
    pub res: flume::Sender<Result<JsonValue, String>>,
}

#[async_trait]
impl<F> Processor for F
where
    F: Fn(JsonValue) -> Result<JsonValue, String> + Send + Sync + 'static,
{
    async fn call(&self, args: JsonValue) -> Result<JsonValue, String> {
        (self)(args)
    }
}
