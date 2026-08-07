// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! AI Runtime for My Language
//!
//! Provides AI model abstraction and execution:
//! - Multiple provider support (Anthropic, OpenAI, Ollama)
//! - Response caching with rocketcache
//! - Newtonian agent orchestration
//! - Streaming support
//!
//! # Security
//!
//! API keys are stored using secure memory practices:
//! - Keys are zeroized when dropped
//! - Debug output never shows actual key values
//! - Keys are cloned minimally to reduce exposure

#![forbid(unsafe_code)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Secure API key wrapper that zeroizes memory on drop
///
/// This wrapper ensures that API keys are securely handled:
/// - Memory is zeroed when the key is dropped
/// - Debug output shows "[REDACTED]" instead of the actual key
/// - Clone is implemented but should be used sparingly
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureApiKey(String);

impl SecureApiKey {
    /// Create a new secure API key
    pub fn new(key: String) -> Self {
        SecureApiKey(key)
    }

    /// Get a reference to the key for use in API calls
    ///
    /// # Security
    /// This should only be used when making actual API requests.
    /// Never log or serialize the returned value.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecureApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecureApiKey([REDACTED])")
    }
}

impl From<String> for SecureApiKey {
    fn from(s: String) -> Self {
        SecureApiKey::new(s)
    }
}

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
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
    #[serde(default)]
    pub stream: bool,
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
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
}

/// Tool call from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
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
    api_key: SecureApiKey,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        AnthropicProvider {
            api_key: SecureApiKey::new(api_key),
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
            .header("x-api-key", self.api_key.expose())
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
            tool_calls: Vec::new(),
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
    api_key: SecureApiKey,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        OpenAIProvider {
            api_key: SecureApiKey::new(api_key),
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
            .header("Authorization", format!("Bearer {}", self.api_key.expose()))
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
            tool_calls: Vec::new(),
        })
    }

    async fn embed(&self, text: &str) -> Result<EmbeddingResponse, AIError> {
        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key.expose()))
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
            tool_calls: Vec::new(),
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
    ttl: std::time::Duration,
}

#[derive(Debug, Clone)]
struct CachedResponse {
    response: CompletionResponse,
    timestamp: std::time::Instant,
}

/// Default time-to-live for cached AI responses. Picked as five minutes
/// because: (a) AI responses are model-call expensive enough that short
/// repeats deserve to hit the cache, (b) five minutes is short enough
/// that long-lived processes don't serve stale answers indefinitely or
/// retain memory for keys nobody asks about anymore.
const DEFAULT_TTL: std::time::Duration = std::time::Duration::from_secs(300);

impl AICache {
    /// Build a cache with the [`DEFAULT_TTL`] (five minutes).
    pub fn new() -> Self {
        Self::with_ttl(DEFAULT_TTL)
    }

    /// Build a cache with a custom time-to-live.
    ///
    /// Entries older than `ttl` are treated as absent by [`Self::get`]
    /// and are opportunistically evicted on the next [`Self::set`].
    pub fn with_ttl(ttl: std::time::Duration) -> Self {
        AICache {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// Look up `key`. Returns `None` if the entry is absent or its
    /// timestamp has exceeded `ttl` -- an expired entry is reported as
    /// absent but not removed under the read lock; eviction happens on
    /// the next write so we don't pay for an upgrade here.
    pub async fn get(&self, key: &str) -> Option<CompletionResponse> {
        let cache = self.cache.read().await;
        cache.get(key).and_then(|c| {
            if c.timestamp.elapsed() <= self.ttl {
                Some(c.response.clone())
            } else {
                None
            }
        })
    }

    /// Insert `response` for `key`. As a side effect, every entry whose
    /// timestamp has exceeded `ttl` is dropped -- we already hold the
    /// write lock, so this lazy sweep keeps memory bounded without a
    /// background task.
    pub async fn set(&self, key: String, response: CompletionResponse) {
        let mut cache = self.cache.write().await;
        let ttl = self.ttl;
        cache.retain(|_, entry| entry.timestamp.elapsed() <= ttl);
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
    pub providers: Vec<Box<dyn AIProvider>>,
    cache: AICache,
    pub default_model: String,
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
            tools: Vec::new(),
            stream: false,
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

/// Create runtime from environment variables
/// Checks ANTHROPIC_API_KEY, OPENAI_API_KEY, OLLAMA_HOST
pub fn runtime_from_env() -> AIRuntime {
    let mut runtime = AIRuntime::new();

    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        runtime = runtime.with_anthropic(key);
        runtime = runtime.with_default_model("claude-sonnet-4-20250514".to_string());
    }

    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        runtime = runtime.with_openai(key);
        if runtime.providers.len() == 1 {
            runtime = runtime.with_default_model("gpt-4o".to_string());
        }
    }

    if std::env::var("OLLAMA_HOST").is_ok() || std::env::var("OLLAMA_ENABLED").is_ok() {
        let host = std::env::var("OLLAMA_HOST").ok();
        runtime = runtime.with_ollama(host);
    }

    runtime
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
            tools: vec![],
            stream: false,
        };
        let key = AICache::cache_key(&request);
        assert!(!key.is_empty());
    }

    fn dummy_response(content: &str) -> CompletionResponse {
        CompletionResponse {
            content: content.to_string(),
            model: "test".to_string(),
            usage: Usage { input_tokens: 0, output_tokens: 0 },
            tool_calls: Vec::new(),
        }
    }

    #[tokio::test]
    async fn test_cache_round_trip() {
        let cache = AICache::new();
        assert!(cache.get("k").await.is_none());
        cache.set("k".to_string(), dummy_response("hello")).await;
        assert_eq!(cache.get("k").await.map(|r| r.content), Some("hello".to_string()));
    }

    #[tokio::test]
    async fn test_cache_ttl_expires_entries() {
        // 50 ms TTL: insert, sleep past it, expect the entry to be reported
        // as absent. Regression for the field that used to be captured at
        // `set` time and never consulted on `get`.
        let cache = AICache::with_ttl(std::time::Duration::from_millis(50));
        cache.set("k".to_string(), dummy_response("hello")).await;
        assert!(cache.get("k").await.is_some(), "entry should be fresh");
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        assert!(
            cache.get("k").await.is_none(),
            "entry should be reported absent once past TTL"
        );
    }

    #[tokio::test]
    async fn test_cache_set_evicts_expired_entries() {
        // After TTL elapses, a subsequent `set` should sweep stale entries
        // out so the cache cannot grow unboundedly even if old keys are
        // never read again.
        let cache = AICache::with_ttl(std::time::Duration::from_millis(50));
        cache.set("old1".to_string(), dummy_response("a")).await;
        cache.set("old2".to_string(), dummy_response("b")).await;
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        cache.set("fresh".to_string(), dummy_response("c")).await;
        let inner = cache.cache.read().await;
        assert_eq!(inner.len(), 1, "expired entries should be swept on set");
        assert!(inner.contains_key("fresh"));
    }
}
