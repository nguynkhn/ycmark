use clap::Parser as CliParser;
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag};
use yaml_rust2::{Yaml, YamlLoader};

use std::collections::HashMap;

fn stringify(root: &Yaml) -> HashMap<String, String> {
    let mut map = HashMap::new();
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

fn convert(input: &str, template: Option<String>) -> String {
    let options = Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;
    let mut yaml_block: Option<CowStr> = None;
    let mut html_output = String::new();

    let mut parser = Parser::new_ext(input, options).peekable();
    if let Some(Event::Start(Tag::MetadataBlock(_))) = parser.peek() {
        if let Some(Event::Text(text)) = parser.nth(1) {
            yaml_block = Some(text);
        }
    }

    pulldown_cmark::html::push_html(&mut html_output, parser);
    html_output = html_output.trim().to_owned();

    if let Some(yaml_block) = yaml_block {
        let yaml = YamlLoader::load_from_str(&yaml_block).unwrap();
        let root = &yaml[0];

        let mut metadata = stringify(root);
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
}

fn main() {
    let args = Args::parse();

    let input = if let Some(file) = args.file {
        std::fs::read_to_string(file).expect("failed to read from file")
    } else {
        let stdin = std::io::stdin();
        std::io::read_to_string(stdin).expect("failed to read from stdin")
    };
    let template = args
        .template
        .map(|path| std::fs::read_to_string(path).expect("failed to read template file"));

    let result = convert(&input, template);
    if let Some(file) = args.output {
        std::fs::write(file, result).expect("failed to write to file");
    } else {
        println!("{result}");
    }
}
