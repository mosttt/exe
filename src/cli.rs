use clap::{Args, Parser, Subcommand};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about = "upload app and executable operation", long_about = None)]
// #[command(name = "MyApp")]
// #[command(author = "Kevin K. <kbknapp@gmail.com>")]
// #[command(version = "1.0")]
// #[command(about = "Does awesome things", long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Run the application
    Run(Run),
    /// show the log of the application
    Log(Log),
}

#[derive(Args, Debug)]
pub(crate) struct Log {
    /// Path to the config file
    #[arg(long, short)]
    pub(crate) config: Option<Box<Path>>,

    /// id for which to execute the operation
    #[arg(long, short)]
    pub(crate) id: Option<String>,

    #[arg(long)]
    pub(crate) all_config: Option<Box<Path>>,

    #[arg(long)]
    pub(crate) all_id: bool,
}

#[derive(Args, Debug)]
pub(crate) struct Run {
    /// Path to the config file
    #[arg(long, short)]
    pub(crate) config: Option<Box<Path>>,

    /// id for which to execute the operation
    #[arg(long, short)]
    pub(crate) id: Option<String>,

    #[arg(long)]
    pub(crate) all_id: bool,
}
