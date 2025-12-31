// SPDX-License-Identifier: MIT
//! AI Runtime for My Language
//!
//! Provides AI model abstraction and execution:
//! - Multiple provider support (Anthropic, OpenAI, Ollama)
//! - Response caching with rocketcache
//! - Newtonian agent orchestration
//! - Streaming support

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// AI runtime errors
#[derive(Debug, Error)]
pub enum AIError {
    #[error("provider error: {0}")]
    ProviderError(String),

    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("rate limited")]
    RateLimited,

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("network error: {0}")]
    NetworkError(String),

    #[error("cache error: {0}")]
    CacheError(String),
}

/// Completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub system: Option<String>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Message role
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
}

/// Token usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
    pub model: String,
}

/// AI provider trait
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Complete a prompt
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError>;

    /// Generate embeddings
    async fn embed(&self, text: &str) -> Result<EmbeddingResponse, AIError>;

    /// Check if model is available
    fn supports_model(&self, model: &str) -> bool;
}

/// Anthropic provider
pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        AnthropicProvider {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": request.model,
                "messages": request.messages,
                "max_tokens": request.max_tokens.unwrap_or(1024),
                "temperature": request.temperature.unwrap_or(0.7),
            }))
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AIError::RateLimited);
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(e.to_string()))?;

        // Parse Anthropic response format
        let content = body["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(CompletionResponse {
            content,
            model: request.model,
            usage: Usage::default(),
        })
    }

    async fn embed(&self, _text: &str) -> Result<EmbeddingResponse, AIError> {
        // Anthropic doesn't have embedding API yet
        Err(AIError::ProviderError(
            "Anthropic does not support embeddings".to_string(),
        ))
    }

    fn supports_model(&self, model: &str) -> bool {
        model.starts_with("claude")
    }
}

/// OpenAI provider
pub struct OpenAIProvider {
    api_key: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        OpenAIProvider {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": request.model,
                "messages": request.messages,
                "max_tokens": request.max_tokens,
                "temperature": request.temperature,
            }))
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AIError::RateLimited);
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(e.to_string()))?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(CompletionResponse {
            content,
            model: request.model,
            usage: Usage::default(),
        })
    }

    async fn embed(&self, text: &str) -> Result<EmbeddingResponse, AIError> {
        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": "text-embedding-3-small",
                "input": text,
            }))
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(e.to_string()))?;

        let embedding: Vec<f32> = body["data"][0]["embedding"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
            .unwrap_or_default();

        Ok(EmbeddingResponse {
            embedding,
            model: "text-embedding-3-small".to_string(),
        })
    }

    fn supports_model(&self, model: &str) -> bool {
        model.starts_with("gpt") || model.starts_with("o1")
    }
}

/// Ollama provider (local)
pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: Option<String>) -> Self {
        OllamaProvider {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
        let prompt = request
            .messages
            .iter()
            .map(|m| format!("{}: {}", match m.role {
                Role::System => "System",
                Role::User => "User",
                Role::Assistant => "Assistant",
            }, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&serde_json::json!({
                "model": request.model,
                "prompt": prompt,
                "stream": false,
            }))
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(e.to_string()))?;

        let content = body["response"].as_str().unwrap_or("").to_string();

        Ok(CompletionResponse {
            content,
            model: request.model,
            usage: Usage::default(),
        })
    }

    async fn embed(&self, text: &str) -> Result<EmbeddingResponse, AIError> {
        let response = self
            .client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&serde_json::json!({
                "model": "nomic-embed-text",
                "prompt": text,
            }))
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(e.to_string()))?;

        let embedding: Vec<f32> = body["embedding"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
            .unwrap_or_default();

        Ok(EmbeddingResponse {
            embedding,
            model: "nomic-embed-text".to_string(),
        })
    }

    fn supports_model(&self, _model: &str) -> bool {
        true // Ollama can run any model
    }
}

/// AI cache for response deduplication
/// TODO: Replace with rocketcache integration
pub struct AICache {
    cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
}

#[derive(Debug, Clone)]
struct CachedResponse {
    response: CompletionResponse,
    timestamp: std::time::Instant,
}

impl AICache {
    pub fn new() -> Self {
        AICache {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<CompletionResponse> {
        let cache = self.cache.read().await;
        cache.get(key).map(|c| c.response.clone())
    }

    pub async fn set(&self, key: String, response: CompletionResponse) {
        let mut cache = self.cache.write().await;
        cache.insert(
            key,
            CachedResponse {
                response,
                timestamp: std::time::Instant::now(),
            },
        );
    }

    fn cache_key(request: &CompletionRequest) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        request.model.hash(&mut hasher);
        for msg in &request.messages {
            msg.content.hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }
}

impl Default for AICache {
    fn default() -> Self {
        Self::new()
    }
}

/// AI Runtime - main entry point
pub struct AIRuntime {
    providers: Vec<Box<dyn AIProvider>>,
    cache: AICache,
    default_model: String,
}

impl AIRuntime {
    pub fn new() -> Self {
        AIRuntime {
            providers: Vec::new(),
            cache: AICache::new(),
            default_model: "claude-3-opus".to_string(),
        }
    }

    pub fn with_anthropic(mut self, api_key: String) -> Self {
        self.providers.push(Box::new(AnthropicProvider::new(api_key)));
        self
    }

    pub fn with_openai(mut self, api_key: String) -> Self {
        self.providers.push(Box::new(OpenAIProvider::new(api_key)));
        self
    }

    pub fn with_ollama(mut self, base_url: Option<String>) -> Self {
        self.providers.push(Box::new(OllamaProvider::new(base_url)));
        self
    }

    pub fn with_default_model(mut self, model: String) -> Self {
        self.default_model = model;
        self
    }

    /// Execute AI query
    pub async fn query(&self, prompt: &str, model: Option<&str>) -> Result<String, AIError> {
        let model = model.unwrap_or(&self.default_model);

        let request = CompletionRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: Role::User,
                content: prompt.to_string(),
            }],
            temperature: None,
            max_tokens: None,
            system: None,
        };

        // Check cache
        let cache_key = AICache::cache_key(&request);
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached.content);
        }

        // Find provider
        let provider = self
            .providers
            .iter()
            .find(|p| p.supports_model(model))
            .ok_or_else(|| AIError::ModelNotFound(model.to_string()))?;

        let response = provider.complete(request).await?;

        // Cache response
        self.cache.set(cache_key, response.clone()).await;

        Ok(response.content)
    }

    /// Verify a condition using AI
    pub async fn verify(&self, condition: &str) -> Result<bool, AIError> {
        let prompt = format!(
            "Answer only 'true' or 'false': {}",
            condition
        );
        let result = self.query(&prompt, None).await?;
        Ok(result.trim().to_lowercase() == "true")
    }

    /// Generate embeddings
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, AIError> {
        for provider in &self.providers {
            match provider.embed(text).await {
                Ok(response) => return Ok(response.embedding),
                Err(_) => continue,
            }
        }
        Err(AIError::ProviderError("No provider supports embeddings".to_string()))
    }
}

impl Default for AIRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Newtonian agents module
pub mod agents {
    use super::*;

    /// Agent spectrum colors
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Spectrum {
        Red,     // Performance optimization
        Orange,  // Concurrency/parallelism
        Yellow,  // Contract verification
        Green,   // Configuration/dependencies
        Blue,    // Audit/security
        Indigo,  // Compile-time optimization
        Violet,  // Governance/policy
    }

    /// Agent trait
    #[async_trait]
    pub trait Agent: Send + Sync {
        fn spectrum(&self) -> Spectrum;
        async fn execute(&self, task: &str, context: &AgentContext) -> Result<AgentOutput, AIError>;
    }

    /// Agent execution context
    pub struct AgentContext {
        pub runtime: Arc<AIRuntime>,
        pub variables: HashMap<String, String>,
    }

    /// Agent output
    #[derive(Debug)]
    pub struct AgentOutput {
        pub result: String,
        pub suggestions: Vec<String>,
        pub metrics: HashMap<String, f64>,
    }

    /// Agent orchestrator
    pub struct Orchestrator {
        agents: HashMap<Spectrum, Box<dyn Agent>>,
    }

    impl Orchestrator {
        pub fn new() -> Self {
            Orchestrator {
                agents: HashMap::new(),
            }
        }

        pub fn register(&mut self, agent: Box<dyn Agent>) {
            self.agents.insert(agent.spectrum(), agent);
        }

        pub async fn execute(
            &self,
            spectrum: Spectrum,
            task: &str,
            context: &AgentContext,
        ) -> Result<AgentOutput, AIError> {
            let agent = self
                .agents
                .get(&spectrum)
                .ok_or_else(|| AIError::ProviderError(format!("No agent for {:?}", spectrum)))?;

            agent.execute(task, context).await
        }
    }

    impl Default for Orchestrator {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_runtime() {
        let runtime = AIRuntime::new();
        assert_eq!(runtime.default_model, "claude-3-opus");
    }

    #[test]
    fn test_cache_key() {
        let request = CompletionRequest {
            model: "test".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "Hello".to_string(),
            }],
            temperature: None,
            max_tokens: None,
            system: None,
        };
        let key = AICache::cache_key(&request);
        assert!(!key.is_empty());
    }
}
