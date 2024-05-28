use std::collections::BTreeMap;

use clap::Parser;
use inquire::{Select, Text};
use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use serde::{Deserialize, Serialize};

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
    help: String,
    default: Option<String>,
    placeholder: Option<String>,
    exec: String,
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

    let cmd = NixCmd {
        refresh: true.into(),
        ..NixCmd::default()
    };
    let url: FlakeUrl = format!("{}#templates", registry).into();

    // Read flake-parts/templates and eval it to JSON, then Rust types
    let templates =
        nix_rs::flake::eval::nix_eval_attr_json::<BTreeMap<String, Template>>(&cmd, &url, false)
            .await?;
    let names = templates.keys().collect::<Vec<_>>();

    // Let the user pick the template
    let template = Select::new("Select a template", names)
        .with_help_message("Choose a flake template to use")
        .prompt()?;

    // Prompt for template parameters
    let param_values = templates
        .get(template)
        .unwrap()
        .params
        .iter()
        .map(|(name, param)| {
            let default = param.default.as_deref().unwrap_or("");
            let placeholder = param.placeholder.as_deref().unwrap_or("");
            let value = Text::new(name)
                .with_help_message(&param.help)
                .with_placeholder(placeholder)
                .with_default(default)
                .prompt()?;
            Ok((name.clone(), value))
        })
        .collect::<anyhow::Result<BTreeMap<String, String>>>()?;

    // println!("Templates: {:#?}", templates);
    println!("Res: {:#?}", param_values);

    // TODO Run `nix flake init ...`,
    // TODO Exec prompt 'exec's
    Ok(())
}
