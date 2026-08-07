// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Concurrency primitives for My Language
//!
//! Provides threads, channels, and synchronization primitives.

use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Thread handle wrapper
pub struct Thread {
    handle: Option<JoinHandle<()>>,
}

impl Thread {
    /// Spawn a new thread
    pub fn spawn<F>(f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Thread {
            handle: Some(thread::spawn(f)),
        }
    }

    /// Join the thread, waiting for it to finish
    pub fn join(&mut self) -> Result<(), String> {
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| "Thread panicked".to_string())
        } else {
            Err("Thread already joined".to_string())
        }
    }

    /// Sleep for the given duration in milliseconds
    pub fn sleep_ms(ms: u64) {
        thread::sleep(Duration::from_millis(ms));
    }

    /// Sleep for the given duration in seconds
    pub fn sleep(secs: u64) {
        thread::sleep(Duration::from_secs(secs));
    }

    /// Get the current thread ID
    pub fn current_id() -> u64 {
        thread::current().id().as_u64().get()
    }
}

/// Channel for sending messages between threads
pub struct Channel<T> {
    sender: mpsc::Sender<T>,
    receiver: Arc<Mutex<mpsc::Receiver<T>>>,
}

impl<T> Channel<T> {
    /// Create a new channel
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Channel {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    /// Send a message through the channel
    pub fn send(&self, value: T) -> Result<(), String> {
        self.sender.send(value).map_err(|_| "Receiver dropped".to_string())
    }

    /// Receive a message from the channel (blocking)
    pub fn recv(&self) -> Result<T, String> {
        self.receiver
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .recv()
            .map_err(|_| "Sender dropped".to_string())
    }

    /// Try to receive a message without blocking
    pub fn try_recv(&self) -> Option<T> {
        self.receiver.lock().unwrap_or_else(|e| e.into_inner()).try_recv().ok()
    }

    /// Clone the sender
    pub fn sender(&self) -> mpsc::Sender<T> {
        self.sender.clone()
    }
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Mutex wrapper for shared mutable state
pub struct SharedMutex<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> SharedMutex<T> {
    /// Create a new shared mutex
    pub fn new(value: T) -> Self {
        SharedMutex {
            inner: Arc::new(Mutex::new(value)),
        }
    }

    /// Lock the mutex and execute a function with the value
    pub fn with_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut *guard)
    }

    /// Clone the shared mutex
    pub fn clone(&self) -> Self {
        SharedMutex {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Clone> SharedMutex<T> {
    /// Get a copy of the value
    pub fn get(&self) -> T {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Set the value
    pub fn set(&self, value: T) {
        *self.inner.lock().unwrap_or_else(|e| e.into_inner()) = value;
    }
}

/// Read-write lock wrapper for shared state
pub struct SharedRwLock<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> SharedRwLock<T> {
    /// Create a new shared read-write lock
    pub fn new(value: T) -> Self {
        SharedRwLock {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    /// Read the value (multiple readers allowed)
    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.inner.read().unwrap_or_else(|e| e.into_inner());
        f(&*guard)
    }

    /// Write the value (exclusive access)
    pub fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.write().unwrap_or_else(|e| e.into_inner());
        f(&mut *guard)
    }

    /// Clone the shared lock
    pub fn clone(&self) -> Self {
        SharedRwLock {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Clone> SharedRwLock<T> {
    /// Get a copy of the value
    pub fn get(&self) -> T {
        self.inner.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Set the value
    pub fn set(&self, value: T) {
        *self.inner.write().unwrap_or_else(|e| e.into_inner()) = value;
    }
}

/// Atomic counter for thread-safe counting
pub struct AtomicCounter {
    value: Arc<Mutex<i64>>,
}

impl AtomicCounter {
    /// Create a new counter starting at 0
    pub fn new() -> Self {
        AtomicCounter {
            value: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a counter with an initial value
    pub fn with_value(initial: i64) -> Self {
        AtomicCounter {
            value: Arc::new(Mutex::new(initial)),
        }
    }

    /// Increment the counter and return the new value
    pub fn increment(&self) -> i64 {
        let mut val = self.value.lock().unwrap_or_else(|e| e.into_inner());
        *val += 1;
        *val
    }

    /// Decrement the counter and return the new value
    pub fn decrement(&self) -> i64 {
        let mut val = self.value.lock().unwrap_or_else(|e| e.into_inner());
        *val -= 1;
        *val
    }

    /// Get the current value
    pub fn get(&self) -> i64 {
        *self.value.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Set the value
    pub fn set(&self, new_val: i64) {
        *self.value.lock().unwrap_or_else(|e| e.into_inner()) = new_val;
    }

    /// Clone the counter
    pub fn clone(&self) -> Self {
        AtomicCounter {
            value: Arc::clone(&self.value),
        }
    }
}

impl Default for AtomicCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_spawn() {
        let mut thread = Thread::spawn(|| {
            Thread::sleep_ms(10);
        });
        assert!(thread.join().is_ok());
    }

    #[test]
    fn test_channel_send_recv() {
        let chan = Channel::new();
        chan.send(42).unwrap();
        assert_eq!(chan.recv().unwrap(), 42);
    }

    #[test]
    fn test_shared_mutex() {
        let mutex = SharedMutex::new(0);
        mutex.with_lock(|val| *val = 42);
        assert_eq!(mutex.get(), 42);
    }

    #[test]
    fn test_atomic_counter() {
        let counter = AtomicCounter::new();
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.decrement(), 1);
    }
}
