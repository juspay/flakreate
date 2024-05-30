use std::collections::BTreeMap;

use nix_rs::{command::NixCmdError, flake::url::FlakeUrl};
use template::Template;

use crate::nixcmd;

pub mod fileop;
pub mod param;
/// Rust module for working with flake templates
///
/// Enriches native flake templates with support for replaceable parameters.
pub mod template;

/// Fetch the templates defined in a flake
pub async fn fetch(url: &FlakeUrl) -> Result<BTreeMap<String, Template>, NixCmdError> {
    nix_rs::flake::eval::nix_eval_attr_json::<BTreeMap<String, Template>>(
        nixcmd().await,
        url,
        false,
    )
    .await
}
