use clap::Parser;

#[derive(Debug, Parser)]
pub struct Arguments {
    // path to the command line chain configuration file
    #[clap(short, long)]
    pub configuration_file: Option<String>,

    // path to the directory containing the command line chain configuration files
    #[clap(short = 'd', long)]
    pub configuration_files: Option<String>,

    // use bookmarked configuration files
    #[clap(short, long)]
    pub bookmark: bool,

    // delete a bookmark
    #[clap(short = 'r', long)]
    pub delete_bookmark: bool,

    // generate a configuration template
    #[clap(short, long)]
    pub generate: bool,
}
