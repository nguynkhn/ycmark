use yaml_rust2::{scanner::ScanError, Yaml, YamlLoader};

use std::collections::HashMap;

pub type Metadata = HashMap<String, String>;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Scan(ScanError),
    InvalidType,
}

pub fn extract_yaml_block(input: &str) -> Option<(&str, &str)> {
    let mut lines = input.split_inclusive("\n");

    if let Some(begin_idx) = lines
        .next()
        .take_if(|line| line.trim_end() == "---")
        .map(str::len)
    {
        let end_idx = lines
            .by_ref()
            .take_while(|&line| !matches!(line.trim_end(), "---" | "..."))
            .fold(begin_idx, |acc, line| acc + line.len());
        let rest_idx = lines.fold(input.len(), |acc, line| acc - line.len());

        return Some((&input[begin_idx..end_idx], &input[rest_idx..]));
    }

    None
}

pub fn parse_metadata(input: &str) -> Result<Metadata, ParseError> {
    let yaml = YamlLoader::load_from_str(input).map_err(ParseError::Scan)?;
    let root = yaml
        .get(0)
        .take_if(|node| matches!(node, Yaml::Hash(_)) && yaml.len() == 1)
        .ok_or(ParseError::InvalidType)?;

    let mut metadata = Metadata::new();
    let mut queue = vec![("".to_string(), root)];

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
            _ => return Err(ParseError::InvalidType),
        };

        metadata.insert(key, value);
    }

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_success() {
        let input = "---
key: value
...
markdown";
        let result = extract_yaml_block(input);
        assert_eq!(result, Some(("key: value\n", "markdown")));
    }

    #[test]
    fn test_extract_success_2() {
        let input = "---
key: value";
        let result = extract_yaml_block(input);
        assert_eq!(result, Some(("key: value", "")));
    }


    #[test]
    fn test_extract_fail_begin() {
        let input = "paragraph
---
key: value";
        let result = extract_yaml_block(input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_fail_trailing() {
        let input = "
--- trailing
key: value
...
markdown";
        let result = extract_yaml_block(input);
        assert_eq!(result, None);
    }


    use super::*;

    #[test]
    fn test_parse_success() {
        let input = "title: yaml metadata
date: 20251107
list:
    - item 1
    - item 2
map:
    key: value";
        let result = parse_metadata(input);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.get("title"), Some(&"yaml metadata".to_string()));
        assert_eq!(metadata.get("date"), Some(&"20251107".to_string()));
        assert_eq!(metadata.get("list.0"), Some(&"item 1".to_string()));
        assert_eq!(metadata.get("list.1"), Some(&"item 2".to_string()));
        assert_eq!(metadata.get("map.key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_fail_scan() {
        let input = "this: is: not valid";
        let result = parse_metadata(input);
        assert!(matches!(result.err(), Some(ParseError::Scan(_))));
    }

    #[test]
    fn test_parse_fail_type() {
        let input = "what in the world";
        let result = parse_metadata(input);
        assert_eq!(result.err(), Some(ParseError::InvalidType));
    }
}
