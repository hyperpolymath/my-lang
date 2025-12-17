//! Markdown processing module for My SSG
//!
//! Parses markdown with YAML frontmatter and converts to HTML.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct MarkdownDocument {
    /// Frontmatter metadata
    pub frontmatter: Frontmatter,
    /// Raw markdown content
    pub content: String,
    /// Rendered HTML content
    pub html: String,
}

#[derive(Debug, Clone, Default)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub date: Option<String>,
    pub template: Option<String>,
    pub tags: Vec<String>,
    pub draft: bool,
    pub summary: Option<String>,
    pub custom: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum MarkdownError {
    #[error("invalid frontmatter: {0}")]
    InvalidFrontmatter(String),
    #[error("parsing error: {0}")]
    ParseError(String),
}

/// Parse a markdown file with frontmatter
pub fn parse(content: &str) -> Result<MarkdownDocument, MarkdownError> {
    let (frontmatter, markdown) = extract_frontmatter(content)?;
    let html = render_markdown(&markdown);

    Ok(MarkdownDocument {
        frontmatter,
        content: markdown,
        html,
    })
}

/// Extract YAML frontmatter from content
fn extract_frontmatter(content: &str) -> Result<(Frontmatter, String), MarkdownError> {
    let content = content.trim();

    // Check for frontmatter delimiter
    if !content.starts_with("---") {
        return Ok((Frontmatter::default(), content.to_string()));
    }

    // Find the closing delimiter
    let rest = &content[3..];
    if let Some(end_idx) = rest.find("\n---") {
        let yaml_content = &rest[..end_idx].trim();
        let markdown = &rest[end_idx + 4..].trim();

        let frontmatter = parse_yaml_frontmatter(yaml_content)?;
        Ok((frontmatter, markdown.to_string()))
    } else {
        Ok((Frontmatter::default(), content.to_string()))
    }
}

/// Parse simple YAML-like frontmatter
fn parse_yaml_frontmatter(content: &str) -> Result<Frontmatter, MarkdownError> {
    let mut fm = Frontmatter::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            // Remove quotes if present
            let value = value.trim_matches('"').trim_matches('\'');

            match key {
                "title" => fm.title = Some(value.to_string()),
                "date" => fm.date = Some(value.to_string()),
                "template" => fm.template = Some(value.to_string()),
                "draft" => fm.draft = value == "true",
                "summary" => fm.summary = Some(value.to_string()),
                "tags" => {
                    // Parse [tag1, tag2] format
                    let tags_str = value.trim_matches(|c| c == '[' || c == ']');
                    fm.tags = tags_str
                        .split(',')
                        .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                _ => {
                    fm.custom.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    Ok(fm)
}

/// Render markdown to HTML
pub fn render_markdown(content: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_content = String::new();
    let mut in_list = false;
    let mut list_type = ListType::Unordered;

    for line in content.lines() {
        // Handle code blocks
        if line.starts_with("```") {
            if in_code_block {
                // End code block
                html.push_str(&format!(
                    "<pre><code class=\"language-{}\">{}</code></pre>\n",
                    if code_lang.is_empty() {
                        "text"
                    } else {
                        &code_lang
                    },
                    escape_html(&code_content.trim_end())
                ));
                code_content.clear();
                code_lang.clear();
                in_code_block = false;
            } else {
                // Start code block
                code_lang = line[3..].trim().to_string();
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            code_content.push_str(line);
            code_content.push('\n');
            continue;
        }

        // Handle lists
        let trimmed = line.trim();
        let is_ul_item = trimmed.starts_with("- ") || trimmed.starts_with("* ");
        let is_ol_item = trimmed
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .count()
            > 0
            && trimmed.contains(". ");

        if is_ul_item || is_ol_item {
            let new_type = if is_ul_item {
                ListType::Unordered
            } else {
                ListType::Ordered
            };

            if !in_list || list_type != new_type {
                if in_list {
                    html.push_str(match list_type {
                        ListType::Unordered => "</ul>\n",
                        ListType::Ordered => "</ol>\n",
                    });
                }
                html.push_str(match new_type {
                    ListType::Unordered => "<ul>\n",
                    ListType::Ordered => "<ol>\n",
                });
                list_type = new_type;
                in_list = true;
            }

            let item_content = if is_ul_item {
                &trimmed[2..]
            } else {
                trimmed.split_once(". ").map(|(_, s)| s).unwrap_or("")
            };
            html.push_str(&format!("<li>{}</li>\n", render_inline(item_content)));
            continue;
        } else if in_list {
            html.push_str(match list_type {
                ListType::Unordered => "</ul>\n",
                ListType::Ordered => "</ol>\n",
            });
            in_list = false;
        }

        // Handle headings
        if line.starts_with('#') {
            let level = line.chars().take_while(|c| *c == '#').count();
            let text = line[level..].trim();
            let id = slugify(text);
            html.push_str(&format!(
                "<h{} id=\"{}\">{}</h{}>\n",
                level,
                id,
                render_inline(text),
                level
            ));
            continue;
        }

        // Handle blockquotes
        if line.starts_with("> ") {
            let quote_text = &line[2..];
            html.push_str(&format!(
                "<blockquote>{}</blockquote>\n",
                render_inline(quote_text)
            ));
            continue;
        }

        // Handle horizontal rules
        if line.trim() == "---" || line.trim() == "***" || line.trim() == "___" {
            html.push_str("<hr>\n");
            continue;
        }

        // Handle empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Regular paragraph
        html.push_str(&format!("<p>{}</p>\n", render_inline(line)));
    }

    // Close any open list
    if in_list {
        html.push_str(match list_type {
            ListType::Unordered => "</ul>\n",
            ListType::Ordered => "</ol>\n",
        });
    }

    html
}

#[derive(PartialEq)]
enum ListType {
    Unordered,
    Ordered,
}

/// Render inline markdown (bold, italic, code, links)
fn render_inline(text: &str) -> String {
    let mut result = text.to_string();

    // Process inline code first (to avoid processing markdown inside code)
    result = render_inline_code(&result);

    // Bold: **text** or __text__
    result = render_pattern(&result, "**", "<strong>", "</strong>");
    result = render_pattern(&result, "__", "<strong>", "</strong>");

    // Italic: *text* or _text_
    result = render_pattern(&result, "*", "<em>", "</em>");
    result = render_pattern(&result, "_", "<em>", "</em>");

    // Links: [text](url)
    result = render_links(&result);

    // Images: ![alt](url)
    result = render_images(&result);

    result
}

fn render_inline_code(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    let mut in_code = false;
    let mut code_content = String::new();

    while let Some(c) = chars.next() {
        if c == '`' && !in_code {
            in_code = true;
            continue;
        } else if c == '`' && in_code {
            result.push_str(&format!("<code>{}</code>", escape_html(&code_content)));
            code_content.clear();
            in_code = false;
            continue;
        }

        if in_code {
            code_content.push(c);
        } else {
            result.push(c);
        }
    }

    // If we ended in a code block, just append the backtick and content
    if in_code {
        result.push('`');
        result.push_str(&code_content);
    }

    result
}

fn render_pattern(text: &str, marker: &str, open_tag: &str, close_tag: &str) -> String {
    let mut result = String::new();
    let mut remaining = text;

    while let Some(start_idx) = remaining.find(marker) {
        result.push_str(&remaining[..start_idx]);
        let after_start = &remaining[start_idx + marker.len()..];

        if let Some(end_idx) = after_start.find(marker) {
            let content = &after_start[..end_idx];
            result.push_str(open_tag);
            result.push_str(content);
            result.push_str(close_tag);
            remaining = &after_start[end_idx + marker.len()..];
        } else {
            result.push_str(marker);
            remaining = after_start;
        }
    }

    result.push_str(remaining);
    result
}

fn render_links(text: &str) -> String {
    let mut result = String::new();
    let mut remaining = text;

    while let Some(start) = remaining.find('[') {
        result.push_str(&remaining[..start]);
        let after_bracket = &remaining[start + 1..];

        if let Some(end_bracket) = after_bracket.find("](") {
            let link_text = &after_bracket[..end_bracket];
            let after_paren = &after_bracket[end_bracket + 2..];

            if let Some(close_paren) = after_paren.find(')') {
                let url = &after_paren[..close_paren];
                result.push_str(&format!(
                    "<a href=\"{}\">{}</a>",
                    escape_html(url),
                    link_text
                ));
                remaining = &after_paren[close_paren + 1..];
                continue;
            }
        }

        result.push('[');
        remaining = after_bracket;
    }

    result.push_str(remaining);
    result
}

fn render_images(text: &str) -> String {
    let mut result = String::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("![") {
        result.push_str(&remaining[..start]);
        let after_start = &remaining[start + 2..];

        if let Some(end_bracket) = after_start.find("](") {
            let alt_text = &after_start[..end_bracket];
            let after_paren = &after_start[end_bracket + 2..];

            if let Some(close_paren) = after_paren.find(')') {
                let url = &after_paren[..close_paren];
                result.push_str(&format!(
                    "<img src=\"{}\" alt=\"{}\">",
                    escape_html(url),
                    escape_html(alt_text)
                ));
                remaining = &after_paren[close_paren + 1..];
                continue;
            }
        }

        result.push_str("![");
        remaining = after_start;
    }

    result.push_str(remaining);
    result
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Convert text to URL-friendly slug
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: Test Post
date: 2025-01-01
tags: [rust, ssg]
---

# Hello

This is content.
"#;

        let doc = parse(content).unwrap();
        assert_eq!(doc.frontmatter.title, Some("Test Post".to_string()));
        assert_eq!(doc.frontmatter.date, Some("2025-01-01".to_string()));
        assert_eq!(doc.frontmatter.tags, vec!["rust", "ssg"]);
        assert!(doc.content.contains("# Hello"));
    }

    #[test]
    fn test_render_headings() {
        let md = "# Heading 1\n## Heading 2";
        let html = render_markdown(md);
        assert!(html.contains("<h1"));
        assert!(html.contains("<h2"));
    }

    #[test]
    fn test_render_emphasis() {
        let html = render_inline("This is **bold** and *italic*");
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn test_render_code() {
        let html = render_inline("Use `code` here");
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_render_links() {
        let html = render_inline("Check [this link](https://example.com)");
        assert!(html.contains("<a href=\"https://example.com\">this link</a>"));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("My--Cool  Title"), "my-cool-title");
    }
}
