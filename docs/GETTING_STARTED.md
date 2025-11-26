# Getting Started with Nexus Functions

## Quick Start (5 Minutes)

This guide will walk you through deploying your first serverless function with Nexus.

## Prerequisites

- Rust 1.75+ (`rustup` installed)
- Cargo
- WASM target: `rustup target add wasm32-wasi`

## Installation

### Option 1: Build from Source

```bash
# Clone the repository
git clone https://github.com/nexus-faas/core.git
cd core

# Build the CLI
cargo build --release

# Add to PATH (optional)
export PATH=$PATH:$(pwd)/target/release
```

### Option 2: Download Binary (Future)

```bash
# Download latest release
curl -sSL https://nexus.dev/install.sh | sh
```

## Create Your First Function

### 1. Create a New Project

```bash
nexus new my-first-function --lang=rust
cd my-first-function
```

This generates:
```
my-first-function/
├── nexus.yaml          # Function configuration
├── src/
│   └── lib.rs          # Function handler
├── Cargo.toml          # Rust dependencies
└── build.sh            # Build script
```

### 2. Examine the Configuration

**nexus.yaml:**
```yaml
version: v1

functions:
  - name: hello-world
    on:
      http:
        method: POST
        path: /events/hello
    runtime: wasi-preview1
    code: ./build/handler.wasm
    timeout: 5s
    memory: 128Mi
```

**src/lib.rs:**
```rust
use std::io::{stdin, stdout, Read, Write};

#[no_mangle]
pub extern "C" fn handle_event() {
    // Read event payload from stdin
    let mut input = Vec::new();
    stdin().read_to_end(&mut input).unwrap();
    
    // Parse event (simplified)
    let event_data = String::from_utf8_lossy(&input);
    eprintln!("[INFO] Processing event: {}", event_data);
    
    // Generate response
    let response = format!(
        r#"{{"message": "Hello from Nexus!", "received": {}}}"#,
        event_data
    );
    
    // Write response to stdout
    stdout().write_all(response.as_bytes()).unwrap();
}
```

### 3. Build the Function

```bash
# Compile to WASM
cargo build --target wasm32-wasi --release

# Create build directory
mkdir -p build
cp target/wasm32-wasi/release/my_first_function.wasm build/handler.wasm
```

Or use the build script:
```bash
./build.sh
```

### 4. Start Development Server

```bash
nexus dev
```

Output:
```
✓ Starting embedded NATS JetStream...
✓ Loaded configuration from nexus.yaml
✓ Registered function 'hello-world'
✓ Serving HTTP on http://localhost:8080

Ready to receive events!
```

### 5. Trigger Your Function

In another terminal:

```bash
curl -X POST http://localhost:8080/events/hello \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice", "message": "Testing Nexus!"}'
```

Response:
```json
{
  "event_id": "abc123-def456-789",
  "status": "published",
  "function": "hello-world"
}
```

### 6. View Logs

The development server will show:

```
[2025-11-26T10:23:45Z INFO nexus] [trace=abc123] Event received: POST /events/hello
[2025-11-26T10:23:45Z INFO nexus] [trace=abc123] Executing function 'hello-world'
[2025-11-26T10:23:45Z INFO hello-world] [trace=abc123] Processing event: {"name": "Alice", ...}
[2025-11-26T10:23:45Z INFO nexus] [trace=abc123] Function completed in 1.2ms
```

### 7. Replay the Event

Copy the event ID from the response and replay:

```bash
nexus replay abc123-def456-789
```

Output:
```
✓ Found event abc123-def456-789
✓ Replaying with original payload...
[trace=abc123-replay] Processing event: {"name": "Alice", ...}
✓ Replay completed in 0.9ms

Output: {"message": "Hello from Nexus!", "received": {"name": "Alice", ...}}
```

## Understanding the Event Flow

```
1. HTTP Request → API Gateway
2. Convert to CloudEvent → Add event ID, timestamp
3. Publish to NATS JetStream → Persistent storage
4. Trigger function → Load WASM, inject trace ID
5. Execute → Read from stdin, write to stdout
6. Log output → With trace context
7. Store result → Available for replay
```

## Writing More Complex Functions

### Parsing JSON Events

```rust
use serde::{Deserialize, Serialize};
use std::io::{stdin, stdout, Read, Write};

#[derive(Deserialize)]
struct UserEvent {
    user_id: String,
    email: String,
}

#[derive(Serialize)]
struct WelcomeResponse {
    message: String,
    user_id: String,
}

#[no_mangle]
pub extern "C" fn handle_event() {
    let mut input = Vec::new();
    stdin().read_to_end(&mut input).unwrap();
    
    // Parse JSON
    let event: UserEvent = serde_json::from_slice(&input).unwrap();
    
    // Process
    let response = WelcomeResponse {
        message: format!("Welcome, {}!", event.email),
        user_id: event.user_id,
    };
    
    // Output JSON
    let output = serde_json::to_vec(&response).unwrap();
    stdout().write_all(&output).unwrap();
}
```

**Cargo.toml:**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Accessing Environment Variables

```rust
fn handle_event() {
    // Access trace ID
    let trace_id = std::env::var("TRACE_ID")
        .unwrap_or_else(|_| "unknown".to_string());
    
    eprintln!("[trace={}] Starting execution", trace_id);
    
    // Custom env vars from nexus.yaml
    let log_level = std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string());
}
```

### Error Handling

```rust
use std::process;

#[no_mangle]
pub extern "C" fn handle_event() {
    let result = process_event();
    
    match result {
        Ok(output) => {
            stdout().write_all(&output).unwrap();
        }
        Err(e) => {
            eprintln!("[ERROR] Function failed: {}", e);
            process::exit(1); // Exit with error code
        }
    }
}

fn process_event() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Your logic here
    Ok(vec![])
}
```

## Testing Functions Locally

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_processing() {
        let input = r#"{"user_id": "123", "email": "test@example.com"}"#;
        let result = process_event(input.as_bytes());
        assert!(result.is_ok());
    }
}
```

Run tests:
```bash
cargo test
```

### Integration Tests

Create `tests/integration_test.rs`:

```rust
use std::process::Command;

#[test]
fn test_end_to_end() {
    // Start dev server in background
    let mut server = Command::new("nexus")
        .args(&["dev"])
        .spawn()
        .expect("Failed to start server");
    
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // Send test event
    let output = Command::new("curl")
        .args(&[
            "-X", "POST",
            "http://localhost:8080/events/hello",
            "-d", r#"{"test": true}"#
        ])
        .output()
        .expect("Failed to execute curl");
    
    assert!(output.status.success());
    
    // Cleanup
    server.kill().expect("Failed to kill server");
}
```

## Multiple Functions in One Project

**nexus.yaml:**
```yaml
version: v1

functions:
  - name: user-signup
    on:
      http:
        method: POST
        path: /events/user.created
    runtime: wasi-preview1
    code: ./build/signup.wasm
    
  - name: user-notification
    on:
      http:
        method: POST
        path: /events/user.notify
    runtime: wasi-preview1
    code: ./build/notify.wasm
```

Build both:
```bash
cargo build --target wasm32-wasi --release --bin signup
cargo build --target wasm32-wasi --release --bin notify

cp target/wasm32-wasi/release/signup.wasm build/
cp target/wasm32-wasi/release/notify.wasm build/
```

## CLI Reference

### Core Commands

```bash
# Start development server
nexus dev

# Deploy to production (future)
nexus deploy --env=production

# Create new function template
nexus new <name> --lang=rust|assemblyscript

# Replay event by ID
nexus replay <event-id>

# View recent events
nexus events --limit=50

# View function logs
nexus logs <function-name>

# Get function info
nexus info <function-name>
```

### Development Flags

```bash
# Custom port
nexus dev --port=9000

# Custom config file
nexus dev --config=./custom.yaml

# Verbose logging
nexus dev --verbose

# Hot reload (watch mode)
nexus dev --watch
```

## Next Steps

- Read [ARCHITECTURE.md](./ARCHITECTURE.md) for system design
- Explore [examples/](../examples/) for more patterns
- Check [API_REFERENCE.md](./API_REFERENCE.md) for HTTP API docs
- See [DEPLOYMENT.md](./DEPLOYMENT.md) for production setup

## Troubleshooting

### Function Not Executing

1. Check WASM binary exists: `ls build/handler.wasm`
2. Verify nexus.yaml path matches: `code: ./build/handler.wasm`
3. Check logs for compilation errors

### NATS Connection Issues

```bash
# Check if NATS is running
nexus dev --verbose

# Look for: "✓ Starting embedded NATS JetStream..."
```

### WASM Compilation Errors

```bash
# Ensure WASM target is installed
rustup target add wasm32-wasi

# Check Rust version
rustc --version  # Should be 1.75+
```

## Community & Support

- **GitHub:** https://github.com/nexus-faas/core
- **Discord:** https://discord.gg/nexus-faas
- **Documentation:** https://docs.nexus.dev
- **Issues:** https://github.com/nexus-faas/core/issues

---

**Congratulations!** 🎉 You've deployed your first Nexus function. Ready to build something amazing?
