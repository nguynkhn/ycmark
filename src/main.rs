use pulldown_cmark::{CowStr, Event, Options, Parser, Tag};
use yaml_rust2::{Yaml, YamlLoader};

use std::collections::HashMap;

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

    if let Some(yaml_block) = yaml_block {
        let yaml = YamlLoader::load_from_str(&yaml_block).unwrap();
        let root = yaml[0].as_hash().unwrap();

        let mut metadata: HashMap<String, String> = root
            .into_iter()
            .map(|(key, value)| {
                let key = key.clone().into_string().unwrap();
                let value = match value {
                    Yaml::Real(r) => r.to_string(),
                    Yaml::Integer(i) => i.to_string(),
                    Yaml::String(s) => s.clone(),
                    Yaml::Boolean(b) => b.to_string(),
                    Yaml::Null => "null".to_string(),
                    _ => "".to_string(),
                };

                (key, value)
            })
            .collect();

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

fn main() {
}
