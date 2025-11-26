# Nexus Functions - Development Project Plan

## Executive Summary

**Project Name:** Nexus Functions  
**Project Type:** Event-Driven Serverless Platform (FaaS)  
**Development Phase:** MVP (Minimum Viable Product)  
**Timeline:** 7-Day Sprint + 4-Week Extended Development  
**Primary Language:** Rust  
**License Strategy:** Elastic License v2 (Source-Available)

## Project Vision

Build a next-generation event-driven serverless platform that solves the core pain points of existing FaaS solutions:
- **Replay-first architecture** - Events as the source of truth
- **Sub-5ms cold starts** - WASM-first runtime with container fallback
- **Zero-code observability** - eBPF instrumentation without SDKs
- **Hybrid deployment** - Same code runs on AWS, Azure, GCP, on-prem, or edge
- **Developer-first experience** - Simple CLI, declarative config, local-first development

---

## 📋 Project Structure

```
path/folder/
├── core/                    # Core runtime engine and orchestration
├── cli/                     # Command-line interface (nexus CLI)
├── runtime/                 # WASM runtime (Wasmtime) integration
├── event-fabric/            # NATS JetStream event bus
├── api-gateway/             # HTTP ingestion and routing
├── observability/           # OpenTelemetry integration
├── examples/                # Sample functions and templates
├── docs/                    # Technical documentation
├── tests/                   # Integration and unit tests
└── infra/                   # Docker Compose and deployment configs
```

---

## 🎯 MVP Success Criteria

**Core Goal:** From zero to deployed function in under 5 minutes

1. ✅ Record an HTTP event to NATS JetStream
2. ✅ Deploy a Rust WASM function locally
3. ✅ Trigger the function via HTTP webhook
4. ✅ Auto-inject trace ID into logs
5. ✅ Replay the same event with `nexus replay <event-id>`

**Target Metric:** 90-second end-to-end demo from install to replay

---

## 📅 7-Day MVP Sprint Plan

### **Day 1: Foundation & Local Development Server**
**Goal:** Scaffold CLI and embedded development environment

**Tasks:**
- [ ] Initialize Rust workspace with Cargo
- [ ] Create `nexus-cli` crate with Clap argument parsing
- [ ] Implement `nexus dev` command
- [ ] Embed NATS JetStream server in development mode
- [ ] Create Axum HTTP server with basic routing
- [ ] Add configuration management (TOML/YAML)

**Deliverable:** `nexus dev` starts a local server on `localhost:8080` with embedded NATS

**Files:**
```
cli/src/main.rs
cli/src/commands/dev.rs
core/src/server.rs
event-fabric/src/embedded_nats.rs
```

---

### **Day 2: Configuration Parser**
**Goal:** Parse and validate `nexus.yaml` specification

**Tasks:**
- [ ] Define `nexus.yaml` schema (Rust structs with Serde)
- [ ] Implement YAML parser for function definitions
- [ ] Add validation for `on:` triggers (HTTP, NATS)
- [ ] Add validation for `runtime:` and `code:` paths
- [ ] Create error handling for malformed configs
- [ ] Add hot-reload watching for config changes

**Deliverable:** CLI loads and validates `nexus.yaml` with clear error messages

**Files:**
```
core/src/config/mod.rs
core/src/config/schema.rs
core/src/config/validator.rs
```

**Sample nexus.yaml:**
```yaml
version: v1
functions:
  - name: user-welcome
    on:
      http:
        method: POST
        path: /events/user.created
    runtime: wasi-preview1
    code: ./build/welcome.wasm
    timeout: 5s
    memory: 128Mi
```

---

### **Day 3: Event Ingestion Pipeline**
**Goal:** HTTP → CloudEvents → NATS JetStream

**Tasks:**
- [ ] Implement HTTP webhook endpoint handler
- [ ] Convert HTTP requests to CloudEvents format
- [ ] Add event ID generation (UUID or ULID)
- [ ] Publish CloudEvents to NATS JetStream stream
- [ ] Implement event persistence with retention policy
- [ ] Add basic HTTP response handling

**Deliverable:** `POST /events/user.created` publishes to NATS and returns event ID

**Files:**
```
api-gateway/src/webhook.rs
event-fabric/src/cloudevents.rs
event-fabric/src/publisher.rs
```

**Test:**
```bash
curl -X POST http://localhost:8080/events/user.created \
  -H "Content-Type: application/json" \
  -d '{"user_id": "u42", "email": "alice@example.com"}'

# Response: {"event_id": "abc123", "status": "published"}
```

---

### **Day 4: WASM Loader & Executor**
**Goal:** Load and execute WASM functions

**Tasks:**
- [ ] Integrate Wasmtime runtime engine
- [ ] Implement WASM module loader from file path
- [ ] Create WASI preview1 runtime environment
- [ ] Pass CloudEvent bytes to WASM via stdin
- [ ] Capture WASM stdout/stderr
- [ ] Add timeout and memory limit enforcement
- [ ] Handle WASM execution errors gracefully

**Deliverable:** Load `handler.wasm` and invoke with event payload

**Files:**
```
runtime/src/wasm_loader.rs
runtime/src/wasm_executor.rs
runtime/src/wasi_env.rs
```

**Test Function (Rust):**
```rust
// examples/rust-handler/src/lib.rs
use std::io::{stdin, stdout, Read, Write};

#[no_mangle]
pub extern "C" fn handle_event() {
    let mut input = Vec::new();
    stdin().read_to_end(&mut input).unwrap();
    
    let response = format!("Processed: {}", String::from_utf8_lossy(&input));
    stdout().write_all(response.as_bytes()).unwrap();
}
```

---

### **Day 5: Observability Foundation**
**Goal:** Auto-inject trace IDs and structured logging

**Tasks:**
- [ ] Integrate OpenTelemetry Rust SDK
- [ ] Generate trace ID for each event
- [ ] Inject `TRACE_ID` environment variable into WASM
- [ ] Capture and format function logs with trace context
- [ ] Implement structured logging (JSON format)
- [ ] Add basic metrics (execution time, memory usage)

**Deliverable:** Logs show `[trace=abc123] Processing user event`

**Files:**
```
observability/src/tracing.rs
observability/src/logger.rs
core/src/execution/instrumented.rs
```

**Log Output:**
```
[2025-11-26T10:23:45Z INFO nexus] [trace=abc123] Function 'user-welcome' started
[2025-11-26T10:23:45Z INFO user-welcome] [trace=abc123] Processing user event
[2025-11-26T10:23:45Z INFO nexus] [trace=abc123] Function completed in 1.2ms
```

---

### **Day 6: Event Replay System**
**Goal:** Persist events and replay on-demand

**Tasks:**
- [ ] Implement event storage in JetStream with retention
- [ ] Create `nexus replay <event-id>` command
- [ ] Fetch original event from NATS by event ID
- [ ] Re-trigger function with identical payload
- [ ] Add replay metadata (is_replay: true, original_timestamp)
- [ ] Handle replay failures with error reporting

**Deliverable:** `nexus replay abc123` re-executes the function

**Files:**
```
cli/src/commands/replay.rs
event-fabric/src/replay.rs
core/src/replay_engine.rs
```

**Test:**
```bash
# Original execution
curl -X POST http://localhost:8080/events/user.created \
  -d '{"user_id": "u42"}'
# Response: {"event_id": "abc123"}

# Replay
nexus replay abc123
# Output: 
# OK Replaying event abc123
# [trace=abc123-replay] Processing user event
# OK Replay completed in 0.8ms
```

---

### **Day 7: Integration Demo & Polish**
**Goal:** End-to-end demo and documentation

**Tasks:**
- [ ] Create example project template
- [ ] Implement `nexus new --lang=rust` scaffolding
- [ ] Write quickstart documentation
- [ ] Record 90-second demo video
- [ ] Add CLI help text and usage examples
- [ ] Package binaries for Windows/Linux/macOS

**Deliverable:** Complete demo from `nexus new` to `nexus replay`

**Demo Script:**
```bash
# 1. Create new project
nexus new my-app --lang=rust
cd my-app

# 2. Start dev server
nexus dev

# 3. Deploy function (in another terminal)
curl -X POST http://localhost:8080/events/user.created \
  -d '{"user_id": "u42", "email": "alice@example.com"}'

# 4. Check logs (trace ID visible)

# 5. Replay event
nexus replay <event-id>

# Total time: < 5 minutes
```

---

## 🗓️ Extended Development Plan (Week 2-4)

### **Week 2: Advanced Runtime & Performance**
- [ ] Add AssemblyScript runtime support
- [ ] Implement function warm pool (reduce cold starts)
- [ ] Add concurrent execution with worker pool
- [ ] Implement function versioning
- [ ] Add resource limits (CPU throttling)

### **Week 3: Production Features**
- [ ] Multi-tenancy support (namespace isolation)
- [ ] Authentication & authorization (JWT)
- [ ] Rate limiting and quota management
- [ ] Add Kafka/SQS event source adapters
- [ ] Implement DLQ (Dead Letter Queue)

### **Week 4: Observability & Ops**
- [ ] Full OpenTelemetry Collector integration
- [ ] Distributed tracing (span propagation)
- [ ] Metrics dashboard (Prometheus exporter)
- [ ] Function profiling (CPU/memory flamegraphs)
- [ ] Alert rules and notifications

---

## 🛠️ Technology Stack

| Component | Technology | Justification |
|-----------|-----------|---------------|
| **Backend** | Rust | Performance, safety, WASM ecosystem |
| **Event Bus** | NATS JetStream | Lightweight, replayable, embeddable |
| **WASM Runtime** | Wasmtime | Production-grade, Rust-native |
| **HTTP Server** | Axum | Async, composable, minimal overhead |
| **CLI** | Clap | Ergonomic, fast, comprehensive |
| **Config** | Serde + YAML | Type-safe parsing, validation |
| **Observability** | OpenTelemetry | Vendor-neutral, standards-based |
| **Tracing** | Tokio Tracing | Async-aware, low overhead |
| **Testing** | Tokio Test + Cargo | Native Rust testing framework |

---

## 📦 Dependencies (Cargo.toml)

```toml
[workspace]
members = ["cli", "core", "runtime", "event-fabric", "api-gateway", "observability"]

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
clap = { version = "4.4", features = ["derive"] }
wasmtime = "17.0"
async-nats = "0.33"
opentelemetry = "0.21"
uuid = { version = "1.6", features = ["v4"] }
tracing = "0.1"
anyhow = "1.0"
```

---

## 🧪 Testing Strategy

### Unit Tests
- Config parser validation
- CloudEvents serialization
- WASM module loading
- Event replay logic

### Integration Tests
- End-to-end HTTP → WASM execution
- Event persistence and retrieval
- CLI command execution
- Multi-function orchestration

### Performance Tests
- Cold start latency (target: <5ms)
- Concurrent execution (1000 req/s)
- Memory footprint (<50MB base)
- Event replay speed

---

## 📊 Key Performance Indicators (KPIs)

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cold Start | <5ms | WASM initialization time |
| Hot Execution | <1ms | Function execution time |
| Event Latency | <10ms | HTTP → NATS → Function |
| Replay Speed | <50ms | Event fetch + re-execution |
| Memory Usage | <50MB | Base runtime footprint |
| Throughput | >1000 req/s | Local single-node |
| Binary Size | <20MB | Compiled CLI executable |

---

## 🚀 Deployment Roadmap

### Phase 1: Local Development (MVP - Week 1)
- ✅ Local NATS JetStream
- ✅ Single-node Axum server
- ✅ WASM-only runtime

### Phase 2: Single-Node Production (Week 2-4)
- Docker Compose deployment
- External NATS cluster
- Multi-function support
- Basic auth

### Phase 3: Distributed System (Month 2-3)
- Kubernetes deployment
- Multi-region NATS replication
- Container runtime support (gVisor)
- eBPF instrumentation

### Phase 4: Enterprise Platform (Month 4-6)
- Multi-cloud support (AWS, Azure, GCP)
- Policy engine (OPA/Rego)
- Cost simulator
- Web console UI

---

## 🔐 Security Considerations

1. **Sandbox Isolation:** WASM provides memory-safe execution
2. **Resource Limits:** CPU/memory quotas per function
3. **Network Isolation:** No network access by default (WASI capability model)
4. **Secrets Management:** Environment variable injection (future: Vault integration)
5. **Audit Logging:** All events and replays tracked with trace IDs

---

## 📚 Documentation Plan

### Developer Docs (`docs/`)
- [ ] `ARCHITECTURE.md` - System design and component interaction
- [ ] `GETTING_STARTED.md` - Quickstart guide
- [ ] `API_REFERENCE.md` - HTTP and CLI API docs
- [ ] `WRITING_FUNCTIONS.md` - Function authoring guide
- [ ] `DEPLOYMENT.md` - Production deployment guide
- [ ] `TROUBLESHOOTING.md` - Common issues and solutions

### Community Docs
- [ ] `README.md` - Project overview and installation
- [ ] `CONTRIBUTING.md` - Contribution guidelines
- [ ] `CODE_OF_CONDUCT.md` - Community standards
- [ ] `ROADMAP.md` - Public feature roadmap
- [ ] `CHANGELOG.md` - Version history

---

## 🎯 Next Immediate Actions

1. **Day 1 Kickoff (Today):**
   - Initialize Cargo workspace
   - Set up repository structure
   - Create `nexus-cli` and `nexus-core` crates
   - Start embedded NATS server implementation

2. **Repository Setup:**
   - Initialize Git repository
   - Create `.gitignore` for Rust
   - Set up GitHub Actions CI/CD
   - Add license file (Elastic v2)

3. **Team Alignment:**
   - Confirm Day 1-7 sprint commitments
   - Set up daily standup schedule
   - Define demo success criteria
   - Assign component ownership

---

## 📝 Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2025-11-26 | Use Rust for backend | Performance, safety, WASM ecosystem |
| 2025-11-26 | NATS JetStream as event bus | Lightweight, replayable, embeddable |
| 2025-11-26 | WASM-only for MVP | Fastest path to cold-start advantage |
| 2025-11-26 | Elastic v2 license | Source-available, discourages SaaS cloning |
| 2025-11-26 | Local-first development | Zero cloud cost, faster iteration |

---

## ✅ Success Metrics

**MVP Complete When:**
- [ ] `nexus dev` starts local server in <5 seconds
- [ ] Sample WASM function executes in <5ms
- [ ] Event replay works 100% reliably
- [ ] Trace IDs appear in all logs
- [ ] Complete demo runs in <90 seconds
- [ ] Documentation covers all MVP features
- [ ] At least 5 example functions in `/examples`

**Demo Day Requirements:**
- Live coding session: `nexus new` → deploy → trigger → replay
- Performance metrics displayed in real-time
- Comparison slide with AWS Lambda/Google Cloud Functions
- Open-source announcement ready (README, LICENSE, CONTRIBUTING)

---

## 🔗 References

- **Project Scope:** `Project_scope.md`
- **NATS Documentation:** https://docs.nats.io/
- **Wasmtime Guide:** https://docs.wasmtime.dev/
- **OpenTelemetry Spec:** https://opentelemetry.io/docs/
- **CloudEvents Spec:** https://cloudevents.io/

---

**Last Updated:** 2025-11-26  
**Status:** Planning Complete ✅ - Ready to Execute Day 1
