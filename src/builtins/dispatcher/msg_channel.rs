use crate::JsonValue;

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

impl MsgChannel {
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
        args: JsonValue,
    ) -> (Self, flume::Receiver<Result<JsonValue, String>>) {
        let (sender, receiver) = flume::bounded(1);
        (
            Self {
                namespace: namespace.into(),
                name: name.into(),
                args,
                res: sender,
            },
            receiver,
        )
    }
}
