use std::collections::BTreeMap;

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
    name: String,
    help: String,
    default: Option<String>,
    placeholder: Option<String>,
    exec: String,
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

    pub fn prompt_values(&self) -> anyhow::Result<BTreeMap<String, String>> {
        self.params
            .iter()
            .map(|(name, param)| Ok((name.clone(), param.prompt_value()?)))
            .collect()
    }
}

impl Param {
    pub fn prompt_value(&self) -> anyhow::Result<String> {
        let value = Text::new(&self.name)
            .with_help_message(&self.help)
            .with_placeholder(self.placeholder.as_deref().unwrap_or(""))
            .with_default(self.default.as_deref().unwrap_or(""))
            .prompt()?;
        Ok(value)
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

    // Prompt for template parameters
    let param_values = templates.get(template).unwrap().prompt_values()?;

    // println!("Templates: {:#?}", templates);
    println!("Res: {:#?}", param_values);

    // TODO Run `nix flake init ...`,
    // TODO Exec prompt 'exec's
    Ok(())
}
