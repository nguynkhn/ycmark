use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    combinator::{rest, verify},
    multi::many0,
    sequence::delimited,
};

use crate::metadata::Metadata;

const TEMPLATE_OPEN_SIGIL: &str = "$";
const TEMPLATE_CLOSE_SIGIL: &str = "$";
const TEMPLATE_ESCAPED_CHAR: &str = "$";

#[derive(Debug)]
pub enum TemplateNode<'a> {
    Literal(&'a str),
    Variable(&'a str),
}

fn parse_variable(input: &str) -> IResult<&str, TemplateNode> {
    let is_variable_char = |c: char| c.is_alphanumeric() || matches!(c, '_' | '-' | '.');

    delimited(
        tag(TEMPLATE_OPEN_SIGIL),
        take_while1(is_variable_char),
        tag(TEMPLATE_CLOSE_SIGIL),
    )
    .map(TemplateNode::Variable)
    .parse(input)
}

fn parse_literal(input: &str) -> IResult<&str, TemplateNode> {
    verify(alt((take_until(TEMPLATE_OPEN_SIGIL), rest)), |s: &str| {
        !s.is_empty()
    })
    .map(TemplateNode::Literal)
    .parse(input)
}

fn parse_template(input: &str) -> IResult<&str, Vec<TemplateNode>> {
    many0(alt((parse_variable, parse_literal))).parse(input)
}

pub fn apply_template(template: String, metadata: Metadata) -> String {
    match parse_template(&template) {
        Ok((_, nodes)) => nodes
            .into_iter()
            .map(|node| match node {
                TemplateNode::Literal(s) => s,
                TemplateNode::Variable("") => TEMPLATE_ESCAPED_CHAR,
                TemplateNode::Variable(s) => metadata.get(s).map_or(s, |v| v.as_str()),
            })
            .collect(),

        _ => template,
    }
}
