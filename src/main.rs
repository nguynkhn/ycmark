use clap::Parser as CliParser;
use comrak::{markdown_to_html, Options as CMarkOptions};
use yaml_rust2::{Yaml, YamlLoader};

use std::collections::HashMap;

type Metadata = HashMap<String, String>;

fn stringify(root: &Yaml) -> Metadata {
    let mut map = Metadata::new();
    let mut queue = vec![];

    queue.push(("".to_string(), root));

    while let Some((key, node)) = queue.pop() {
        let value = match node {
            Yaml::Hash(h) => {
                for (name, node) in h.into_iter() {
                    let name = name.clone().into_string().unwrap();
                    let key = if key.is_empty() {
                        name
                    } else {
                        format!("{key}.{name}")
                    };

                    queue.push((key, node))
                }
                continue;
            }
            Yaml::Array(a) => {
                for (index, node) in a.iter().enumerate() {
                    let key = if key.is_empty() {
                        index.to_string()
                    } else {
                        format!("{key}.{index}")
                    };

                    queue.push((key, node))
                }
                continue;
            }
            Yaml::Real(r) => r.to_string(),
            Yaml::Integer(i) => i.to_string(),
            Yaml::String(s) => s.clone(),
            Yaml::Boolean(b) => b.to_string(),
            Yaml::Null => "null".to_string(),
            _ => "".to_string(),
        };

        map.insert(key, value);
    }

    map
}

struct Options {
    smart: bool,
    safe: bool,
}

impl Options {
    fn to_parser_options(&self) -> CMarkOptions {
        let mut options = CMarkOptions::default();

        if self.smart {
            options.parse.smart = true;
        }

        if self.safe {
            options.render.unsafe_ = false;
        }

        options
    }
}

fn convert(input: &str, template: Option<String>, options: Options) -> String {
    let options = options.to_parser_options();
    let mut metadata: Option<Metadata> = None;

    // detect yaml metadata
    let mut lines = input.trim().lines();

    if lines.next().is_some_and(|line| line == "---") {
        let yaml_block = lines
            .by_ref()
            .take_while(|&line| !matches!(line, "..." | "---"))
            .collect::<Vec<_>>()
            .join("\n");


        let yaml = YamlLoader::load_from_str(&yaml_block).unwrap();
        let root = &yaml[0];

        metadata = Some(stringify(root));
    }

    let input = &lines.collect::<Vec<_>>().join("\n");
    let mut html_output = markdown_to_html(input, &options).trim().to_string();
    if let Some(mut metadata) = metadata {
        metadata.insert("body".to_string(), html_output.clone());

        if let Some(template) = template {
            html_output = template.clone();

            metadata.into_iter().for_each(|(key, value)| {
                html_output = html_output.replace(&format!("${key}$"), &value);
            });
        }
    }

    html_output
}

#[derive(Debug, CliParser)]
#[command(version, about)]
struct Args {
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
    };

    let result = convert(&input, template, options);
    //  TODO: rewrite this
    if let Some(file) = args.output {
        std::fs::write(file, result).expect("failed to write to file");
    } else {
        println!("{result}");
    }
}
