use crate::{JsonValue, MsgChannel};

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
