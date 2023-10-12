use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(short = 'V', long = "vsync")]
    pub vsync: bool,
}
