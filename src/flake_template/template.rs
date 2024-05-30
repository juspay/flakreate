use std::{collections::BTreeMap, path::PathBuf};

use inquire::Text;
use nix_rs::{command::NixCmdError, flake::url::FlakeUrl};
use serde::{Deserialize, Serialize};

use crate::nixcmd;

/// A Nix flake template
///
/// Defined per the spec in [nix flake init](https://nix.dev/manual/nix/2.22/command-ref/new-cli/nix3-flake-init)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    description: String,
    path: String,
    params: BTreeMap<String, Param>,
}

/// A parameter to be filled in by the user in a nix flake template path.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Param {
    /// Main message when prompting the user for input
    name: String,
    /// Message displayed at the line below the prompt.
    help: String,
    /// The default value used in the template files, that must be replaced by
    /// the user provided value (if it is different)
    default: String,
    /// Short hint that describes the expected value of the input.
    placeholder: Option<String>,
    /// Files to do replacement on.
    files: Vec<PathBuf>,
    /// Whether the user must provide a value
    #[serde(default)]
    required: bool,
}

/// Replacement semantics for a [`Param`]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Replace {
    name: String,
    from: String,
    to: String,
    /// The files to do replacement on.
    ///
    /// Replacements happen on file *content* as well as file *name*. When the
    /// later happens, the file is naturally renamed.
    files: Vec<PathBuf>,
}

impl Replace {
    pub async fn apply(&self) -> anyhow::Result<()> {
        // TODO: Refactor the LLM generated code below
        for file in &self.files {
            let content = tokio::fs::read_to_string(file).await?;
            let content = content.replace(&self.from, &self.to);
            println!("REPLACE[{}]: {}", self.name, file.display());
            tokio::fs::write(file, content).await?;
            // Now, rename the file if filename contains 'from' as substring
            if file.to_string_lossy().contains(&self.from) {
                let new_file = file.with_file_name(
                    file.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .replace(&self.from, &self.to),
                );
                println!(
                    "RENAME[{}]: {} -> {}",
                    self.name,
                    file.display(),
                    new_file.display()
                );
                tokio::fs::rename(file, new_file).await?;
            }
        }
        Ok(())
    }
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

impl Param {
    pub fn prompt_value(&self) -> anyhow::Result<Replace> {
        let to = Text::new(&self.name)
            .with_help_message(&self.help)
            .with_placeholder(self.placeholder.as_deref().unwrap_or(""))
            .with_default(&self.default)
            .prompt()?;
        let from = self.default.clone();
        let replace = Replace {
            name: self.name.clone(),
            from,
            to,
            files: self.files.clone(),
        };
        // TODO: return nothing if from == to
        Ok(replace)
    }
}
