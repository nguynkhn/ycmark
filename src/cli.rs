use crate::Format;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long, value_enum, default_value_t = Format::Html)]
    pub to: Format,

    #[arg(value_name = "FILE")]
    pub file: Option<String>,

    #[arg(short = 'T', long)]
    pub template: Option<String>,

    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(long)]
    pub smart: bool,

    #[arg(long)]
    pub safe: bool,

    #[arg(long)]
    pub hardbreaks: bool,

    #[arg(long)]
    pub sourcepos: bool,

    #[arg(short, long, default_value_t = 0)]
    pub columns: usize,
}
