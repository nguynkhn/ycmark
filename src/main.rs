use clap::{ArgAction, Parser};
use comrak::Options;

use ycmark::{Format, Metadata, convert};

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

    #[arg(long, action = ArgAction::Append, value_parser = parse_key_value)]
    metadata: Vec<(String, String)>,
}

fn parse_key_value(input: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = input.splitn(2, ':').collect();

    match parts.len() {
        2 => Ok((parts[0].to_string(), parts[1].to_string())),
        _ => Err(format!("expected format key:value, got '{input}'")),
    }
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
    let metadata: Metadata = args.metadata.iter().cloned().collect();

    let output =
        convert(&input, args.to, template, metadata, args.to_options()).unwrap_or_else(|err| {
            eprintln!("error while reading metadata: {err}");
            input
        });

    match &args.output {
        Some(path) => fs::write(path, output).expect("failed to write to file"),
        None => println!("{output}"),
    };
}
