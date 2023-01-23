mod msg_channel;

pub use msg_channel::MsgChannel;

#[js::bind(object, public)]
#[quickjs(bare)]
#[allow(non_upper_case_globals)]
pub(crate) mod disp {
    use std::time::Duration;

    use crate::{JsonValue, MsgChannel};
    use tracing::{info, warn};

    #[derive(Debug, Clone)]
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
                .send_timeout(msg, Duration::from_millis(100))
                .map_err(|_| js::Error::UnrelatedRuntime)?;
            res.recv_timeout(Duration::from_millis(500))
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
