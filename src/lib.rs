//! boluo 的额外组件库。

#![forbid(unsafe_code)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

#[cfg(feature = "cookie")]
pub mod cookie;
