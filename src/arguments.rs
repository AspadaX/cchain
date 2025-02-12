use clap::Parser;

#[derive(Debug, Parser)]
pub struct Arguments {
    /// path to the command line chain file
    #[clap(short, long)]
    pub configuration_file: Option<String>,

    /// path to the directory containing the command line chain files
    #[clap(short = 'd', long)]
    pub configuration_files: Option<String>,

    /// use bookmarked command line chains
    #[clap(short, long)]
    pub bookmark: bool,

    /// delete a bookmark
    #[clap(short = 'r', long)]
    pub delete_bookmark: bool,

    /// generate a command line chain template
    #[clap(short, long)]
    pub generate: bool,
}
