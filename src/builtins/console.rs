#[js::bind(object, public)]
#[quickjs(bare)]
#[allow(non_upper_case_globals)]
pub(crate) mod con {
    use crate::JsonValue;
    use tracing::{error, info, warn};

    #[derive(Debug, Clone)]
    pub struct Console;

    impl Console {
        #[allow(clippy::unused_self)]
        pub fn log(&self, args: js::Rest<JsonValue>) {
            if atty::is(atty::Stream::Stdout) {
                println!("{}", to_vec_string(args).join(" "));
            } else {
                info!("{}", to_vec_string(args).join(" "));
            }
        }

        #[allow(clippy::unused_self)]
        pub fn warn(&self, args: js::Rest<JsonValue>) {
            if atty::is(atty::Stream::Stdout) {
                println!("{}", to_vec_string(args).join(" "));
            } else {
                warn!("{}", to_vec_string(args).join(" "));
            }
        }

        #[allow(clippy::unused_self)]
        pub fn error(&self, args: js::Rest<JsonValue>) {
            if atty::is(atty::Stream::Stdout) {
                println!("{}", to_vec_string(args).join(" "));
            } else {
                error!("{}", to_vec_string(args).join(" "));
            }
        }
    }

    fn to_vec_string(args: js::Rest<JsonValue>) -> Vec<String> {
        let mut v = Vec::new();
        for arg in args.0.iter() {
            v.push(arg.to_string());
        }
        v
    }
}
