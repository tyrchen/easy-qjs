mod dispatcher;
mod engine;
pub(crate) mod error;
mod msg_channel;
mod value;

use serde::{Deserialize, Serialize};

pub use error::Error;

pub type Processor = fn(Option<JsonValue>) -> Result<JsonValue, String>;

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
    pub args: Option<JsonValue>,
    /// the sender of the response
    pub res: flume::Sender<Result<JsonValue, String>>,
}
