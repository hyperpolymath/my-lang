// My Language Demo: AI-First Programming
// This file demonstrates the key features of the language

// ============================================
// 1. AI Model Configuration
// ============================================

ai_model claude {
    provider: "anthropic"
    model: "claude-3-opus"
    temperature: 0.7
    cache: true
}

ai_model gpt4 {
    provider: "openai"
    model: "gpt-4-turbo"
    temperature: 0.5
    cache: false
}

// ============================================
// 2. Prompt Templates
// ============================================

prompt summarize { "Summarize the following text in 3 sentences: {text}" }

prompt translate { "Translate the following from {source_lang} to {target_lang}: {text}" }

prompt analyze_sentiment { "Analyze the sentiment of: {text}. Return: positive, negative, or neutral." }

// ============================================
// 3. Structs with AI Features
// ============================================

#[derive(Debug, Clone)]
struct User {
    name: String,
    #[ai_validate("valid email format")]
    email: String,
    #[ai_embed]
    bio: String,
    age: Int,
}

#[ai_generate]
struct Article {
    title: String,
    #[ai_embed]
    content: String,
    #[ai_validate("ISO date format")]
    published_date: String,
}

// ============================================
// 4. Effects
// ============================================

effect Logger {
    op log: String
    op error: String
}

effect Database {
    op query: String
    op insert: String
}

// ============================================
// 5. Functions with AI Features
// ============================================

// Basic function
fn greet(name: String) -> String {
    return "Hello, " + name;
}

// Function with AI return type
fn ask_ai(question: String) -> AI<String> {
    let answer = ai query {
        prompt: question
        model: claude
    };
    return answer;
}

// Function with contracts and AI checks
#[ai_optimize]
fn divide(a: Int, b: Int) -> Int
    where pre: b != 0, ai_ensure: "result is mathematically correct"
{
    return a / b;
}

// Function with AI test generation
#[ai_test]
fn fibonacci(n: Int) -> Int {
    if n <= 1 {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}

// ============================================
// 6. AI Expressions
// ============================================

fn demonstrate_ai_expressions() {
    // Quick AI query
    let answer = ai! { "What is the capital of France?" };

    // AI query with structured body
    let analysis = ai query {
        prompt: "Analyze this code"
        context: code_snippet
    };

    // AI verification
    let is_valid = ai verify {
        input: user_data
        constraint: "must be valid JSON"
    };

    // AI generation
    let generated = ai generate(template, context);

    // AI embedding
    let embedding = ai embed(document);

    // AI classification
    let category = ai classify(text);

    // Prompt template invocation
    let summary = summarize!(article_text);
}

// ============================================
// 7. Control Flow
// ============================================

fn control_flow_demo(x: Int) {
    // If-else
    if x > 0 {
        let positive = true;
    } else {
        let negative = true;
    }

    // Match expression
    let result = match x {
        0 => "zero",
        1 => "one",
        _ => "other",
    };

    // Lambda expressions
    let double = |n: Int| => n * 2;
    let complex = |a: Int, b: Int| {
        let sum = a + b;
        return sum * 2;
    };
}

// ============================================
// 8. Concurrency
// ============================================

fn concurrent_demo() {
    // Spawn concurrent task
    go {
        let result = ai query { prompt: "background task" };
    }
}

// ============================================
// 9. Comptime
// ============================================

comptime {
    let x = 42;
}

#[comptime]
fn compile_time_check() {
    let y = 100;
}

// ============================================
// 10. Imports
// ============================================

use std::io;
use std::collections::{HashMap, Vec};

// ============================================
// 11. Main Entry Point
// ============================================

fn main() {
    // Use AI to analyze
    let analysis = ai query {
        prompt: "Describe this user profile"
    };

    // Quick AI query
    let greeting = ai! { "Generate a friendly greeting" };

    // Match on result
    match analysis {
        Ok(result) => result,
        Err(e) => "Analysis failed",
    };
}
