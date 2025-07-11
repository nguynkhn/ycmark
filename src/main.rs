use clap::Parser;

use ycmark::{convert, Format, Options, ParseError};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long, value_name = "FORMAT", value_enum, default_value_t = Format::Html)]
    to: Format,

    #[arg(value_name = "FILE")]
    file: Option<String>,

    #[arg(short = 'T', long, value_name = "FILE")]
    template: Option<String>,

    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    #[arg(long)]
    smart: bool,

    #[arg(long)]
    safe: bool,

    #[arg(long)]
    hardbreaks: bool,

    #[arg(long)]
    sourcepos: bool,

    #[arg(short, long, value_name = "NUMBER", default_value_t = 0)]
    columns: usize,
}

fn main() {
    let args = Args::parse();

    let input = args.file.map_or_else(
        || {
            let stdin = std::io::stdin();
            std::io::read_to_string(stdin).expect("failed to read from stdin")
        },
        |file| std::fs::read_to_string(file).expect("failed to read from file"),
    );
    let template = args
        .template
        .map(|path| std::fs::read_to_string(path).expect("failed to read template file"));
    let options = Options {
        smart: args.smart,
        safe: args.safe,
        hardbreaks: args.hardbreaks,
        sourcepos: args.sourcepos,
        columns: args.columns,
    };

    match convert(&input, args.to, template, options) {
        Ok(output) => {
            if let Some(file) = args.output {
                std::fs::write(file, output).expect("failed to write to file");
            } else {
                println!("{output}");
            }
        }
        Err(ParseError::Scan(err)) => eprintln!("failed to read metadata: {}", err.info()),
        Err(ParseError::InvalidType) => eprintln!("failed to read metadata: unexpected type"),
    };
}
