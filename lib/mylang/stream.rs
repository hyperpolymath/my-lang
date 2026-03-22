//! Streaming AI Response Module
//!
//! Types and utilities for handling streaming AI responses,
//! enabling real-time output and progressive content delivery.

use std::collections::VecDeque;

// ============================================================================
// Stream Types
// ============================================================================

/// A chunk of streaming content
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub index: usize,
    pub is_final: bool,
    pub metadata: Option<ChunkMetadata>,
}

/// Metadata for a stream chunk
#[derive(Debug, Clone, Default)]
pub struct ChunkMetadata {
    pub finish_reason: Option<String>,
    pub model: Option<String>,
    pub created: Option<u64>,
}

impl StreamChunk {
    pub fn new(content: &str, index: usize) -> Self {
        StreamChunk {
            content: content.to_string(),
            index,
            is_final: false,
            metadata: None,
        }
    }

    pub fn final_chunk(content: &str, index: usize) -> Self {
        StreamChunk {
            content: content.to_string(),
            index,
            is_final: true,
            metadata: None,
        }
    }

    pub fn empty_final(index: usize) -> Self {
        StreamChunk {
            content: String::new(),
            index,
            is_final: true,
            metadata: None,
        }
    }
}

// ============================================================================
// Stream Buffer
// ============================================================================

/// Buffer for accumulating streamed content
#[derive(Debug, Clone, Default)]
pub struct StreamBuffer {
    chunks: VecDeque<StreamChunk>,
    accumulated: String,
    is_complete: bool,
}

impl StreamBuffer {
    pub fn new() -> Self {
        StreamBuffer {
            chunks: VecDeque::new(),
            accumulated: String::new(),
            is_complete: false,
        }
    }

    /// Add a chunk to the buffer
    pub fn push(&mut self, chunk: StreamChunk) {
        self.accumulated.push_str(&chunk.content);
        if chunk.is_final {
            self.is_complete = true;
        }
        self.chunks.push_back(chunk);
    }

    /// Get accumulated content so far
    pub fn content(&self) -> &str {
        &self.accumulated
    }

    /// Check if stream is complete
    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    /// Get number of chunks received
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.accumulated.clear();
        self.is_complete = false;
    }

    /// Pop the next unread chunk
    pub fn pop_chunk(&mut self) -> Option<StreamChunk> {
        self.chunks.pop_front()
    }

    /// Peek at the next unread chunk
    pub fn peek_chunk(&self) -> Option<&StreamChunk> {
        self.chunks.front()
    }

    /// Take all content and reset
    pub fn take(&mut self) -> String {
        let content = std::mem::take(&mut self.accumulated);
        self.clear();
        content
    }
}

// ============================================================================
// Stream Iterator
// ============================================================================

/// Iterator over stream chunks
pub struct StreamIterator {
    buffer: StreamBuffer,
}

impl StreamIterator {
    pub fn new() -> Self {
        StreamIterator {
            buffer: StreamBuffer::new(),
        }
    }

    /// Feed a chunk into the iterator
    pub fn feed(&mut self, chunk: StreamChunk) {
        self.buffer.push(chunk);
    }

    /// Check if there are more chunks
    pub fn has_next(&self) -> bool {
        !self.buffer.chunks.is_empty() || !self.buffer.is_complete
    }
}

impl Default for StreamIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for StreamIterator {
    type Item = StreamChunk;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.pop_chunk()
    }
}

// ============================================================================
// Stream Transformer
// ============================================================================

/// Transform streaming content on the fly
pub struct StreamTransformer<F>
where
    F: Fn(&str) -> String,
{
    transform: F,
    buffer: StreamBuffer,
}

impl<F> StreamTransformer<F>
where
    F: Fn(&str) -> String,
{
    pub fn new(transform: F) -> Self {
        StreamTransformer {
            transform,
            buffer: StreamBuffer::new(),
        }
    }

    /// Transform and push a chunk
    pub fn push(&mut self, chunk: StreamChunk) -> StreamChunk {
        let transformed_content = (self.transform)(&chunk.content);
        let transformed = StreamChunk {
            content: transformed_content,
            index: chunk.index,
            is_final: chunk.is_final,
            metadata: chunk.metadata,
        };
        self.buffer.push(transformed.clone());
        transformed
    }

    /// Get accumulated transformed content
    pub fn content(&self) -> &str {
        self.buffer.content()
    }
}

// ============================================================================
// Mock Stream (for testing)
// ============================================================================

/// Mock stream for testing streaming functionality
pub struct MockStream {
    chunks: Vec<String>,
    index: usize,
    delay_ms: u64,
}

impl MockStream {
    pub fn new(content: &str, chunk_size: usize) -> Self {
        let chunks: Vec<String> = content
            .chars()
            .collect::<Vec<_>>()
            .chunks(chunk_size)
            .map(|c| c.iter().collect())
            .collect();

        MockStream {
            chunks,
            index: 0,
            delay_ms: 0,
        }
    }

    pub fn from_chunks(chunks: Vec<String>) -> Self {
        MockStream {
            chunks,
            index: 0,
            delay_ms: 0,
        }
    }

    pub fn with_delay(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }

    /// Get next chunk (simulates async streaming)
    pub fn next_chunk(&mut self) -> Option<StreamChunk> {
        if self.delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.delay_ms));
        }

        if self.index >= self.chunks.len() {
            return None;
        }

        let is_final = self.index == self.chunks.len() - 1;
        let chunk = if is_final {
            StreamChunk::final_chunk(&self.chunks[self.index], self.index)
        } else {
            StreamChunk::new(&self.chunks[self.index], self.index)
        };

        self.index += 1;
        Some(chunk)
    }

    /// Collect all chunks to buffer
    pub fn collect(&mut self) -> StreamBuffer {
        let mut buffer = StreamBuffer::new();
        while let Some(chunk) = self.next_chunk() {
            buffer.push(chunk);
        }
        buffer
    }

    /// Reset stream to beginning
    pub fn reset(&mut self) {
        self.index = 0;
    }
}

// ============================================================================
// Stream Utilities
// ============================================================================

/// Collect stream content with a callback for each chunk
pub fn stream_with_callback<F>(stream: &mut MockStream, mut callback: F) -> String
where
    F: FnMut(&str, bool),
{
    let mut content = String::new();
    while let Some(chunk) = stream.next_chunk() {
        callback(&chunk.content, chunk.is_final);
        content.push_str(&chunk.content);
    }
    content
}

/// Format streaming output for display (with cursor effect)
pub fn format_streaming_output(partial: &str, complete: bool) -> String {
    if complete {
        partial.to_string()
    } else {
        format!("{}▌", partial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_buffer() {
        let mut buffer = StreamBuffer::new();

        buffer.push(StreamChunk::new("Hello, ", 0));
        buffer.push(StreamChunk::new("World", 1));
        buffer.push(StreamChunk::final_chunk("!", 2));

        assert_eq!(buffer.content(), "Hello, World!");
        assert!(buffer.is_complete());
        assert_eq!(buffer.chunk_count(), 3);
    }

    #[test]
    fn test_mock_stream() {
        let mut stream = MockStream::new("Hello, World!", 3);
        let buffer = stream.collect();

        assert_eq!(buffer.content(), "Hello, World!");
        assert!(buffer.is_complete());
    }

    #[test]
    fn test_stream_transformer() {
        let mut transformer = StreamTransformer::new(|s| s.to_uppercase());

        transformer.push(StreamChunk::new("hello", 0));
        transformer.push(StreamChunk::final_chunk(" world", 1));

        assert_eq!(transformer.content(), "HELLO WORLD");
    }

    #[test]
    fn test_stream_callback() {
        let mut stream = MockStream::from_chunks(vec![
            "Hello".to_string(),
            ", ".to_string(),
            "World!".to_string(),
        ]);

        let mut parts = Vec::new();
        stream_with_callback(&mut stream, |content, _is_final| {
            parts.push(content.to_string());
        });

        assert_eq!(parts, vec!["Hello", ", ", "World!"]);
    }

    #[test]
    fn test_format_streaming() {
        assert_eq!(format_streaming_output("Hello", false), "Hello▌");
        assert_eq!(format_streaming_output("Hello", true), "Hello");
    }
}
