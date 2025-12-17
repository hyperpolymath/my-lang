//! Prompt Building Module
//!
//! Utilities for constructing, templating, and managing AI prompts
//! specific to My Language's prompt blocks and AI integration.

use std::collections::HashMap;

// ============================================================================
// Prompt Templates
// ============================================================================

/// A prompt template with variable substitution
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    template: String,
    variables: HashMap<String, String>,
    default_values: HashMap<String, String>,
}

impl PromptTemplate {
    /// Create a new prompt template
    pub fn new(template: &str) -> Self {
        PromptTemplate {
            template: template.to_string(),
            variables: HashMap::new(),
            default_values: HashMap::new(),
        }
    }

    /// Set a variable value
    pub fn set(&mut self, name: &str, value: &str) -> &mut Self {
        self.variables.insert(name.to_string(), value.to_string());
        self
    }

    /// Set a default value for a variable
    pub fn set_default(&mut self, name: &str, value: &str) -> &mut Self {
        self.default_values.insert(name.to_string(), value.to_string());
        self
    }

    /// Set multiple variables at once
    pub fn set_all(&mut self, vars: HashMap<String, String>) -> &mut Self {
        self.variables.extend(vars);
        self
    }

    /// Render the template with current variables
    pub fn render(&self) -> String {
        let mut result = self.template.clone();

        // Replace {{variable}} patterns
        for (name, value) in &self.variables {
            let pattern = format!("{{{{{}}}}}", name);
            result = result.replace(&pattern, value);
        }

        // Replace remaining with defaults
        for (name, value) in &self.default_values {
            let pattern = format!("{{{{{}}}}}", name);
            result = result.replace(&pattern, value);
        }

        result
    }

    /// Get list of variables in template
    pub fn variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        let mut chars = self.template.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' && chars.peek() == Some(&'{') {
                chars.next(); // consume second {
                let mut var_name = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '}' {
                        chars.next();
                        if chars.peek() == Some(&'}') {
                            chars.next();
                            if !var_name.is_empty() {
                                vars.push(var_name);
                            }
                        }
                        break;
                    }
                    var_name.push(c);
                    chars.next();
                }
            }
        }

        vars
    }

    /// Check if all required variables are set
    pub fn is_complete(&self) -> bool {
        for var in self.variables() {
            if !self.variables.contains_key(&var) && !self.default_values.contains_key(&var) {
                return false;
            }
        }
        true
    }

    /// Get missing variables
    pub fn missing_variables(&self) -> Vec<String> {
        self.variables()
            .into_iter()
            .filter(|v| !self.variables.contains_key(v) && !self.default_values.contains_key(v))
            .collect()
    }
}

// ============================================================================
// Prompt Builder (Fluent API)
// ============================================================================

/// Fluent prompt builder
#[derive(Debug, Clone, Default)]
pub struct PromptBuilder {
    parts: Vec<String>,
    prefix: Option<String>,
    suffix: Option<String>,
    separator: String,
}

impl PromptBuilder {
    pub fn new() -> Self {
        PromptBuilder {
            parts: Vec::new(),
            prefix: None,
            suffix: None,
            separator: "\n".to_string(),
        }
    }

    /// Add a part to the prompt
    pub fn add(mut self, text: &str) -> Self {
        self.parts.push(text.to_string());
        self
    }

    /// Add a part if condition is true
    pub fn add_if(mut self, condition: bool, text: &str) -> Self {
        if condition {
            self.parts.push(text.to_string());
        }
        self
    }

    /// Add a formatted part
    pub fn add_fmt(mut self, format: &str, args: &[&str]) -> Self {
        let mut text = format.to_string();
        for (i, arg) in args.iter().enumerate() {
            text = text.replace(&format!("{{{}}}", i), arg);
        }
        self.parts.push(text);
        self
    }

    /// Set prefix
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    /// Set suffix
    pub fn suffix(mut self, suffix: &str) -> Self {
        self.suffix = Some(suffix.to_string());
        self
    }

    /// Set separator between parts
    pub fn separator(mut self, sep: &str) -> Self {
        self.separator = sep.to_string();
        self
    }

    /// Build the final prompt
    pub fn build(self) -> String {
        let mut result = String::new();

        if let Some(prefix) = self.prefix {
            result.push_str(&prefix);
            result.push_str(&self.separator);
        }

        result.push_str(&self.parts.join(&self.separator));

        if let Some(suffix) = self.suffix {
            result.push_str(&self.separator);
            result.push_str(&suffix);
        }

        result
    }
}

// ============================================================================
// Prompt Library (for reusable prompts)
// ============================================================================

/// A library of reusable prompts
#[derive(Debug, Clone, Default)]
pub struct PromptLibrary {
    prompts: HashMap<String, PromptTemplate>,
}

impl PromptLibrary {
    pub fn new() -> Self {
        PromptLibrary {
            prompts: HashMap::new(),
        }
    }

    /// Register a prompt template
    pub fn register(&mut self, name: &str, template: &str) {
        self.prompts.insert(name.to_string(), PromptTemplate::new(template));
    }

    /// Get a prompt template by name
    pub fn get(&self, name: &str) -> Option<&PromptTemplate> {
        self.prompts.get(name)
    }

    /// Get a mutable prompt template by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut PromptTemplate> {
        self.prompts.get_mut(name)
    }

    /// Check if a prompt exists
    pub fn has(&self, name: &str) -> bool {
        self.prompts.contains_key(name)
    }

    /// List all prompt names
    pub fn list(&self) -> Vec<&String> {
        self.prompts.keys().collect()
    }

    /// Render a prompt with given variables
    pub fn render(&self, name: &str, vars: HashMap<String, String>) -> Option<String> {
        self.prompts.get(name).map(|p| {
            let mut template = p.clone();
            template.set_all(vars);
            template.render()
        })
    }
}

// ============================================================================
// Common Prompt Patterns
// ============================================================================

/// Create a system prompt for role-based AI
pub fn role_prompt(role: &str, context: &str) -> String {
    format!("You are {}. {}", role, context)
}

/// Create a few-shot learning prompt
pub fn few_shot_prompt(examples: &[(String, String)], query: &str) -> String {
    let mut prompt = String::new();

    for (input, output) in examples {
        prompt.push_str(&format!("Input: {}\nOutput: {}\n\n", input, output));
    }

    prompt.push_str(&format!("Input: {}\nOutput:", query));
    prompt
}

/// Create a chain-of-thought prompt
pub fn chain_of_thought(task: &str) -> String {
    format!(
        "{}\n\nLet's think about this step by step:\n",
        task
    )
}

/// Create a structured output prompt
pub fn structured_output(task: &str, format: &str) -> String {
    format!(
        "{}\n\nRespond in the following format:\n{}",
        task, format
    )
}

/// Create a JSON output prompt
pub fn json_output(task: &str, schema: &str) -> String {
    format!(
        "{}\n\nRespond with valid JSON matching this schema:\n{}\n\nJSON Response:",
        task, schema
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_template() {
        let mut template = PromptTemplate::new("Hello, {{name}}! You are {{age}} years old.");
        template.set("name", "Alice");
        template.set("age", "30");

        assert_eq!(
            template.render(),
            "Hello, Alice! You are 30 years old."
        );
    }

    #[test]
    fn test_template_variables() {
        let template = PromptTemplate::new("{{greeting}}, {{name}}!");
        let vars = template.variables();

        assert!(vars.contains(&"greeting".to_string()));
        assert!(vars.contains(&"name".to_string()));
    }

    #[test]
    fn test_template_defaults() {
        let mut template = PromptTemplate::new("{{greeting}}, {{name}}!");
        template.set_default("greeting", "Hello");
        template.set("name", "World");

        assert_eq!(template.render(), "Hello, World!");
    }

    #[test]
    fn test_prompt_builder() {
        let prompt = PromptBuilder::new()
            .prefix("Instructions:")
            .add("Step 1: Do this")
            .add("Step 2: Do that")
            .suffix("Good luck!")
            .build();

        assert!(prompt.contains("Instructions:"));
        assert!(prompt.contains("Step 1"));
        assert!(prompt.contains("Step 2"));
        assert!(prompt.contains("Good luck!"));
    }

    #[test]
    fn test_prompt_library() {
        let mut lib = PromptLibrary::new();
        lib.register("greeting", "Hello, {{name}}!");

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        let result = lib.render("greeting", vars);
        assert_eq!(result, Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_few_shot() {
        let examples = vec![
            ("2 + 2".to_string(), "4".to_string()),
            ("3 + 3".to_string(), "6".to_string()),
        ];

        let prompt = few_shot_prompt(&examples, "5 + 5");
        assert!(prompt.contains("Input: 2 + 2"));
        assert!(prompt.contains("Output: 4"));
        assert!(prompt.contains("Input: 5 + 5"));
    }
}
