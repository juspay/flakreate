use std::path::PathBuf;

use inquire::Text;
use serde::{Deserialize, Serialize};

use super::replace::{Replace, ReplaceOp};

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

impl Param {
    /// Prompt the user for a value for this parameter.
    ///
    /// Return a [`Replace`] that knows how to apply the replacement.
    pub fn prompt_value(&self) -> anyhow::Result<Replace> {
        let to = Text::new(&self.name)
            .with_help_message(&self.help)
            .with_placeholder(self.placeholder.as_deref().unwrap_or(""))
            .with_default(&self.default)
            .prompt()?;
        let from = self.default.clone();
        let replace = Replace {
            name: self.name.clone(),
            from: from.clone(),
            to: to.clone(),
            ops: ReplaceOp::ops_for_replacing(&from, &to, &self.files),
        };
        // TODO: return nothing if from == to
        Ok(replace)
    }
}
