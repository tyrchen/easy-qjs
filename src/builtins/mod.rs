#[cfg(feature = "console")]
pub(crate) mod console;
#[cfg(feature = "dispatcher")]
pub(crate) mod dispatcher;
#[cfg(feature = "fetch")]
pub(crate) mod fetch;

#[cfg(feature = "console")]
pub(crate) use console::*;
#[cfg(feature = "dispatcher")]
pub(crate) use dispatcher::*;
#[cfg(feature = "fetch")]
pub(crate) use fetch::*;
