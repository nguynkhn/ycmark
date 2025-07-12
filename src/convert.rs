use crate::metadata::{extract_yaml_block, parse_metadata, ParseError};
use comrak::{markdown_to_commonmark, markdown_to_commonmark_xml, markdown_to_html};

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
    pub fn to_comrak_options(&self) -> comrak::Options {
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
    let (metadata_str, markdown) = extract_yaml_block(input)
        .map(|(meta, body)| (Some(meta), body))
        .unwrap_or((None, input));

    let metadata = match metadata_str {
        Some(meta) => Some(parse_metadata(meta)?),
        None => None,
    };

    let comrak_opts = options.to_comrak_options();
    let mut output = match format {
        Format::Html => markdown_to_html(markdown, &comrak_opts),
        Format::CommonMark => markdown_to_commonmark(markdown, &comrak_opts),
        Format::Xml => markdown_to_commonmark_xml(markdown, &comrak_opts),
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
