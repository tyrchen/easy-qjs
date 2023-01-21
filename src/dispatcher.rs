use crate::JsonValue;
use js::Rest;

#[js::bind(object, public)]
#[quickjs(bare)]
#[allow(non_upper_case_globals)]
pub mod disp {
    use crate::{JsonValue, MsgChannel};
    use tracing::{info, warn};

    #[derive(Clone)]
    pub struct Dispatcher {
        #[quickjs(hide)]
        pub(super) sender: flume::Sender<MsgChannel>,
    }

    impl Dispatcher {
        #[quickjs(constructor = false)]
        #[quickjs(skip)]
        pub fn new(sender: flume::Sender<MsgChannel>) -> Self {
            Self { sender }
        }

        pub fn dispatch(
            &self,
            ns: String,
            name: String,
            args: JsonValue,
        ) -> Result<JsonValue, js::Error> {
            info!("dispatch: {} {} {:?}", ns, name, args);
            let (msg, res) = MsgChannel::new(ns, name, args);
            self.sender
                .send(msg)
                .map_err(|_| js::Error::UnrelatedRuntime)?;
            res.recv()
                .map_err(|e| {
                    warn!("recv error: {:?}", e);
                    js::Error::UnrelatedRuntime
                })?
                .map_err(|e| {
                    warn!("execution error: {:?}", e);
                    js::Error::Unknown
                })
        }
    }
}

#[js::bind(object, public)]
#[quickjs(rename = "fetch")]
#[allow(unused_variables)]
pub async fn fetch(args: Rest<JsonValue>) -> Result<JsonValue, js::Error> {
    Ok(JsonValue::default())
}
