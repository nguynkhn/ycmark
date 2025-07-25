use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, line_ending, multispace0, not_line_ending, space0, space1},
    combinator::{eof, peek, recognize},
    multi::many_till,
    sequence::{delimited, preceded},
};
use yaml_rust2::{YamlLoader, scanner::ScanError, yaml::Yaml};

use std::collections::HashMap;

pub type Metadata = HashMap<String, String>;

fn stringify_yaml(node: &Yaml) -> String {
    match node {
        Yaml::Real(r) => r.to_string(),
        Yaml::Integer(i) => i.to_string(),
        Yaml::String(s) => s.clone(),
        Yaml::Boolean(b) => b.to_string(),
        Yaml::Null => "null".to_string(),
        _ => "".to_string(),
    }
}

fn flatten_yaml(root: &Yaml, metadata: &mut Metadata) {
    let mut queue = vec![("".to_string(), root)];

    while let Some((prefix, node)) = queue.pop() {
        match node {
            Yaml::Hash(h) => h.iter().for_each(|(k, v)| {
                if let Some(k_str) = k.as_str() {
                    let new_key = if prefix.is_empty() {
                        k_str.to_string()
                    } else {
                        format!("{prefix}.{k_str}")
                    };

                    queue.push((new_key, v));
                }
            }),
            Yaml::Array(a) => a.iter().enumerate().for_each(|(i, v)| {
                let new_key = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{prefix}.{i}")
                };

                queue.push((new_key, v));
            }),
            _ => {
                let value = stringify_yaml(node);
                metadata.insert(prefix, value);
            }
        }
    }
}

pub fn extract_metadata(input: &str) -> IResult<&str, &str> {
    let yaml_comment = || recognize((tag("#"), not_line_ending));
    let yaml_begin_line = recognize((
        multispace0,
        tag("---"),
        alt((preceded(space1, yaml_comment()), space0)),
        line_ending,
    ));
    let yaml_end_line = || {
        recognize((
            multispace0,
            alt((tag("..."), tag("---"))),
            alt((preceded(space1, yaml_comment()), space0)),
            alt((line_ending, eof)),
        ))
    };
    let yaml_content = recognize(many_till(anychar, peek(yaml_end_line())));
    delimited(yaml_begin_line, yaml_content, yaml_end_line()).parse(input)
}

pub fn read_metadata(input: &str) -> Result<Metadata, ScanError> {
    let docs = YamlLoader::load_from_str(input)?;
    let mut metadata = Metadata::new();

    if let Some(root) = docs.first().filter(|node| node.is_hash()) {
        flatten_yaml(root, &mut metadata);
    }

    Ok(metadata)
}
