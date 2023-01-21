use snafu::Snafu;

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    // failed to create js runtime
    #[snafu(display("Failed to create javascript runtime"))]
    JsRuntime { source: js::Error },
    // failed to create js context
    #[snafu(display("Failed to create javascript context"))]
    JsContext { source: js::Error },
    #[snafu(display("Failed to execute javascript code"))]
    JsExecute { source: js::Error },
    #[snafu(display("Javascript code returned an error: {}", msg))]
    JsResult { msg: String },
}
