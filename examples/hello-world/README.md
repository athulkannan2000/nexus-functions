# Nexus Functions - Example: Hello World

A simple hello world function that demonstrates basic event processing.

## Building

```bash
cargo build --target wasm32-wasi --release
mkdir -p ../../build
cp target/wasm32-wasi/release/hello_world.wasm ../../build/handler.wasm
```

## Testing Locally

```bash
# Start the dev server (from project root)
cargo run -p nexus-cli -- dev

# Trigger the function
curl -X POST http://localhost:8080/events/hello \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello Nexus!", "name": "Alice"}'
```

## Expected Output

```json
{
  "event_id": "abc123-def456",
  "status": "published"
}
```
