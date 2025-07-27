use clap::ValueEnum;
use comrak::{Options, markdown_to_commonmark, markdown_to_commonmark_xml, markdown_to_html};
use yaml_rust2::scanner::ScanError;

use crate::{
    metadata::{Metadata, extract_metadata, read_metadata},
    template::apply_template,
};

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
    mut metadata: Metadata,
    options: Options,
) -> Result<String, ScanError> {
    let (markdown, yaml) = extract_metadata(input).unwrap_or((input, ""));

    if !yaml.is_empty() {
        read_metadata(yaml, &mut metadata)?;
    }

    let mut output = match format {
        Format::Html => markdown_to_html(markdown, &options),
        Format::CommonMark => markdown_to_commonmark(markdown, &options),
        Format::Xml => markdown_to_commonmark_xml(markdown, &options),
    };
    output = output.trim().to_string();

    if let Some(template) = template {
        metadata.insert("body".to_string(), output);

        output = apply_template(template, metadata);
    }

    Ok(output)
}
