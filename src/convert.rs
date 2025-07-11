use comrak::{markdown_to_commonmark, markdown_to_commonmark_xml, markdown_to_html};

use crate::metadata::{extract_yaml_block, parse_metadata, Metadata, ParseError};

#[derive(Debug, Default, Clone, clap::ValueEnum)]
#[clap(rename_all = "lowercase")]
pub enum Format {
    #[default]
    Html,
    CommonMark,
    Xml,
}

#[derive(Debug, Default)]
pub struct Options {
    pub smart: bool,
    pub safe: bool,
    pub hardbreaks: bool,
    pub sourcepos: bool,
    pub columns: usize,
}

impl Options {
    pub fn to_cmark_options(&self) -> comrak::Options {
        let mut options = comrak::Options::default();

        options.parse.smart = self.smart;
        options.render.unsafe_ = !self.safe;
        options.render.hardbreaks = self.hardbreaks;
        options.render.sourcepos = self.sourcepos;
        options.render.width = self.columns;

        options
    }
}

pub fn convert(
    input: &str,
    format: Format,
    template: Option<String>,
    options: Options,
) -> Result<String, ParseError> {
    let options = options.to_cmark_options();
    let mut metadata: Option<Metadata> = None;
    let mut markdown_body = input;

    if let Some((yaml_block, _markdown_body)) = extract_yaml_block(input) {
        markdown_body = _markdown_body;
        metadata = Some(parse_metadata(yaml_block)?);
    }

    let mut output = match format {
        Format::Html => markdown_to_html(&markdown_body, &options),
        Format::CommonMark => markdown_to_commonmark(&markdown_body, &options),
        Format::Xml => markdown_to_commonmark_xml(&markdown_body, &options),
    };
    output = output.trim().to_string();

    if let Some(mut metadata) = metadata {
        if let Some(template) = template {
            metadata.insert("body".to_string(), output);
            output = template.clone();

            metadata.into_iter().for_each(|(key, value)| {
                output = output.replace(&format!("${key}$"), &value);
            });
        }
    }

    Ok(output)
}
