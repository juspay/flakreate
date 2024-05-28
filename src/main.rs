use clap::Parser;
use inquire::Text;

#[derive(Parser, Debug)]
#[clap(author = "Sridhar Ratnakumar", version, about)]
/// Application configuration
struct Args {
    /// whether to be verbose
    #[arg(short = 'v')]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }
    println!("Welcome to flakreate! Let's create your flake template:");
    let registry = Text::new("Template registry")
        .with_help_message("Flake that contains a registry of templates to choose from")
        .with_placeholder("Flake URL reference")
        .with_default("github:flake-parts/templates")
        .prompt()?;
    println!("Using {}!", registry);

    // TODO Read flake-parts/templates and eval it to JSON, then Rust types
    // TODO Prompt to select a template to use
    // TODO Prompt for the parameter values of the template
    // TODO Run `nix flake init ...`, followed by the param patches
    Ok(())
}
