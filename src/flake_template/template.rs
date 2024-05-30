use std::collections::BTreeMap;

use nix_rs::{command::NixCmdError, flake::url::FlakeUrl};
use serde::{Deserialize, Serialize};

use crate::nixcmd;

use super::{param::Param, replace::Replace};

/// A Nix flake template
///
/// Defined per the spec in [nix flake init](https://nix.dev/manual/nix/2.22/command-ref/new-cli/nix3-flake-init)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    description: String,
    path: String,
    params: BTreeMap<String, Param>,
}

impl Template {
    pub async fn fetch_flake_templates(
        url: &FlakeUrl,
    ) -> Result<BTreeMap<String, Self>, NixCmdError> {
        nix_rs::flake::eval::nix_eval_attr_json::<BTreeMap<String, Template>>(
            nixcmd().await,
            url,
            false,
        )
        .await
    }

    pub fn prompt_replacements(&self) -> anyhow::Result<BTreeMap<String, Replace>> {
        self.params
            .iter()
            .map(|(name, param)| Ok((name.clone(), param.prompt_value()?)))
            .collect()
    }
}
