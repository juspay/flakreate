use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Replacement semantics for a [`Param`]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Replace {
    pub name: String,
    pub from: String,
    pub to: String,
    /// The files to do replacement on.
    ///
    /// Replacements happen on file *content* as well as file *name*. When the
    /// later happens, the file is naturally renamed.
    pub ops: Vec<ReplaceOp>,
}

impl Replace {
    pub async fn apply(&self) -> anyhow::Result<()> {
        // TODO: Refactor the LLM generated code below
        for op in &self.ops {
            match op {
                ReplaceOp::ContentReplace(file, from, to) => {
                    let content = tokio::fs::read_to_string(file).await?;
                    let content = content.replace(from, to);
                    println!(
                        "REPLACE[{}]: {} : {} -> {}",
                        self.name,
                        file.display(),
                        from,
                        to
                    );
                    tokio::fs::write(file, content).await?;
                }
                ReplaceOp::FileRename(file, new_name) => {
                    println!("RENAME[{}]: {} -> {}", self.name, file.display(), new_name);
                    tokio::fs::rename(file, new_name).await?;
                }
            }
        }
        Ok(())
    }
}

/// FIXME: Don't need [`Replace`] when this is sufficient!
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ReplaceOp {
    /// Replace all occurrences of `from` with `to` in the file content
    ContentReplace(PathBuf, String, String),
    /// Rename the file to the given name
    FileRename(PathBuf, String),
}

impl ReplaceOp {
    pub fn ops_for_replacing(from: &str, to: &str, files: &[PathBuf]) -> Vec<ReplaceOp> {
        files
            .iter()
            .flat_map(|file| {
                let mut items: Vec<ReplaceOp> = vec![];
                if to != from {
                    items.push(ReplaceOp::ContentReplace(
                        file.clone(),
                        from.to_string(),
                        to.to_string(),
                    ));
                    if file.to_string_lossy().contains(from) {
                        items.push(ReplaceOp::FileRename(
                            file.clone(),
                            file.to_string_lossy().replace(from, to),
                        ))
                    }
                }
                items
            })
            .collect()
    }
}
