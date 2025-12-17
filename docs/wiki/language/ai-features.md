# AI Features

My Language provides first-class AI integration, making AI operations as natural as any other language feature.

## Overview

AI capabilities in My Language are:
- **Type-safe**: AI operations return typed values with effect tracking
- **Composable**: AI calls can be combined with regular code seamlessly
- **Configurable**: Fine-grained control over models, parameters, and behavior
- **Cacheable**: Built-in caching and optimization for AI calls

## Quick AI Queries

The simplest way to use AI is the `ai!` macro:

```ml
// Basic query
let answer = ai! { "What is the capital of France?" };
// answer: AI<String> = "Paris"

// With variable interpolation
let topic = "quantum computing";
let explanation = ai! { "Explain {topic} in simple terms" };

// Multi-line prompts
let story = ai! {
    "Write a short story about a programmer who discovers
    their code has become sentient. Make it exactly 100 words."
};
```

## AI Expressions

For more control, use full AI expressions:

### Query

```ml
let response = ai query {
    prompt: "Analyze this code for bugs"
    context: code_snippet
    model: "gpt-4"
    temperature: 0.3
    max_tokens: 500
};
```

### Verify

Validate data against constraints:

```ml
let is_valid = ai verify {
    input: user_data
    constraint: "must be a valid email address"
};
// is_valid: AI<Bool>

let json_valid = ai verify {
    input: json_string
    constraint: "must be valid JSON matching schema: { name: string, age: number }"
};
```

### Generate

Generate content with specific requirements:

```ml
let code = ai generate {
    prompt: "Generate a sorting function"
    language: "rust"
    style: "functional"
};

let email = ai generate {
    prompt: "Write a professional email"
    context: {
        subject: "Meeting request",
        recipient: "John",
        purpose: "discuss project timeline",
    }
    tone: "formal"
    max_length: 200
};
```

### Embed

Create vector embeddings for semantic search:

```ml
let embedding = ai embed(document);
// embedding: AI<List<Float>> - typically 1536 dimensions

// Use for similarity search
let similar_docs = vector_db.search(embedding, limit: 10);
```

### Classify

Categorize content:

```ml
let sentiment = ai classify {
    input: review_text
    categories: ["positive", "negative", "neutral"]
};
// sentiment: AI<String>

let intent = ai classify {
    input: user_message
    categories: [
        "question",
        "complaint",
        "feedback",
        "purchase",
        "support",
    ]
    allow_multiple: true
};
// intent: AI<List<String>>
```

## AI Models

Declare reusable AI model configurations:

```ml
ai_model Summarizer {
    provider: "openai"
    model: "gpt-4-turbo"
    temperature: 0.3
    max_tokens: 500
    system_prompt: "You are a concise summarizer. Always respond with bullet points."
}

ai_model CodeReviewer {
    provider: "anthropic"
    model: "claude-3-opus"
    temperature: 0.1
    system_prompt: """
        You are an expert code reviewer. Focus on:
        - Security vulnerabilities
        - Performance issues
        - Code clarity
        - Best practices
    """
}

ai_model LocalLLM {
    provider: "local"
    model_path: "./models/llama-7b.gguf"
    gpu_layers: 35
    context_size: 4096
}
```

Using models:

```ml
// Use specific model
let summary = ai query {
    model: Summarizer
    prompt: "Summarize this document"
    context: document
};

let review = ai query {
    model: CodeReviewer
    prompt: "Review this code"
    context: source_code
};
```

## Prompt Templates

Define reusable, typed prompts:

```ml
prompt summarize(
    text: String,
    max_words: Int = 100,
    style: String = "concise"
) -> String {
    """
    Summarize the following text in at most {max_words} words.
    Style: {style}

    Text:
    {text}

    Summary:
    """
}

prompt extract_entities(text: String) -> List<Entity> {
    """
    Extract all named entities from the following text.
    Return as JSON array with format: [{"name": "...", "type": "..."}]

    Types: PERSON, ORGANIZATION, LOCATION, DATE, PRODUCT

    Text: {text}
    """
}

prompt code_review(
    code: String,
    language: String,
    focus: List<String> = ["bugs", "security", "style"]
) -> CodeReview {
    """
    Review the following {language} code.
    Focus areas: {focus.join(", ")}

    Code:
    ```{language}
    {code}
    ```

    Provide review as JSON:
    {
        "issues": [{"severity": "high|medium|low", "line": N, "description": "..."}],
        "suggestions": ["..."],
        "overall_quality": 1-10
    }
    """
}
```

Using prompts:

```ml
// Invoke with prompt! macro
let summary = summarize!(article_text);
let summary = summarize!(article_text, max_words: 50);

let entities = extract_entities!(document);

let review = code_review!(
    source_code,
    language: "rust",
    focus: ["security", "performance"]
);
```

## AI Pipelines

Chain AI operations together:

```ml
// Pipeline syntax
let result = text
    |> ai embed
    |> search_similar_docs(db, _)
    |> ai query { prompt: "Synthesize: {_}" }
    |> ai verify { input: _, constraint: "factually accurate" };

// Explicit pipeline
fn analyze_document(doc: String) -> AI<Analysis> {
    // Step 1: Extract key information
    let entities = extract_entities!(doc);

    // Step 2: Summarize
    let summary = summarize!(doc, max_words: 200);

    // Step 3: Generate analysis
    let analysis = ai query {
        prompt: """
            Given these entities: {entities}
            And this summary: {summary}
            Provide a detailed analysis.
        """
    };

    analysis
}
```

## Streaming

Handle streaming AI responses:

```ml
// Streaming response
async fn stream_story(prompt: String) -> Stream<String> {
    ai stream {
        prompt: prompt
        on_token: |token| yield token
        on_complete: |full_text| log("Complete: {full_text.len()} chars")
    }
}

// Usage
let stream = stream_story("Write an epic tale").await;
for await token in stream {
    print(token);  // Print each token as it arrives
}

// Collect stream
let full_response = stream.collect::<String>().await;
```

## Error Handling

AI operations can fail:

```ml
fn safe_query(prompt: String) -> Result<String, AIError> {
    match ai query { prompt: prompt } {
        Ok(response) => Ok(response),
        Err(AIError::RateLimited { retry_after }) => {
            sleep(retry_after);
            safe_query(prompt)  // Retry
        }
        Err(AIError::InvalidPrompt { reason }) => {
            Err(AIError::InvalidPrompt { reason })
        }
        Err(e) => Err(e),
    }
}

// With timeout
let result = timeout(Duration::seconds(30)) {
    ai query { prompt: long_prompt }
}.await;

match result {
    Ok(response) => use(response),
    Err(TimeoutError) => fallback(),
}
```

## Caching

Cache AI responses for efficiency:

```ml
#[ai_cached(ttl: 3600)]  // Cache for 1 hour
fn get_embedding(text: String) -> AI<List<Float>> {
    ai embed(text)
}

#[ai_cached(key: |q| hash(q), ttl: 86400)]
fn answer_faq(question: String) -> AI<String> {
    ai query {
        prompt: "Answer this FAQ: {question}"
        model: FAQModel
    }
}

// Manual cache control
let cache = AICache::new(capacity: 1000);

let result = cache.get_or_compute("key", || {
    ai query { prompt: "expensive query" }
}).await;
```

## Testing AI Code

Mock AI responses in tests:

```ml
#[test]
#[ai_mock(responses: ["mocked response"])]
fn test_ai_function() {
    let result = ai query { prompt: "test" };
    assert_eq(result, "mocked response");
}

#[test]
#[ai_mock(file: "test_fixtures/ai_responses.json")]
fn test_with_fixtures() {
    let summary = summarize!("test text");
    assert(summary.contains("expected content"));
}

// Programmatic mocking
fn test_complex_flow() {
    let mock = AIMock::new()
        .on_query("pattern1").respond("response1")
        .on_query("pattern2").respond("response2")
        .on_embed(_).respond(vec![0.1; 1536]);

    with_ai_mock(mock) {
        run_ai_dependent_code();
    }
}
```

## Cost Tracking

Monitor AI usage:

```ml
// Track costs
let tracker = AITracker::new();

with_ai_tracker(tracker) {
    let result1 = ai query { prompt: "query 1" };
    let result2 = ai query { prompt: "query 2" };
}

print("Total tokens: {tracker.total_tokens()}");
print("Estimated cost: ${tracker.estimated_cost()}");

// Budget limits
#[ai_budget(max_tokens: 10000, max_cost: 1.00)]
fn limited_operation() -> AI<String> {
    // Will fail if budget exceeded
    ai query { prompt: "..." }
}
```

## Provider Configuration

Configure AI providers at runtime:

```ml
// Set default provider
AIConfig::set_default_provider(OpenAI {
    api_key: env("OPENAI_API_KEY"),
    organization: env("OPENAI_ORG"),
});

// Add multiple providers
AIConfig::add_provider("anthropic", Anthropic {
    api_key: env("ANTHROPIC_API_KEY"),
});

AIConfig::add_provider("local", LocalLLM {
    model_path: "./models/llama.gguf",
});

// Use specific provider
let result = ai query {
    provider: "anthropic"
    prompt: "..."
};
```

## Best Practices

### 1. Use Typed Prompts

```ml
// Good: Typed return value
prompt get_user(name: String) -> User { ... }
let user: AI<User> = get_user!("Alice");

// Avoid: Untyped responses
let response = ai! { "Get user Alice" };  // AI<String>
```

### 2. Handle Failures

```ml
// Good: Explicit error handling
match ai query { prompt: p } {
    Ok(r) => process(r),
    Err(e) => handle_error(e),
}

// Avoid: Unwrapping without handling
let result = ai query { prompt: p }.unwrap();  // May panic
```

### 3. Set Appropriate Timeouts

```ml
// Good: Bounded operations
let result = timeout(Duration::seconds(30)) {
    ai query { prompt: long_prompt }
}.await?;

// Avoid: Unbounded AI calls in production
let result = ai query { prompt: p };  // May hang
```

### 4. Cache When Appropriate

```ml
// Good: Cache expensive, repeatable queries
#[ai_cached(ttl: 3600)]
fn get_embedding(text: String) -> AI<List<Float>> { ... }

// Avoid: Caching dynamic/personalized content
#[ai_cached]  // Wrong: User-specific content shouldn't be cached
fn get_recommendations(user: User) -> AI<List<Item>> { ... }
```

### 5. Use Appropriate Models

```ml
// Good: Match model to task
ai query { model: FastModel, prompt: simple_query }
ai query { model: PowerfulModel, prompt: complex_reasoning }

// Avoid: Using expensive models for simple tasks
ai query { model: GPT4, prompt: "Is 'hello' a greeting?" }
```

## AI Type Reference

| Type | Description |
|------|-------------|
| `AI<T>` | AI computation returning type T |
| `AIError` | Error from AI operations |
| `AIModel` | Model configuration |
| `AIProvider` | Provider interface |
| `AICache` | Response cache |
| `AITracker` | Usage tracker |
| `Stream<T>` | Streaming AI response |

## Related Documentation

- [AI Type System](types.md#ai-types)
- [Effect System](effects.md)
- [Neuron Framework](../ecosystem/neuron-ai.md)
- [Testing AI Code](../guides/testing.md#testing-ai)
