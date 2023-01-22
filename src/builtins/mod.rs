#[cfg(feature = "console")]
mod console;
mod dispatcher;
#[cfg(feature = "fetch")]
mod fetch;

#[cfg(feature = "console")]
pub(crate) use console::*;
pub(crate) use dispatcher::*;
#[cfg(feature = "fetch")]
pub(crate) use fetch::*;
