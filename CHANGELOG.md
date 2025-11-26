# Nexus Functions - Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planning Phase - 2025-11-26

#### Added
- 📋 Initial project planning and architecture documentation
- 📁 Project structure with modular crates
- 📝 Comprehensive documentation (ARCHITECTURE, GETTING_STARTED, ROADMAP)
- 🎯 7-day MVP sprint plan
- 🏗️ Development folder structure at `path/folder/`

#### Components Scaffolded
- `cli/` - Command-line interface
- `core/` - Core orchestration engine
- `runtime/` - WASM runtime integration
- `event-fabric/` - NATS JetStream event bus
- `api-gateway/` - HTTP ingestion layer
- `observability/` - OpenTelemetry integration
- `examples/` - Sample functions
- `docs/` - Technical documentation
- `tests/` - Integration tests
- `infra/` - Infrastructure configs

---

## [0.1.0] - TBD (Week 1 Target)

### MVP Features (Planned)

#### Added
- ✅ Local development server (`nexus dev`)
- ✅ Embedded NATS JetStream for local development
- ✅ HTTP webhook ingestion
- ✅ CloudEvents format support
- ✅ WASM function loader (Wasmtime)
- ✅ WASI preview1 runtime environment
- ✅ Auto-injected trace IDs
- ✅ Event replay (`nexus replay <event-id>`)
- ✅ Function templates (`nexus new --lang=rust`)
- ✅ Structured logging with trace context

#### Developer Experience
- CLI with `dev`, `replay`, `new` commands
- YAML configuration (`nexus.yaml`)
- Hot-reload for config changes
- Clear error messages
- 90-second quickstart guide

#### Performance Targets
- Cold start: <5ms
- Hot execution: <1ms
- Event latency: <10ms
- Throughput: 1,000 req/s (single node)

---

## [0.2.0] - TBD (Week 4 Target)

### Production Features (Planned)

#### Added
- Multi-tenancy support (namespace isolation)
- JWT authentication
- Rate limiting and quotas
- External NATS cluster support
- Docker Compose deployment
- AssemblyScript runtime support
- Function versioning
- Dead Letter Queue (DLQ)
- Kafka event source adapter
- AWS SQS event source adapter

#### Improvements
- Function warm pool (reduced cold starts)
- Concurrent execution with worker threads
- Enhanced observability (metrics, spans)
- Production deployment guide
- Performance benchmarks

---

## [0.3.0] - TBD (Month 3 Target)

### Distributed System (Planned)

#### Added
- Kubernetes Helm charts
- Kubernetes operator
- NATS global replication
- Distributed tracing (Jaeger integration)
- Prometheus metrics exporter
- Container runtime support (gVisor)
- Node.js runtime (via WASM-JS)
- Python runtime (RustPython)
- eBPF instrumentation
- Multi-region deployment support

#### Improvements
- Horizontal scaling
- Multi-cloud deployment guides
- Advanced observability dashboard
- Cost attribution per tenant

---

## [1.0.0] - TBD (Month 6 Target)

### GA Release (Planned)

#### Added
- Policy engine (OPA/Rego)
- Cost simulator and limits
- Geographic restrictions
- Compliance rules (GDPR, HIPAA)
- Audit logging
- VS Code extension
- Web console (function designer)
- Flow designer (visual workflows)
- AWS EKS deployment
- Azure AKS deployment
- GCP GKE deployment
- Edge deployment (Fly.io, Cloudflare)
- Hybrid cloud support

#### Enterprise Features
- Professional support
- Migration tools (from AWS Lambda)
- SLA monitoring
- Blue/green deployments
- Canary releases

---

## Version History

### Versioning Strategy

- **0.1.x** - MVP and local development
- **0.2.x** - Single-node production features
- **0.3.x** - Distributed system capabilities
- **0.9.x** - Release candidates
- **1.0.0** - General availability
- **1.x.x** - Stable API, production-ready

### Release Cadence

- **Weekly** during MVP phase (v0.1.x)
- **Bi-weekly** during alpha/beta (v0.2.x - v0.9.x)
- **Monthly** after GA (v1.x.x)
- **Hotfixes** as needed

---

## Breaking Changes Policy

We follow semantic versioning:

- **Major** (x.0.0) - Breaking API changes
- **Minor** (0.x.0) - New features, backward compatible
- **Patch** (0.0.x) - Bug fixes, backward compatible

Pre-1.0 releases may have breaking changes in minor versions.

---

## Migration Guides

Migration guides will be published for each major version:
- [Migrating to v1.0.0](./docs/migrations/v1.0.0.md) (future)

---

## Contributors

See [GitHub Contributors](https://github.com/nexus-faas/core/graphs/contributors)

---

**Last Updated:** 2025-11-26  
**Current Version:** Unreleased (Planning Phase)
