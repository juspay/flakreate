use std::{collections::BTreeMap, path::PathBuf};

use inquire::Text;
use nix_rs::{command::NixCmdError, flake::url::FlakeUrl};
use serde::{Deserialize, Serialize};

use crate::nixcmd;

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
