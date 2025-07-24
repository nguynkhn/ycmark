use clap::Parser;
use comrak::Options;

use ycmark::{Format, convert};

use std::{fs, io};

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[arg(value_name = "FILE")]
    file: Option<String>,

    #[arg(short, long, value_name = "FORMAT", value_enum, default_value_t = Format::Html)]
    to: Format,

    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    #[arg(short = 'T', long, value_name = "FILE")]
    template: Option<String>,

    #[arg(long, value_name = "COLUMNS", default_value_t = 0)]
    columns: usize,

    #[arg(long)]
    smart: bool,

    #[arg(long)]
    unsafe_: bool,

    #[arg(long)]
    hardbreaks: bool,

    #[arg(long)]
    sourcepos: bool,
}

impl Args {
    fn to_options(&self) -> Options {
        let mut options = Options::default();

        options.render.width = self.columns;
        options.parse.smart = self.smart;
        options.render.unsafe_ = self.unsafe_;
        options.render.hardbreaks = self.hardbreaks;
        options.render.sourcepos = self.sourcepos;

        options
    }
}

fn main() {
    let args = Args::parse();

    let input = args.file.as_ref().map_or_else(
        || io::read_to_string(io::stdin()).expect("failed to read from stdin"),
        |path| fs::read_to_string(path).expect("failed to read from file"),
    );
    let template = args
        .template
        .as_ref()
        .map(|path| fs::read_to_string(path).expect("failed to read template"));

    let output = convert(&input, args.to, template, args.to_options()).unwrap_or_else(|err| {
        eprintln!("error while reading metadata: {err}");
        input
    });

    match &args.output {
        Some(path) => fs::write(path, output).expect("failed to write to file"),
        None => println!("{output}"),
    };
}
