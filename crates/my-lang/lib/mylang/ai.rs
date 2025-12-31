//! AI Integration Module
//!
//! Core AI model types, configuration, and interaction patterns
//! specific to My Language's first-class AI integration.

use std::collections::HashMap;

// ============================================================================
// AI Model Configuration
// ============================================================================

/// AI Model types supported by the language
#[derive(Debug, Clone, PartialEq)]
pub enum AiModelType {
    /// OpenAI GPT models
    OpenAI(String),
    /// Anthropic Claude models
    Anthropic(String),
    /// Local/custom models
    Local(String),
    /// Mock model for testing
    Mock,
}

impl AiModelType {
    pub fn from_str(s: &str) -> Self {
        if s.starts_with("gpt-") || s.starts_with("o1") {
            AiModelType::OpenAI(s.to_string())
        } else if s.starts_with("claude-") {
            AiModelType::Anthropic(s.to_string())
        } else if s == "mock" || s == "test" {
            AiModelType::Mock
        } else {
            AiModelType::Local(s.to_string())
        }
    }

    pub fn name(&self) -> &str {
        match self {
            AiModelType::OpenAI(name) => name,
            AiModelType::Anthropic(name) => name,
            AiModelType::Local(name) => name,
            AiModelType::Mock => "mock",
        }
    }
}

/// AI model configuration
#[derive(Debug, Clone)]
pub struct AiModelConfig {
    pub model_type: AiModelType,
    pub temperature: f64,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub stop_sequences: Vec<String>,
    pub system_prompt: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl Default for AiModelConfig {
    fn default() -> Self {
        AiModelConfig {
            model_type: AiModelType::Mock,
            temperature: 0.7,
            max_tokens: None,
            top_p: None,
            presence_penalty: None,
            frequency_penalty: None,
            stop_sequences: Vec::new(),
            system_prompt: None,
            api_key: None,
            base_url: None,
        }
    }
}

impl AiModelConfig {
    pub fn new(model: &str) -> Self {
        AiModelConfig {
            model_type: AiModelType::from_str(model),
            ..Default::default()
        }
    }

    pub fn with_temperature(mut self, temp: f64) -> Self {
        self.temperature = temp.clamp(0.0, 2.0);
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = Some(prompt.to_string());
        self
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }
}

// ============================================================================
// AI Response Types
// ============================================================================

/// AI completion response
#[derive(Debug, Clone)]
pub struct AiResponse {
    pub content: String,
    pub model: String,
    pub finish_reason: FinishReason,
    pub usage: Option<TokenUsage>,
    pub metadata: HashMap<String, String>,
}

/// Why the AI stopped generating
#[derive(Debug, Clone, PartialEq)]
pub enum FinishReason {
    Stop,
    MaxTokens,
    ContentFilter,
    ToolCall,
    Error(String),
}

/// Token usage statistics
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl AiResponse {
    pub fn mock(content: &str) -> Self {
        AiResponse {
            content: content.to_string(),
            model: "mock".to_string(),
            finish_reason: FinishReason::Stop,
            usage: Some(TokenUsage::default()),
            metadata: HashMap::new(),
        }
    }

    pub fn error(msg: &str) -> Self {
        AiResponse {
            content: String::new(),
            model: "error".to_string(),
            finish_reason: FinishReason::Error(msg.to_string()),
            usage: None,
            metadata: HashMap::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self.finish_reason, FinishReason::Stop | FinishReason::MaxTokens)
    }
}

// ============================================================================
// Message Types (for chat models)
// ============================================================================

/// Role in a conversation
#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl MessageRole {
    pub fn as_str(&self) -> &str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
        }
    }
}

/// A message in a conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Message {
            role: MessageRole::System,
            content: content.to_string(),
            name: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: &str) -> Self {
        Message {
            role: MessageRole::User,
            content: content.to_string(),
            name: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: &str) -> Self {
        Message {
            role: MessageRole::Assistant,
            content: content.to_string(),
            name: None,
            tool_call_id: None,
        }
    }

    pub fn tool(content: &str, tool_call_id: &str) -> Self {
        Message {
            role: MessageRole::Tool,
            content: content.to_string(),
            name: None,
            tool_call_id: Some(tool_call_id.to_string()),
        }
    }
}

// ============================================================================
// Conversation Management
// ============================================================================

/// A conversation with an AI model
#[derive(Debug, Clone)]
pub struct Conversation {
    pub messages: Vec<Message>,
    pub config: AiModelConfig,
}

impl Conversation {
    pub fn new(config: AiModelConfig) -> Self {
        let mut conv = Conversation {
            messages: Vec::new(),
            config,
        };
        if let Some(ref system) = conv.config.system_prompt {
            conv.messages.push(Message::system(system));
        }
        conv
    }

    pub fn add_system(&mut self, content: &str) {
        self.messages.push(Message::system(content));
    }

    pub fn add_user(&mut self, content: &str) {
        self.messages.push(Message::user(content));
    }

    pub fn add_assistant(&mut self, content: &str) {
        self.messages.push(Message::assistant(content));
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        if let Some(ref system) = self.config.system_prompt {
            self.messages.push(Message::system(system));
        }
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    pub fn to_messages_vec(&self) -> Vec<(String, String)> {
        self.messages
            .iter()
            .map(|m| (m.role.as_str().to_string(), m.content.clone()))
            .collect()
    }
}

// ============================================================================
// Mock AI Client (for testing)
// ============================================================================

/// Mock AI client for testing without real API calls
pub struct MockAiClient {
    responses: Vec<String>,
    response_index: usize,
}

impl MockAiClient {
    pub fn new() -> Self {
        MockAiClient {
            responses: vec!["This is a mock AI response.".to_string()],
            response_index: 0,
        }
    }

    pub fn with_responses(responses: Vec<String>) -> Self {
        MockAiClient {
            responses,
            response_index: 0,
        }
    }

    pub fn complete(&mut self, _prompt: &str) -> AiResponse {
        let content = if self.responses.is_empty() {
            "Mock response".to_string()
        } else {
            let response = self.responses[self.response_index % self.responses.len()].clone();
            self.response_index += 1;
            response
        };
        AiResponse::mock(&content)
    }

    pub fn chat(&mut self, _messages: &[Message]) -> AiResponse {
        self.complete("")
    }
}

impl Default for MockAiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_type_parsing() {
        assert!(matches!(
            AiModelType::from_str("gpt-4"),
            AiModelType::OpenAI(_)
        ));
        assert!(matches!(
            AiModelType::from_str("claude-3-opus"),
            AiModelType::Anthropic(_)
        ));
        assert!(matches!(
            AiModelType::from_str("mock"),
            AiModelType::Mock
        ));
        assert!(matches!(
            AiModelType::from_str("llama-7b"),
            AiModelType::Local(_)
        ));
    }

    #[test]
    fn test_conversation() {
        let config = AiModelConfig::new("mock")
            .with_system_prompt("You are a helpful assistant.");
        let mut conv = Conversation::new(config);

        assert_eq!(conv.message_count(), 1); // System message

        conv.add_user("Hello");
        conv.add_assistant("Hi there!");

        assert_eq!(conv.message_count(), 3);
        assert_eq!(conv.last_message().unwrap().content, "Hi there!");
    }

    #[test]
    fn test_mock_client() {
        let mut client = MockAiClient::with_responses(vec![
            "First".to_string(),
            "Second".to_string(),
        ]);

        let r1 = client.complete("test");
        let r2 = client.complete("test");
        let r3 = client.complete("test");

        assert_eq!(r1.content, "First");
        assert_eq!(r2.content, "Second");
        assert_eq!(r3.content, "First"); // Cycles back
    }
}
