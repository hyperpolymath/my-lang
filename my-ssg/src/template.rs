//! Template engine for My SSG
//!
//! Uses My Language for template logic and expression evaluation.

use my_lang::{Interpreter, Value};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("template not found: {0}")]
    NotFound(String),
    #[error("syntax error: {0}")]
    SyntaxError(String),
    #[error("evaluation error: {0}")]
    EvalError(String),
}

/// Template context containing variables for rendering
#[derive(Debug, Clone)]
pub struct Context {
    pub variables: HashMap<String, ContextValue>,
}

#[derive(Debug, Clone)]
pub enum ContextValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<ContextValue>),
    Object(HashMap<String, ContextValue>),
}

impl Context {
    pub fn new() -> Self {
        Context {
            variables: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: ContextValue) {
        self.variables.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&ContextValue> {
        // Handle dot notation (e.g., "page.title")
        let parts: Vec<&str> = key.split('.').collect();

        if parts.is_empty() {
            return None;
        }

        let mut current = self.variables.get(parts[0])?;

        for part in &parts[1..] {
            match current {
                ContextValue::Object(map) => {
                    current = map.get(*part)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextValue {
    pub fn to_string_value(&self) -> String {
        match self {
            ContextValue::String(s) => s.clone(),
            ContextValue::Int(n) => n.to_string(),
            ContextValue::Float(f) => f.to_string(),
            ContextValue::Bool(b) => b.to_string(),
            ContextValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string_value()).collect();
                format!("[{}]", items.join(", "))
            }
            ContextValue::Object(map) => {
                let items: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string_value()))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
        }
    }

    pub fn to_mylang_value(&self) -> Value {
        match self {
            ContextValue::String(s) => Value::String(s.clone()),
            ContextValue::Int(n) => Value::Int(*n),
            ContextValue::Float(f) => Value::Float(*f),
            ContextValue::Bool(b) => Value::Bool(*b),
            ContextValue::Array(arr) => {
                Value::Array(arr.iter().map(|v| v.to_mylang_value()).collect())
            }
            ContextValue::Object(map) => {
                let mut hm = std::collections::HashMap::new();
                for (k, v) in map {
                    hm.insert(k.clone(), v.to_mylang_value());
                }
                Value::Record(hm)
            }
        }
    }
}

/// Render a template with the given context
pub fn render(template: &str, context: &Context) -> Result<String, TemplateError> {
    let mut result = String::new();
    let mut remaining = template;

    while !remaining.is_empty() {
        // Look for template tags
        if let Some(start) = remaining.find("{{") {
            // Add text before the tag
            result.push_str(&remaining[..start]);
            let after_start = &remaining[start + 2..];

            // Find closing tag
            if let Some(end) = after_start.find("}}") {
                let expression = after_start[..end].trim();
                let rendered = evaluate_expression(expression, context)?;
                result.push_str(&rendered);
                remaining = &after_start[end + 2..];
            } else {
                return Err(TemplateError::SyntaxError(
                    "unclosed {{ tag".to_string(),
                ));
            }
        } else if let Some(start) = remaining.find("{%") {
            // Control flow tag
            result.push_str(&remaining[..start]);
            let after_start = &remaining[start + 2..];

            if let Some(end) = after_start.find("%}") {
                let tag_content = after_start[..end].trim();
                let (output, new_remaining) =
                    process_control_tag(tag_content, &after_start[end + 2..], context)?;
                result.push_str(&output);
                remaining = new_remaining;
            } else {
                return Err(TemplateError::SyntaxError(
                    "unclosed {% tag".to_string(),
                ));
            }
        } else {
            // No more tags, add remaining text
            result.push_str(remaining);
            break;
        }
    }

    Ok(result)
}

/// Evaluate a simple expression
fn evaluate_expression(expr: &str, context: &Context) -> Result<String, TemplateError> {
    let expr = expr.trim();

    // Handle filters (e.g., "value | uppercase")
    if let Some((value_expr, filter)) = expr.split_once('|') {
        let value = evaluate_expression(value_expr.trim(), context)?;
        return apply_filter(&value, filter.trim());
    }

    // Handle direct variable lookup
    if let Some(value) = context.get(expr) {
        return Ok(value.to_string_value());
    }

    // Handle string literals
    if (expr.starts_with('"') && expr.ends_with('"'))
        || (expr.starts_with('\'') && expr.ends_with('\''))
    {
        return Ok(expr[1..expr.len() - 1].to_string());
    }

    // Handle numeric literals
    if let Ok(n) = expr.parse::<i64>() {
        return Ok(n.to_string());
    }
    if let Ok(f) = expr.parse::<f64>() {
        return Ok(f.to_string());
    }

    // Handle boolean literals
    if expr == "true" {
        return Ok("true".to_string());
    }
    if expr == "false" {
        return Ok("false".to_string());
    }

    // Try to evaluate using My Language interpreter
    let source = format!("fn __eval__() {{ return {}; }}", expr);
    if let Ok(program) = my_lang::parse(&source) {
        let mut interpreter = Interpreter::new();

        // Inject context variables into interpreter
        for (key, value) in &context.variables {
            interpreter
                .env
                .borrow_mut()
                .define(key.clone(), value.to_mylang_value());
        }

        if let Ok(result) = interpreter.run(&program) {
            return Ok(format!("{}", result));
        }
    }

    // Unknown variable - return empty or the expression itself
    Ok(String::new())
}

/// Apply a filter to a value
fn apply_filter(value: &str, filter: &str) -> Result<String, TemplateError> {
    let (filter_name, args) = if let Some((name, args_str)) = filter.split_once(':') {
        (name.trim(), Some(args_str.trim()))
    } else {
        (filter, None)
    };

    match filter_name {
        "uppercase" | "upper" => Ok(value.to_uppercase()),
        "lowercase" | "lower" => Ok(value.to_lowercase()),
        "capitalize" => {
            let mut chars = value.chars();
            match chars.next() {
                None => Ok(String::new()),
                Some(c) => Ok(c.to_uppercase().collect::<String>() + chars.as_str()),
            }
        }
        "trim" => Ok(value.trim().to_string()),
        "length" | "len" => Ok(value.len().to_string()),
        "escape" | "e" => Ok(escape_html(value)),
        "default" => {
            if value.is_empty() {
                Ok(args.unwrap_or("").trim_matches('"').to_string())
            } else {
                Ok(value.to_string())
            }
        }
        "truncate" => {
            let len: usize = args
                .and_then(|a| a.parse().ok())
                .unwrap_or(100);
            if value.len() > len {
                Ok(format!("{}...", &value[..len]))
            } else {
                Ok(value.to_string())
            }
        }
        "date" => {
            // Simple date formatting (would need more implementation)
            Ok(value.to_string())
        }
        _ => Err(TemplateError::SyntaxError(format!(
            "unknown filter: {}",
            filter_name
        ))),
    }
}

/// Process control flow tags (if, for, extends, etc.)
fn process_control_tag<'a>(
    tag_content: &str,
    remaining: &'a str,
    context: &Context,
) -> Result<(String, &'a str), TemplateError> {
    let parts: Vec<&str> = tag_content.split_whitespace().collect();

    if parts.is_empty() {
        return Ok((String::new(), remaining));
    }

    match parts[0] {
        "if" => process_if_tag(&parts[1..], remaining, context),
        "for" => process_for_tag(&parts[1..], remaining, context),
        "extends" => {
            // For extends, we just note it - actual inheritance handled elsewhere
            Ok((String::new(), remaining))
        }
        "include" => process_include_tag(&parts[1..], context),
        _ => Ok((String::new(), remaining)),
    }
}

/// Process if tag
fn process_if_tag<'a>(
    condition_parts: &[&str],
    remaining: &'a str,
    context: &Context,
) -> Result<(String, &'a str), TemplateError> {
    let condition = condition_parts.join(" ");
    let condition_result = evaluate_condition(&condition, context);

    // Find endif
    let endif_pattern = "{% endif %}";
    let else_pattern = "{% else %}";

    // Look for else or endif
    let mut depth = 1;
    let mut pos = 0;
    let mut else_pos = None;
    let mut endif_pos = None;
    let chars: Vec<char> = remaining.chars().collect();

    while pos < chars.len() {
        if remaining[pos..].starts_with("{% if") {
            depth += 1;
        } else if remaining[pos..].starts_with(endif_pattern) {
            depth -= 1;
            if depth == 0 {
                endif_pos = Some(pos);
                break;
            }
        } else if depth == 1 && remaining[pos..].starts_with(else_pattern) {
            else_pos = Some(pos);
        }
        pos += 1;
    }

    let endif_pos = endif_pos.ok_or_else(|| {
        TemplateError::SyntaxError("missing {% endif %}".to_string())
    })?;

    let (then_content, else_content) = if let Some(else_p) = else_pos {
        (
            &remaining[..else_p],
            &remaining[else_p + else_pattern.len()..endif_pos],
        )
    } else {
        (&remaining[..endif_pos], "")
    };

    let output = if condition_result {
        render(then_content, context)?
    } else {
        render(else_content, context)?
    };

    Ok((output, &remaining[endif_pos + endif_pattern.len()..]))
}

/// Process for loop tag
fn process_for_tag<'a>(
    parts: &[&str],
    remaining: &'a str,
    context: &Context,
) -> Result<(String, &'a str), TemplateError> {
    // Parse: for item in collection [limit:n]
    if parts.len() < 3 || parts[1] != "in" {
        return Err(TemplateError::SyntaxError(
            "invalid for loop syntax".to_string(),
        ));
    }

    let item_name = parts[0];
    let collection_name = parts[2];
    let limit: Option<usize> = parts.get(3).and_then(|s| {
        s.strip_prefix("limit:")
            .and_then(|n| n.parse().ok())
    });

    // Find endfor
    let endfor_pattern = "{% endfor %}";
    let mut depth = 1;
    let mut pos = 0;
    let mut endfor_pos = None;

    while pos < remaining.len() {
        if remaining[pos..].starts_with("{% for") {
            depth += 1;
        } else if remaining[pos..].starts_with(endfor_pattern) {
            depth -= 1;
            if depth == 0 {
                endfor_pos = Some(pos);
                break;
            }
        }
        pos += 1;
    }

    let endfor_pos = endfor_pos.ok_or_else(|| {
        TemplateError::SyntaxError("missing {% endfor %}".to_string())
    })?;

    let loop_body = &remaining[..endfor_pos];

    // Get collection from context
    let collection = context.get(collection_name);
    let mut output = String::new();

    if let Some(ContextValue::Array(items)) = collection {
        let items_to_process: Box<dyn Iterator<Item = &ContextValue>> =
            if let Some(lim) = limit {
                Box::new(items.iter().take(lim))
            } else {
                Box::new(items.iter())
            };

        for (index, item) in items_to_process.enumerate() {
            let mut loop_context = context.clone();
            loop_context.insert(item_name, item.clone());

            // Add loop variables
            let mut loop_vars = HashMap::new();
            loop_vars.insert("index".to_string(), ContextValue::Int(index as i64));
            loop_vars.insert("first".to_string(), ContextValue::Bool(index == 0));
            loop_vars.insert("last".to_string(), ContextValue::Bool(index == items.len() - 1));
            loop_context.insert("loop", ContextValue::Object(loop_vars));

            output.push_str(&render(loop_body, &loop_context)?);
        }
    }

    Ok((output, &remaining[endfor_pos + endfor_pattern.len()..]))
}

/// Process include tag
fn process_include_tag(
    parts: &[&str],
    _context: &Context,
) -> Result<(String, &'static str), TemplateError> {
    if parts.is_empty() {
        return Err(TemplateError::SyntaxError(
            "include requires a template name".to_string(),
        ));
    }

    // Include handling would read and render another template
    // For now, return a placeholder
    Ok((format!("<!-- include: {} -->", parts[0].trim_matches('"')), ""))
}

/// Evaluate a condition expression
fn evaluate_condition(condition: &str, context: &Context) -> bool {
    let condition = condition.trim();

    // Handle negation
    if condition.starts_with("not ") || condition.starts_with("!") {
        let inner = if condition.starts_with("not ") {
            &condition[4..]
        } else {
            &condition[1..]
        };
        return !evaluate_condition(inner, context);
    }

    // Handle 'and' operator
    if let Some((left, right)) = condition.split_once(" and ") {
        return evaluate_condition(left, context) && evaluate_condition(right, context);
    }

    // Handle 'or' operator
    if let Some((left, right)) = condition.split_once(" or ") {
        return evaluate_condition(left, context) || evaluate_condition(right, context);
    }

    // Handle comparison operators
    for op in &["==", "!=", ">=", "<=", ">", "<"] {
        if let Some((left, right)) = condition.split_once(op) {
            let left_val = evaluate_expression(left.trim(), context).unwrap_or_default();
            let right_val = evaluate_expression(right.trim(), context).unwrap_or_default();

            return match *op {
                "==" => left_val == right_val,
                "!=" => left_val != right_val,
                ">=" => left_val >= right_val,
                "<=" => left_val <= right_val,
                ">" => left_val > right_val,
                "<" => left_val < right_val,
                _ => false,
            };
        }
    }

    // Simple truthiness check
    if let Some(value) = context.get(condition) {
        match value {
            ContextValue::Bool(b) => return *b,
            ContextValue::String(s) => return !s.is_empty(),
            ContextValue::Int(n) => return *n != 0,
            ContextValue::Array(arr) => return !arr.is_empty(),
            _ => return true,
        }
    }

    condition == "true"
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_variable() {
        let mut ctx = Context::new();
        ctx.insert("name", ContextValue::String("World".to_string()));

        let result = render("Hello, {{ name }}!", &ctx).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_nested_variable() {
        let mut ctx = Context::new();
        let mut page = HashMap::new();
        page.insert("title".to_string(), ContextValue::String("My Page".to_string()));
        ctx.insert("page", ContextValue::Object(page));

        let result = render("Title: {{ page.title }}", &ctx).unwrap();
        assert_eq!(result, "Title: My Page");
    }

    #[test]
    fn test_filter() {
        let mut ctx = Context::new();
        ctx.insert("name", ContextValue::String("world".to_string()));

        let result = render("{{ name | uppercase }}", &ctx).unwrap();
        assert_eq!(result, "WORLD");
    }

    #[test]
    fn test_if_condition() {
        let mut ctx = Context::new();
        ctx.insert("show", ContextValue::Bool(true));

        let template = "{% if show %}visible{% endif %}";
        let result = render(template, &ctx).unwrap();
        assert_eq!(result, "visible");
    }

    #[test]
    fn test_for_loop() {
        let mut ctx = Context::new();
        ctx.insert(
            "items",
            ContextValue::Array(vec![
                ContextValue::String("a".to_string()),
                ContextValue::String("b".to_string()),
            ]),
        );

        let template = "{% for item in items %}{{ item }}{% endfor %}";
        let result = render(template, &ctx).unwrap();
        assert_eq!(result, "ab");
    }
}
