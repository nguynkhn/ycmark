use clap::ValueEnum;
use comrak::{Options, markdown_to_commonmark, markdown_to_commonmark_xml, markdown_to_html};
use yaml_rust2::scanner::ScanError;

use crate::metadata::{extract_metadata, parse_metadata};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Format {
    Html,
    CommonMark,
    Xml,
}

pub fn convert(
    input: &str,
    format: Format,
    template: Option<String>,
    options: Options,
) -> Result<String, ScanError> {
    let (markdown, metadata_str) = extract_metadata(input)
        .map(|(body, meta)| (body, Some(meta)))
        .unwrap_or((input, None));

    let metadata = match metadata_str {
        Some(meta) => Some(parse_metadata(meta)?),
        None => None,
    };

    let mut output = match format {
        Format::Html => markdown_to_html(markdown, &options),
        Format::CommonMark => markdown_to_commonmark(markdown, &options),
        Format::Xml => markdown_to_commonmark_xml(markdown, &options),
    };
    output = output.trim().to_string();

    if let (Some(template), Some(metadata)) = (template, metadata) {
        output = template.replace("$body$", &output);

        metadata.into_iter().for_each(|(key, value)| {
            output = output.replace(&format!("${key}$"), &value);
        });
    }

    Ok(output)
}
