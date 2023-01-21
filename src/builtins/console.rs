#[js::bind(object, public)]
#[quickjs(bare)]
#[allow(non_upper_case_globals)]
pub mod con {
    use crate::JsonValue;
    use tracing::{error, info, warn};

    #[derive(Debug, Clone)]
    pub struct Console;

    impl Console {
        pub fn log(&self, args: js::Rest<JsonValue>) {
            if atty::is(atty::Stream::Stdout) {
                println!("{}", self.to_vec_string(args).join(" "));
            } else {
                info!("{}", self.to_vec_string(args).join(" "));
            }
        }

        pub fn warn(&self, args: js::Rest<JsonValue>) {
            if atty::is(atty::Stream::Stdout) {
                println!("{}", self.to_vec_string(args).join(" "));
            } else {
                warn!("{}", self.to_vec_string(args).join(" "));
            }
        }

        pub fn error(&self, args: js::Rest<JsonValue>) {
            if atty::is(atty::Stream::Stdout) {
                println!("{}", self.to_vec_string(args).join(" "));
            } else {
                error!("{}", self.to_vec_string(args).join(" "));
            }
        }

        fn to_vec_string(&self, args: js::Rest<JsonValue>) -> Vec<String> {
            let mut v = Vec::new();
            for arg in args.0.iter() {
                v.push(arg.to_string());
            }
            v
        }
    }
}
