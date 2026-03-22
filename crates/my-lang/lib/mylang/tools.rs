//! AI Tools Module
//!
//! Support for AI function/tool calling, enabling AI models to
//! invoke defined tools and integrate with My Language's AI blocks.

use std::collections::HashMap;

// ============================================================================
// Tool Definition Types
// ============================================================================

/// A tool parameter definition
#[derive(Debug, Clone)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: ToolParamType,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Types for tool parameters
#[derive(Debug, Clone, PartialEq)]
pub enum ToolParamType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<ToolParamType>),
    Object(HashMap<String, ToolParamType>),
    Enum(Vec<String>),
    Any,
}

impl ToolParamType {
    pub fn as_json_type(&self) -> &'static str {
        match self {
            ToolParamType::String => "string",
            ToolParamType::Integer => "integer",
            ToolParamType::Float => "number",
            ToolParamType::Boolean => "boolean",
            ToolParamType::Array(_) => "array",
            ToolParamType::Object(_) => "object",
            ToolParamType::Enum(_) => "string",
            ToolParamType::Any => "any",
        }
    }
}

impl ToolParameter {
    pub fn new(name: &str, param_type: ToolParamType) -> Self {
        ToolParameter {
            name: name.to_string(),
            description: String::new(),
            param_type,
            required: true,
            default_value: None,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_default(mut self, value: &str) -> Self {
        self.default_value = Some(value.to_string());
        self.required = false;
        self
    }
}

// ============================================================================
// Tool Definition
// ============================================================================

/// A tool that can be called by an AI model
#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

impl ToolDef {
    pub fn new(name: &str, description: &str) -> Self {
        ToolDef {
            name: name.to_string(),
            description: description.to_string(),
            parameters: Vec::new(),
        }
    }

    pub fn param(mut self, param: ToolParameter) -> Self {
        self.parameters.push(param);
        self
    }

    pub fn required_params(&self) -> Vec<&ToolParameter> {
        self.parameters.iter().filter(|p| p.required).collect()
    }

    pub fn optional_params(&self) -> Vec<&ToolParameter> {
        self.parameters.iter().filter(|p| !p.required).collect()
    }

    /// Generate JSON schema for this tool
    pub fn to_json_schema(&self) -> String {
        let mut params = Vec::new();
        let mut required = Vec::new();

        for param in &self.parameters {
            let param_json = format!(
                "\"{}\": {{\"type\": \"{}\", \"description\": \"{}\"}}",
                param.name,
                param.param_type.as_json_type(),
                param.description
            );
            params.push(param_json);

            if param.required {
                required.push(format!("\"{}\"", param.name));
            }
        }

        format!(
            r#"{{"name": "{}", "description": "{}", "parameters": {{"type": "object", "properties": {{{}}}, "required": [{}]}}}}"#,
            self.name,
            self.description,
            params.join(", "),
            required.join(", ")
        )
    }
}

// ============================================================================
// Tool Call Types
// ============================================================================

/// A request to call a tool
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: HashMap<String, ToolValue>,
}

/// A value passed to or returned from a tool
#[derive(Debug, Clone)]
pub enum ToolValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ToolValue>),
    Object(HashMap<String, ToolValue>),
    Null,
}

impl ToolValue {
    pub fn as_string(&self) -> Option<&str> {
        if let ToolValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let ToolValue::Integer(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ToolValue::Float(f) => Some(*f),
            ToolValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let ToolValue::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&Vec<ToolValue>> {
        if let ToolValue::Array(arr) = self {
            Some(arr)
        } else {
            None
        }
    }
}

impl ToolCall {
    pub fn new(id: &str, name: &str) -> Self {
        ToolCall {
            id: id.to_string(),
            name: name.to_string(),
            arguments: HashMap::new(),
        }
    }

    pub fn arg(mut self, name: &str, value: ToolValue) -> Self {
        self.arguments.insert(name.to_string(), value);
        self
    }

    pub fn get(&self, name: &str) -> Option<&ToolValue> {
        self.arguments.get(name)
    }

    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(|v| v.as_string())
    }

    pub fn get_integer(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(|v| v.as_integer())
    }

    pub fn get_float(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(|v| v.as_float())
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name).and_then(|v| v.as_bool())
    }
}

/// Result of a tool call
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub call_id: String,
    pub success: bool,
    pub output: ToolValue,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(call_id: &str, output: ToolValue) -> Self {
        ToolResult {
            call_id: call_id.to_string(),
            success: true,
            output,
            error: None,
        }
    }

    pub fn error(call_id: &str, error: &str) -> Self {
        ToolResult {
            call_id: call_id.to_string(),
            success: false,
            output: ToolValue::Null,
            error: Some(error.to_string()),
        }
    }
}

// ============================================================================
// Tool Registry
// ============================================================================

/// Type alias for tool handler functions
pub type ToolHandler = Box<dyn Fn(&ToolCall) -> ToolResult + Send + Sync>;

/// Registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, ToolDef>,
    handlers: HashMap<String, ToolHandler>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    /// Register a tool with its handler
    pub fn register<F>(&mut self, def: ToolDef, handler: F)
    where
        F: Fn(&ToolCall) -> ToolResult + Send + Sync + 'static,
    {
        let name = def.name.clone();
        self.tools.insert(name.clone(), def);
        self.handlers.insert(name, Box::new(handler));
    }

    /// Register just the tool definition (no handler)
    pub fn register_def(&mut self, def: ToolDef) {
        self.tools.insert(def.name.clone(), def);
    }

    /// Get a tool definition
    pub fn get(&self, name: &str) -> Option<&ToolDef> {
        self.tools.get(name)
    }

    /// Check if a tool exists
    pub fn has(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// List all tool names
    pub fn list(&self) -> Vec<&String> {
        self.tools.keys().collect()
    }

    /// Execute a tool call
    pub fn execute(&self, call: &ToolCall) -> ToolResult {
        if let Some(handler) = self.handlers.get(&call.name) {
            handler(call)
        } else {
            ToolResult::error(&call.id, &format!("Unknown tool: {}", call.name))
        }
    }

    /// Get all tool definitions as JSON schema
    pub fn to_json_schema(&self) -> String {
        let schemas: Vec<String> = self.tools.values().map(|t| t.to_json_schema()).collect();
        format!("[{}]", schemas.join(", "))
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Built-in Tools
// ============================================================================

/// Create common built-in tools
pub fn create_builtin_tools() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // Calculator tool
    let calc_tool = ToolDef::new("calculator", "Evaluate a mathematical expression")
        .param(
            ToolParameter::new("expression", ToolParamType::String)
                .with_description("Mathematical expression to evaluate"),
        );

    registry.register(calc_tool, |call| {
        if let Some(expr) = call.get_string("expression") {
            // Simple evaluation (in real impl, would parse and evaluate)
            ToolResult::success(&call.id, ToolValue::String(format!("Result of: {}", expr)))
        } else {
            ToolResult::error(&call.id, "Missing expression argument")
        }
    });

    // Get current time tool
    let time_tool = ToolDef::new("get_time", "Get the current date and time");

    registry.register(time_tool, |call| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        ToolResult::success(&call.id, ToolValue::Integer(now as i64))
    });

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_def() {
        let tool = ToolDef::new("greet", "Greet a person")
            .param(
                ToolParameter::new("name", ToolParamType::String)
                    .with_description("Name of the person"),
            )
            .param(
                ToolParameter::new("formal", ToolParamType::Boolean)
                    .optional()
                    .with_default("false"),
            );

        assert_eq!(tool.name, "greet");
        assert_eq!(tool.required_params().len(), 1);
        assert_eq!(tool.optional_params().len(), 1);
    }

    #[test]
    fn test_tool_call() {
        let call = ToolCall::new("call-1", "greet")
            .arg("name", ToolValue::String("Alice".to_string()))
            .arg("formal", ToolValue::Boolean(true));

        assert_eq!(call.get_string("name"), Some("Alice"));
        assert_eq!(call.get_bool("formal"), Some(true));
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();

        let tool = ToolDef::new("echo", "Echo the input")
            .param(ToolParameter::new("message", ToolParamType::String));

        registry.register(tool, |call| {
            let msg = call.get_string("message").unwrap_or("(empty)");
            ToolResult::success(&call.id, ToolValue::String(msg.to_string()))
        });

        let call = ToolCall::new("call-1", "echo")
            .arg("message", ToolValue::String("Hello!".to_string()));

        let result = registry.execute(&call);
        assert!(result.success);
    }

    #[test]
    fn test_builtin_tools() {
        let registry = create_builtin_tools();
        assert!(registry.has("calculator"));
        assert!(registry.has("get_time"));
    }
}
