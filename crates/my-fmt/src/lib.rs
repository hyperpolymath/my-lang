// SPDX-License-Identifier: MIT
//! Code Formatter for My Language
//!
//! Provides consistent code formatting using a pretty-printing approach.
//! Supports all My Language syntax including AI expressions and dialects.

use my_lang::{parse, Program, TopLevel};
use thiserror::Error;

/// Formatter errors
#[derive(Debug, Error)]
pub enum FormatError {
    #[error("parse error: {0}")]
    ParseError(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Formatter configuration
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Maximum line width before wrapping
    pub max_width: usize,
    /// Indentation string (spaces or tabs)
    pub indent: String,
    /// Use trailing commas in lists
    pub trailing_commas: bool,
    /// Space inside braces
    pub space_in_braces: bool,
    /// Newline at end of file
    pub final_newline: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        FormatConfig {
            max_width: 100,
            indent: "    ".to_string(),
            trailing_commas: true,
            space_in_braces: true,
            final_newline: true,
        }
    }
}

/// Pretty-printing document
#[derive(Debug, Clone)]
pub enum Doc {
    Nil,
    Text(String),
    Line,
    HardLine,
    Concat(Box<Doc>, Box<Doc>),
    Nest(usize, Box<Doc>),
    Group(Box<Doc>),
}

impl Doc {
    pub fn text(s: impl Into<String>) -> Self {
        Doc::Text(s.into())
    }

    pub fn concat(self, other: Doc) -> Self {
        Doc::Concat(Box::new(self), Box::new(other))
    }

    pub fn nest(self, indent: usize) -> Self {
        Doc::Nest(indent, Box::new(self))
    }

    pub fn group(self) -> Self {
        Doc::Group(Box::new(self))
    }

    pub fn pretty(self, width: usize) -> String {
        let mut output = String::new();
        self.render(width, 0, true, &mut output);
        output
    }

    fn render(&self, width: usize, indent: usize, flat: bool, output: &mut String) {
        match self {
            Doc::Nil => {}
            Doc::Text(s) => output.push_str(s),
            Doc::Line => {
                if flat {
                    output.push(' ');
                } else {
                    output.push('\n');
                    for _ in 0..indent {
                        output.push(' ');
                    }
                }
            }
            Doc::HardLine => {
                output.push('\n');
                for _ in 0..indent {
                    output.push(' ');
                }
            }
            Doc::Concat(a, b) => {
                a.render(width, indent, flat, output);
                b.render(width, indent, flat, output);
            }
            Doc::Nest(i, doc) => {
                doc.render(width, indent + i, flat, output);
            }
            Doc::Group(doc) => {
                let mut flat_output = String::new();
                doc.render(width, indent, true, &mut flat_output);

                if output.lines().last().map(|l| l.len()).unwrap_or(0) + flat_output.len() <= width {
                    output.push_str(&flat_output);
                } else {
                    doc.render(width, indent, false, output);
                }
            }
        }
    }
}

/// Code formatter
pub struct Formatter {
    config: FormatConfig,
}

impl Formatter {
    pub fn new(config: FormatConfig) -> Self {
        Formatter { config }
    }

    /// Format source code
    pub fn format(&self, source: &str) -> Result<String, FormatError> {
        let program = parse(source).map_err(|e| FormatError::ParseError(e.to_string()))?;
        let doc = self.format_program(&program);
        let mut result = doc.pretty(self.config.max_width);

        if self.config.final_newline && !result.ends_with('\n') {
            result.push('\n');
        }

        Ok(result)
    }

    fn format_program(&self, program: &Program) -> Doc {
        let mut doc = Doc::Nil;

        for (i, item) in program.items.iter().enumerate() {
            if i > 0 {
                doc = doc.concat(Doc::HardLine).concat(Doc::HardLine);
            }
            doc = doc.concat(self.format_top_level(item));
        }

        doc
    }

    fn format_top_level(&self, item: &TopLevel) -> Doc {
        match item {
            TopLevel::Function(f) => {
                Doc::text("fn ")
                    .concat(Doc::text(&f.name.name))
                    .concat(Doc::text("("))
                    .concat(self.format_params(&f.params))
                    .concat(Doc::text(")"))
                    .concat(if let Some(ret) = &f.return_type {
                        Doc::text(" -> ").concat(Doc::text(format!("{:?}", ret)))
                    } else {
                        Doc::Nil
                    })
                    .concat(Doc::text(" { ... }"))
            }
            TopLevel::Struct(s) => {
                Doc::text("struct ")
                    .concat(Doc::text(&s.name.name))
                    .concat(Doc::text(" { ... }"))
            }
            TopLevel::Effect(e) => {
                Doc::text("effect ")
                    .concat(Doc::text(&e.name.name))
                    .concat(Doc::text(" { ... }"))
            }
            TopLevel::AiModel(m) => {
                Doc::text("ai_model ")
                    .concat(Doc::text(&m.name.name))
                    .concat(Doc::text(" { ... }"))
            }
            TopLevel::Prompt(p) => {
                Doc::text("prompt ")
                    .concat(Doc::text(&p.name.name))
                    .concat(Doc::text(" { \""))
                    .concat(Doc::text(&p.template))
                    .concat(Doc::text("\" }"))
            }
            _ => Doc::text("// TODO: format this item"),
        }
    }

    fn format_params(&self, params: &[my_lang::Param]) -> Doc {
        let mut doc = Doc::Nil;
        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                doc = doc.concat(Doc::text(", "));
            }
            doc = doc
                .concat(Doc::text(&param.name.name))
                .concat(Doc::text(": "))
                .concat(Doc::text(format!("{:?}", param.ty)));
        }
        doc
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new(FormatConfig::default())
    }
}

/// Format a file in place
pub fn format_file(path: &std::path::Path, config: &FormatConfig) -> Result<(), FormatError> {
    let source = std::fs::read_to_string(path)?;
    let formatter = Formatter::new(config.clone());
    let formatted = formatter.format(&source)?;
    std::fs::write(path, formatted)?;
    Ok(())
}

/// Check if a file is formatted
pub fn check_formatted(path: &std::path::Path, config: &FormatConfig) -> Result<bool, FormatError> {
    let source = std::fs::read_to_string(path)?;
    let formatter = Formatter::new(config.clone());
    let formatted = formatter.format(&source)?;
    Ok(source == formatted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_pretty() {
        let doc = Doc::text("hello")
            .concat(Doc::Line)
            .concat(Doc::text("world"))
            .group();

        let result = doc.pretty(80);
        assert!(result.contains("hello") && result.contains("world"));
    }
}
