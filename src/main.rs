mod cli;

use clap::Parser;
use std::fs;
use std::io::{self, Read};

use cli::Args;
use ycmark::{convert, Format, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input = match &args.file {
        Some(path) => fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let template = match &args.template {
        Some(path) => Some(fs::read_to_string(path)?),
        None => None,
    };

    let opts = Options {
        smart: args.smart,
        safe: args.safe,
        hardbreaks: args.hardbreaks,
        sourcepos: args.sourcepos,
        columns: args.columns,
    };

    let output = convert(&input, args.to, template, opts)?;

    match &args.output {
        Some(path) => fs::write(path, output)?,
        None => println!("{output}"),
    }

    Ok(())
}
