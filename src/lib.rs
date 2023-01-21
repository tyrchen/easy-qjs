#![warn(
  clippy::all,
  clippy::dbg_macro,
  clippy::todo,
  clippy::empty_enum,
  clippy::enum_glob_use,
  clippy::mem_forget,
  clippy::unused_self,
  clippy::filter_map_next,
  clippy::needless_continue,
  clippy::needless_borrow,
  clippy::match_wildcard_for_single_variants,
  clippy::if_let_mutex,
  clippy::mismatched_target_os,
  clippy::await_holding_lock,
  clippy::match_on_vec_items,
  clippy::imprecise_flops,
  clippy::suboptimal_flops,
  clippy::lossy_float_literal,
  clippy::rest_pat_in_fully_bound_structs,
  clippy::fn_params_excessive_bools,
  clippy::exit,
  clippy::inefficient_to_string,
  clippy::linkedlist,
  clippy::macro_use_imports,
  clippy::option_option,
  clippy::verbose_file_reads,
  clippy::unnested_or_patterns,
  clippy::str_to_string,
  rust_2018_idioms,
  future_incompatible,
  nonstandard_style,
  missing_debug_implementations,
  clippy::unwrap_used,
  // missing_docs
)]
#![deny(unreachable_pub, private_in_public)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]

mod builtins;
mod engine;
pub(crate) mod error;
mod msg_channel;
mod value;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use error::Error;

#[async_trait]
pub trait Processor: Send + Sync + 'static {
    async fn call(&self, args: JsonValue) -> Result<JsonValue, String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub args: JsonValue,
    /// the sender of the response
    pub res: flume::Sender<Result<JsonValue, String>>,
}

#[async_trait]
impl<F> Processor for F
where
    F: Fn(JsonValue) -> Result<JsonValue, String> + Send + Sync + 'static,
{
    async fn call(&self, args: JsonValue) -> Result<JsonValue, String> {
        (self)(args)
    }
}
