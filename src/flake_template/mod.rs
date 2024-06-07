use std::collections::BTreeMap;

use fileop::FileOp;
use nix_rs::{command::NixCmdError, flake::url::FlakeUrl};
use param::Param;
use serde::{Deserialize, Serialize};

use crate::nixcmd;

pub mod fileop;
pub mod param;

/// A Nix flake template
///
/// Defined per [this definition](https://nix.dev/manual/nix/2.22/command-ref/new-cli/nix3-flake-init#template-definitions) in the flake.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlakeTemplate {
    description: String,
    path: String,
    #[serde(rename = "welcomeText")]
    welcome_text: Option<String>,
    params: BTreeMap<String, Param>,
}

impl FlakeTemplate {
    pub fn prompt_replacements(&self) -> anyhow::Result<BTreeMap<String, Vec<FileOp>>> {
        self.params
            .iter()
            .map(|(name, param)| Ok((name.clone(), param.prompt_value()?)))
            .collect()
    }
}

/// Fetch the templates defined in a flake
pub async fn fetch(url: &FlakeUrl) -> Result<BTreeMap<String, FlakeTemplate>, NixCmdError> {
    nix_rs::flake::eval::nix_eval_attr_json::<BTreeMap<String, FlakeTemplate>>(
        nixcmd().await,
        &url.with_attr("templates"),
        false,
    )
    .await
}
