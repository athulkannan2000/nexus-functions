# Nexus Functions - Architecture Documentation

## Overview

Nexus Functions is a modern event-driven serverless platform built with replay-first principles, WASM-native execution, and zero-code observability.

## System Architecture

### High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Developer Interface                       │
│  ┌──────────┐  ┌──────────┐  ┌─────────────────────────┐   │
│  │   CLI    │  │ VS Code  │  │   Web Console (Future)  │   │
│  └──────────┘  └──────────┘  └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                     Control Plane                            │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐   │
│  │ API Gateway  │  │   Registry   │  │  Config Parser  │   │
│  │   (Axum)     │  │   (Future)   │  │  (nexus.yaml)   │   │
│  └──────────────┘  └──────────────┘  └─────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                      Event Fabric                            │
│  ┌────────────────────────────────────────────────────┐     │
│  │         NATS JetStream (Embedded/Clustered)        │     │
│  │  • CloudEvents Storage  • Replay Buffer  • DLQ    │     │
│  └────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                      Data Plane                              │
│  ┌──────────────────────┐  ┌────────────────────────────┐  │
│  │   WASM Runtime       │  │    Observability Layer     │  │
│  │   (Wasmtime)         │  │    (OpenTelemetry)         │  │
│  │  • Module Loader     │  │  • Trace Injection         │  │
│  │  • WASI Environment  │  │  • Structured Logging      │  │
│  │  • Execution Engine  │  │  • Metrics Collection      │  │
│  └──────────────────────┘  └────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. CLI (`cli/`)

**Purpose:** Developer interface for managing functions and local development

**Key Commands:**
- `nexus dev` - Start local development server
- `nexus deploy` - Deploy functions (future)
- `nexus replay <event-id>` - Replay events
- `nexus new --lang=rust` - Create new function template
- `nexus logs` - View function logs

**Technology:** Rust + Clap

### 2. Core (`core/`)

**Purpose:** Central orchestration and runtime management

**Modules:**
- `config/` - nexus.yaml parsing and validation
- `server.rs` - Axum HTTP server
- `executor.rs` - Function execution coordinator
- `replay_engine.rs` - Event replay logic

**Technology:** Rust + Tokio + Axum

### 3. Runtime (`runtime/`)

**Purpose:** WASM function execution environment

**Modules:**
- `wasm_loader.rs` - Load WASM modules from disk
- `wasm_executor.rs` - Execute WASM with Wasmtime
- `wasi_env.rs` - WASI preview1 environment setup

**Key Features:**
- Memory-safe execution (WASM sandbox)
- Sub-5ms cold starts
- Resource limits (CPU, memory, timeout)
- WASI preview1 support

**Technology:** Wasmtime + WASI

### 4. Event Fabric (`event-fabric/`)

**Purpose:** Event storage, routing, and replay

**Modules:**
- `embedded_nats.rs` - Embedded NATS server for local dev
- `cloudevents.rs` - CloudEvents format handling
- `publisher.rs` - Publish events to streams
- `replay.rs` - Event replay logic

**Key Features:**
- CloudEvents standard compliance
- Persistent event storage with retention policies
- Event replay by ID
- Dead Letter Queue (DLQ) for failures

**Technology:** NATS JetStream + CloudEvents

### 5. API Gateway (`api-gateway/`)

**Purpose:** HTTP ingestion and routing

**Modules:**
- `webhook.rs` - HTTP endpoint handlers
- `router.rs` - Route events to functions
- `auth.rs` - Authentication (future)

**Endpoints:**
- `POST /events/{type}` - Webhook ingestion
- `GET /events/{id}` - Event retrieval
- `POST /replay/{id}` - Manual replay trigger

**Technology:** Axum + Serde

### 6. Observability (`observability/`)

**Purpose:** Tracing, logging, and metrics

**Modules:**
- `tracing.rs` - OpenTelemetry integration
- `logger.rs` - Structured logging
- `metrics.rs` - Prometheus metrics (future)

**Key Features:**
- Auto-injected trace IDs
- Distributed tracing (future)
- Correlation: event ID → trace → logs
- Zero-SDK instrumentation (future: eBPF)

**Technology:** OpenTelemetry + Tokio Tracing

## Data Flow

### Event Ingestion Flow

```
1. HTTP Request
   POST /events/user.created
   └─> API Gateway (webhook.rs)

2. CloudEvent Creation
   └─> Convert to CloudEvents format
   └─> Generate event ID (UUID)
   └─> Add timestamp, source, type

3. Event Publishing
   └─> NATS JetStream publish
   └─> Stream: "events"
   └─> Subject: "events.user.created"

4. Event Persistence
   └─> JetStream stores with retention
   └─> ACK returned to publisher

5. HTTP Response
   └─> {"event_id": "abc123", "status": "published"}
```

### Function Execution Flow

```
1. Event Trigger
   └─> NATS subscriber receives event

2. Function Resolution
   └─> Match event type to nexus.yaml
   └─> Load WASM module path

3. Runtime Preparation
   └─> Create Wasmtime instance
   └─> Setup WASI environment
   └─> Inject TRACE_ID env var

4. Execution
   └─> Pass event payload via stdin
   └─> Call handle_event() function
   └─> Capture stdout/stderr

5. Observability
   └─> Log with trace context
   └─> Record metrics (duration, memory)
   └─> Emit OpenTelemetry spans

6. Result Handling
   └─> Success: log output
   └─> Failure: send to DLQ
```

### Replay Flow

```
1. CLI Command
   nexus replay abc123

2. Event Retrieval
   └─> Query NATS JetStream by event ID
   └─> Fetch original CloudEvent

3. Replay Metadata
   └─> Add "is_replay: true"
   └─> Add "original_timestamp"
   └─> Generate new trace ID

4. Re-execution
   └─> Follow standard execution flow
   └─> Use original event payload
   └─> Tag logs with [REPLAY]

5. Result
   └─> Display output to CLI
   └─> Store replay audit log
```

## Configuration Schema

### nexus.yaml Structure

```yaml
version: v1

functions:
  - name: user-welcome              # Function identifier
    on:                             # Trigger definition
      http:
        method: POST
        path: /events/user.created
    runtime: wasi-preview1          # WASM runtime version
    code: ./build/welcome.wasm      # Path to WASM binary
    timeout: 5s                     # Execution timeout
    memory: 128Mi                   # Memory limit
    env:                            # Environment variables
      LOG_LEVEL: info
```

### CloudEvents Format

```json
{
  "specversion": "1.0",
  "type": "com.example.user.created",
  "source": "/api/signup",
  "id": "abc123",
  "time": "2025-11-26T10:23:45Z",
  "datacontenttype": "application/json",
  "data": {
    "user_id": "u42",
    "email": "alice@example.com"
  }
}
```

## Scaling Considerations

### MVP (Single Node)
- Embedded NATS JetStream
- Single Axum process
- In-memory WASM module cache
- Target: 1000 req/s

### Production (Distributed)
- External NATS cluster (3+ nodes)
- Multiple API Gateway instances (load balanced)
- Shared WASM module registry
- Target: 10,000+ req/s

### Future (Multi-Region)
- NATS global replication
- Edge function deployment
- Function warm pools
- Target: 100,000+ req/s

## Security Model

### WASM Sandbox
- Memory isolation (no shared memory)
- No file system access (unless WASI-granted)
- No network access by default
- Capability-based security model

### Resource Limits
- CPU quota (future: cgroups)
- Memory limits (enforced by Wasmtime)
- Execution timeout (hard kill after timeout)
- Max payload size (configurable)

### Authentication (Future)
- JWT-based API authentication
- Function-level access control
- Secret injection via env vars
- Integration with Vault/Secrets Manager

## Observability Architecture

### Trace Context Propagation

```
HTTP Request [trace=abc123]
  └─> API Gateway [trace=abc123]
       └─> Event Publisher [trace=abc123]
            └─> NATS Message [trace=abc123]
                 └─> Function Executor [trace=abc123]
                      └─> WASM Function [TRACE_ID=abc123]
                           └─> Logs [trace=abc123]
```

### Metrics Collection (Future)

```
Prometheus Exporter
├─> nexus_events_total (counter)
├─> nexus_function_duration_seconds (histogram)
├─> nexus_function_memory_bytes (gauge)
├─> nexus_replay_count (counter)
└─> nexus_errors_total (counter by type)
```

## Extension Points

### Custom Event Sources (Future)
- Kafka consumer adapter
- AWS SQS poller
- Database CDC (Change Data Capture)
- MQTT broker integration

### Custom Runtimes (Future)
- Container runtime (gVisor)
- Node.js runtime (via WASM-JS)
- Python runtime (via RustPython)

### Policy Engine (Future)
- OPA/Rego policy evaluation
- Cost limits per tenant
- Geographic restrictions
- Compliance rules

## Performance Targets

| Metric | MVP | Production |
|--------|-----|------------|
| Cold Start | <5ms | <3ms |
| Hot Execution | <1ms | <500μs |
| Event Latency | <10ms | <5ms |
| Throughput | 1K req/s | 10K req/s |
| Memory Usage | <50MB | <200MB |

## Technology Decisions

### Why Rust?
- Memory safety without garbage collection
- Excellent WASM toolchain support
- High performance (C++ level)
- Strong async ecosystem (Tokio)

### Why NATS JetStream?
- Lightweight (embeddable for local dev)
- Built-in persistence and replay
- Cloud-native (Kubernetes-friendly)
- High throughput, low latency

### Why Wasmtime?
- Production-grade WASM runtime
- Maintained by Bytecode Alliance
- Excellent Rust integration
- WASI preview1 support

### Why Axum?
- Built on Tokio (async-native)
- Type-safe routing
- Minimal overhead
- Composable middleware

## References

- **WASM Spec:** https://webassembly.org/
- **WASI:** https://wasi.dev/
- **CloudEvents:** https://cloudevents.io/
- **OpenTelemetry:** https://opentelemetry.io/
- **NATS JetStream:** https://docs.nats.io/nats-concepts/jetstream

---

**Last Updated:** 2025-11-26  
**Status:** Living Document - Updated with MVP implementation
