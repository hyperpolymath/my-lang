# Concurrency

My Language provides modern concurrency primitives for safe, efficient parallel and asynchronous programming.

## Async/Await

### Basic Async Functions

```ml
async fn fetch_data(url: String) -> Result<Data, Error> {
    let response = http::get(url).await?;
    let data = response.json::<Data>().await?;
    Ok(data)
}

async fn main() {
    let data = fetch_data("https://api.example.com/data").await;
    match data {
        Ok(d) => print("Got data: {d}"),
        Err(e) => print("Error: {e}"),
    }
}
```

### Awaiting Multiple Futures

```ml
async fn fetch_all(urls: List<String>) -> List<Result<Data, Error>> {
    // Sequential (one at a time)
    let mut results = List::new();
    for url in urls {
        results.push(fetch_data(url).await);
    }
    results
}

async fn fetch_all_concurrent(urls: List<String>) -> List<Result<Data, Error>> {
    // Concurrent (all at once)
    let futures = urls.map(|url| fetch_data(url));
    join_all(futures).await
}
```

### Select

Wait for the first of multiple futures:

```ml
async fn with_timeout(operation: Future<T>, timeout: Duration) -> Result<T, TimeoutError> {
    select {
        result = operation.await => Ok(result),
        _ = sleep(timeout).await => Err(TimeoutError),
    }
}

async fn race_requests() -> Data {
    select {
        data = fetch_from_primary().await => data,
        data = fetch_from_backup().await => data,
    }
}
```

## Spawn and Tasks

### Spawning Tasks

```ml
async fn main() {
    // Spawn a task
    let handle = spawn {
        expensive_computation()
    };

    // Do other work
    do_something_else();

    // Wait for the task
    let result = handle.await;
}
```

### Multiple Tasks

```ml
async fn parallel_processing(items: List<Item>) -> List<Result> {
    let handles = items.map(|item| {
        spawn { process_item(item) }
    });

    // Wait for all
    let results = List::new();
    for handle in handles {
        results.push(handle.await);
    }
    results
}
```

### Task Groups

```ml
async fn scoped_tasks() {
    // All tasks must complete before scope exits
    scope {
        spawn { task1() };
        spawn { task2() };
        spawn { task3() };
    }.await;  // Waits for all three

    print("All tasks complete");
}

async fn collect_results() -> List<Int> {
    scope {
        let handles = (0..10).map(|i| {
            spawn { compute(i) }
        });
        handles.map(|h| h.await).collect()
    }.await
}
```

## Go Blocks

Lightweight concurrent execution (similar to goroutines):

```ml
fn main() {
    // Start concurrent execution
    go {
        for i in 0..100 {
            print("Background: {i}");
            yield;  // Yield to other tasks
        }
    };

    // Main continues immediately
    for i in 0..100 {
        print("Main: {i}");
    }
}
```

### Go with Result

```ml
fn compute_in_background() -> Handle<Int> {
    go {
        expensive_computation()
    }
}

fn main() {
    let handle = compute_in_background();

    // Do other work
    let other_result = quick_computation();

    // Get background result
    let bg_result = handle.join();
}
```

## Channels

### Unbounded Channels

```ml
fn main() {
    let (tx, rx) = channel::<Int>();

    // Sender
    go {
        for i in 0..10 {
            tx.send(i);
        }
        tx.close();
    };

    // Receiver
    while let Some(value) = rx.recv() {
        print("Received: {value}");
    }
}
```

### Bounded Channels

```ml
fn main() {
    // Buffer of 10 items
    let (tx, rx) = bounded_channel::<String>(10);

    // Producer
    go {
        for item in data {
            tx.send(item);  // Blocks if buffer full
        }
    };

    // Consumer
    go {
        while let Some(item) = rx.recv() {
            process(item);
        }
    };
}
```

### Select on Channels

```ml
fn main() {
    let (tx1, rx1) = channel::<Int>();
    let (tx2, rx2) = channel::<String>();

    loop {
        select {
            value = rx1.recv() => {
                print("Got int: {value}");
            }
            value = rx2.recv() => {
                print("Got string: {value}");
            }
            default => {
                print("No messages");
                break;
            }
        }
    }
}
```

## Synchronization Primitives

### Mutex

```ml
use std::sync::Mutex;

struct Counter {
    value: Mutex<Int>,
}

impl Counter {
    fn new() -> Counter {
        Counter { value: Mutex::new(0) }
    }

    fn increment(&self) {
        let mut guard = self.value.lock();
        *guard += 1;
    }

    fn get(&self) -> Int {
        *self.value.lock()
    }
}

fn main() {
    let counter = Counter::new();

    // Multiple tasks incrementing
    scope {
        for _ in 0..100 {
            spawn {
                counter.increment();
            };
        }
    }.await;

    print("Final count: {counter.get()}");  // 100
}
```

### RwLock

```ml
use std::sync::RwLock;

struct Cache {
    data: RwLock<Map<String, Value>>,
}

impl Cache {
    fn get(&self, key: &String) -> Option<Value> {
        let guard = self.data.read();  // Multiple readers
        guard.get(key).cloned()
    }

    fn set(&self, key: String, value: Value) {
        let mut guard = self.data.write();  // Exclusive writer
        guard.insert(key, value);
    }
}
```

### Atomic Types

```ml
use std::sync::atomic::{AtomicInt, AtomicBool};

struct Stats {
    requests: AtomicInt,
    running: AtomicBool,
}

impl Stats {
    fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::SeqCst);
    }

    fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
```

### Once

```ml
use std::sync::Once;

static INIT: Once = Once::new();
static mut CONFIG: Option<Config> = None;

fn get_config() -> &Config {
    INIT.call_once(|| {
        unsafe {
            CONFIG = Some(load_config());
        }
    });
    unsafe { CONFIG.as_ref().unwrap() }
}
```

## Concurrent AI Operations

### Parallel AI Queries

```ml
async fn analyze_documents(docs: List<String>) -> List<Analysis> {
    // Run AI queries in parallel
    let futures = docs.map(|doc| async {
        ai query {
            prompt: "Analyze: {doc}"
        }
    });

    join_all(futures).await
}
```

### AI with Timeout

```ml
async fn safe_ai_query(prompt: String) -> Result<String, Error> {
    select {
        result = ai query { prompt: prompt }.await => {
            Ok(result)
        }
        _ = sleep(Duration::seconds(30)).await => {
            Err(Error::Timeout)
        }
    }
}
```

### Rate-Limited AI Calls

```ml
use std::sync::Semaphore;

struct RateLimitedAI {
    semaphore: Semaphore,
}

impl RateLimitedAI {
    fn new(max_concurrent: Int) -> Self {
        RateLimitedAI {
            semaphore: Semaphore::new(max_concurrent),
        }
    }

    async fn query(&self, prompt: String) -> AI<String> {
        let _permit = self.semaphore.acquire().await;
        ai query { prompt: prompt }
    }
}

async fn batch_queries(prompts: List<String>) {
    let limiter = RateLimitedAI::new(5);  // Max 5 concurrent

    let futures = prompts.map(|p| limiter.query(p));
    join_all(futures).await
}
```

## Structured Concurrency

### Scoped Tasks

All spawned tasks are bounded to a scope:

```ml
async fn process_batch(items: List<Item>) -> List<Result> {
    scope { |s|
        let handles = items.map(|item| {
            s.spawn { process(item) }
        });

        handles.map(|h| h.await).collect()
    }.await
    // All tasks guaranteed complete here
}
```

### Cancellation

```ml
async fn cancellable_operation() {
    let (cancel_tx, cancel_rx) = channel::<()>();

    let handle = spawn {
        loop {
            select {
                _ = cancel_rx.recv() => {
                    print("Cancelled!");
                    break;
                }
                _ = do_work().await => {
                    // Continue working
                }
            }
        }
    };

    // Cancel after 5 seconds
    sleep(Duration::seconds(5)).await;
    cancel_tx.send(());

    handle.await;
}
```

### Supervision

```ml
async fn supervised_workers(tasks: List<Task>) {
    let mut handles = tasks.map(|t| spawn { run_task(t) });

    loop {
        for (i, handle) in handles.iter_mut().enumerate() {
            if handle.is_finished() {
                match handle.result() {
                    Ok(_) => {
                        print("Task {i} completed");
                    }
                    Err(e) => {
                        print("Task {i} failed: {e}, restarting");
                        handles[i] = spawn { run_task(tasks[i]) };
                    }
                }
            }
        }
        yield;
    }
}
```

## Actor Model

### Basic Actor

```ml
actor Counter {
    state count: Int = 0;

    fn increment(&mut self) {
        self.count += 1;
    }

    fn decrement(&mut self) {
        self.count -= 1;
    }

    fn get(&self) -> Int {
        self.count
    }
}

async fn main() {
    let counter = Counter::spawn();

    counter.increment();
    counter.increment();
    counter.decrement();

    let value = counter.get().await;
    print("Count: {value}");  // 1
}
```

### Actor with Messages

```ml
enum CounterMsg {
    Increment,
    Decrement,
    Get { reply: Sender<Int> },
}

actor Counter {
    state count: Int = 0;

    fn handle(&mut self, msg: CounterMsg) {
        match msg {
            CounterMsg::Increment => self.count += 1,
            CounterMsg::Decrement => self.count -= 1,
            CounterMsg::Get { reply } => reply.send(self.count),
        }
    }
}
```

## Best Practices

### 1. Prefer Structured Concurrency

```ml
// Good: Tasks bounded to scope
scope {
    spawn { task1() };
    spawn { task2() };
}.await;

// Avoid: Unstructured spawning
spawn { task1() };  // May outlive caller
spawn { task2() };
```

### 2. Use Channels for Communication

```ml
// Good: Communicate via channels
let (tx, rx) = channel();
spawn { tx.send(compute()) };
let result = rx.recv();

// Avoid: Shared mutable state
let shared = Arc::new(Mutex::new(0));
spawn { *shared.lock() = compute() };  // More complex
```

### 3. Handle Cancellation

```ml
// Good: Respect cancellation
async fn operation(cancel: Receiver<()>) {
    select {
        _ = cancel.recv() => return,
        result = do_work() => use(result),
    }
}

// Avoid: Ignoring cancellation
async fn operation() {
    do_work().await;  // Cannot be cancelled
}
```

### 4. Avoid Deadlocks

```ml
// Good: Lock ordering
fn transfer(from: &Account, to: &Account, amount: Int) {
    let (first, second) = if from.id < to.id {
        (from, to)
    } else {
        (to, from)
    };
    let _g1 = first.lock();
    let _g2 = second.lock();
    // Transfer...
}

// Avoid: Inconsistent lock ordering
fn transfer(from: &Account, to: &Account, amount: Int) {
    let _g1 = from.lock();
    let _g2 = to.lock();  // May deadlock!
}
```

## Performance Tips

1. **Right-size your thread pool**: Match to CPU cores for CPU-bound, larger for I/O-bound
2. **Use bounded channels**: Prevent memory growth under load
3. **Batch AI calls**: Reduce overhead with batching
4. **Minimize lock scope**: Hold locks for shortest time possible
5. **Profile first**: Identify actual bottlenecks before optimizing

## Related Documentation

- [Async Runtime Internals](../internals/async-runtime.md)
- [Effect System](effects.md)
- [AI Features](ai-features.md)
