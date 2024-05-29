use std::{collections::BTreeMap, path::PathBuf};

use clap::Parser;
use inquire::{Select, Text};
use nix_rs::{
    command::{NixCmd, NixCmdError},
    flake::url::FlakeUrl,
};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

static NIXCMD: OnceCell<NixCmd> = OnceCell::const_new();

pub async fn nixcmd() -> &'static NixCmd {
    NIXCMD
        .get_or_init(|| async {
            NixCmd {
                refresh: true.into(),
                ..NixCmd::with_flakes().await.unwrap()
            }
        })
        .await
}

#[derive(Parser, Debug)]
#[clap(author = "Sridhar Ratnakumar", version, about)]
/// Application configuration
struct Args {
    /// whether to be verbose
    #[arg(short = 'v')]
    verbose: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Template {
    description: String,
    path: String,
    params: BTreeMap<String, Param>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Param {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Replace {
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

    pub fn prompt_values(&self) -> anyhow::Result<BTreeMap<String, Replace>> {
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }
    println!("Welcome to flakreate! Let's create your flake template:");
    let registry = Text::new("Template registry")
        .with_help_message("Flake that contains a registry of templates to choose from")
        .with_placeholder("Flake URL reference")
        .with_default("github:flake-parts/templates/flakreate")
        .prompt()?;
    println!("Using {}!", registry);

    let url: FlakeUrl = format!("{}#templates", registry).into();

    // Read flake-parts/templates and eval it to JSON, then Rust types
    let templates = Template::fetch_flake_templates(&url).await?;
    // TODO: avoid duplicates (aliases)
    let names = templates.keys().collect::<Vec<_>>();

    // Let the user pick the template
    let template = Select::new("Select a template", names)
        .with_help_message("Choose a flake template to use")
        .prompt()?;

    let path = Text::new("Directory path")
        .with_help_message("Path to create the flake in")
        .with_placeholder("Filepath")
        .with_default("./tmp")
        .prompt()?;

    // Prompt for template parameters
    let param_values = templates.get(template).unwrap().prompt_values()?;

    // println!("Templates: {:#?}", templates);
    println!("Res: {:#?}", param_values);

    // Create directory path
    tokio::fs::create_dir_all(&path).await?;
    // change working directory to 'path'
    std::env::set_current_dir(&path)?;

    // Run nix flake init
    let template_url = format!("{}#{}", registry, template);
    println!("Running: nix flake init -t {}", template_url);
    nixcmd()
        .await
        .run_with_args_returning_stdout(&["flake", "init", "-t", &template_url])
        .await?;

    // Do the actual replacement
    for replace in param_values.values() {
        replace.apply().await?;
    }
    Ok(())
}
