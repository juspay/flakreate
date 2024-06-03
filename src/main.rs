use clap::Parser;
use flakreate::{
    flake_template::{self, fileop::FileOp},
    nixcmd,
};
use inquire::{Select, Text};
use nix_rs::flake::url::FlakeUrl;

#[derive(Parser, Debug)]
#[clap(author = "Sridhar Ratnakumar", version, about)]
/// Application configuration
struct Args {
    /// whether to be verbose
    #[arg(short = 'v')]
    verbose: bool,

    /// Flake template registry to use
    #[arg(short = 'r', default_value = "github:flake-parts/templates/flakreate")]
    registry: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }

    let url: FlakeUrl = format!("{}#templates", args.registry).into();

    // Read flake-parts/templates and eval it to JSON, then Rust types
    let term = console::Term::stdout();
    term.write_line(format!("Loading registry {}...", args.registry).as_str())?;
    let templates = flake_template::fetch(&url).await?;
    term.clear_last_lines(1)?;
    println!("Loaded registry {}", args.registry);

    // TODO: avoid duplicates (aliases)
    let names = templates.keys().collect::<Vec<_>>();

    println!("Welcome to flakreate! Let's create your flake template:");

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
    let param_values = templates.get(template).unwrap().prompt_replacements()?;

    // println!("Res: {:#?}", param_values);

    // Create directory path
    tokio::fs::create_dir_all(&path).await?;
    // change working directory to 'path'
    std::env::set_current_dir(&path)?;

    // Run nix flake init
    let template_url = format!("{}#{}", args.registry, template);
    println!("$ nix flake init -t {}", template_url);
    nixcmd()
        .await
        .run_with_args_returning_stdout(&["flake", "init", "-t", &template_url])
        .await?;

    // Do the actual replacement
    for replace in param_values.values() {
        FileOp::apply(replace).await?;
    }
    Ok(())
}
