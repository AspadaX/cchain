use clap::{
    builder::{
        styling::{AnsiColor, Effects},
        Styles,
    }, crate_authors, crate_version, crate_description, Args, Parser, Subcommand
};

// Configures Clap v3-style help menu colors
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Debug, Parser)]
#[command(name = "cchain", author = crate_authors!(), long_version = crate_version!())]
#[command(about = crate_description!())]
#[command(styles = STYLES)]
pub struct Arguments {
    /// Groupped features provided by `cchain`
    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run a chain
    Run(RunArguments),
    /// Add chain(s) to your bookmark
    Add(AddArguments),
    /// Add chain(s) to your bookmark
    List(ListArguments),
    /// Remove chain(s) to your bookmark
    #[clap(short_flag = 'r')]
    Remove(RemoveArguments),
    /// Validate the chain syntax
    Check(CheckArguments),
    /// Create a chain template
    New(NewArguments),
    /// Generate a chain
    #[clap(short_flag = 'g')]
    Generate(GenerateArguments),
    /// Check version info
    #[clap(short_flag = 'v')]
    Version(VersionArguments)
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct RunArguments {
    /// Index of the chain,
    /// or a path to a chain
    #[arg(group = "sources")]
    pub chain: String,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct AddArguments {
    /// Path to your chain file or a directory
    /// that contains multiple chains,
    /// or, add all chains under this directory to the bookmark
    #[arg(group = "sources")]
    pub path: String,
}

#[derive(Debug, Parser)]
pub struct ListArguments;

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct RemoveArguments {
    /// Index to your chain in the bookmark.
    /// Can be obtained with `cchain list`
    #[arg(group = "sources")]
    pub index: Option<usize>,
    /// Completely reset the bookmark. This is useful
    /// when `cchain` breaks.
    #[arg(short, long, group = "sources", default_value = "false")]
    pub reset: bool,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct CheckArguments {
    /// A path to a chain, or an index in the bookmark
    #[arg(group = "sources")]
    pub chain: String,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct NewArguments {
    /// Name the generated chain, by default,
    /// it will be a template file.
    #[arg(group = "sources")]
    pub name: Option<String>,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct GenerateArguments {
    /// Generate a command line chain but with LLM
    /// making the chain.
    #[arg(group = "sources")]
    pub llm: String,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(false).multiple(false))]
pub struct VersionArguments;