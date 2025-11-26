# Nexus Functions Roadmap

## Vision

Build the most developer-friendly event-driven serverless platform with replay-first architecture, sub-millisecond cold starts, and zero-code observability.

---

## Phase 1: MVP - Local Development (Week 1) 🚧 **IN PROGRESS**

**Goal:** Prove the core concept works end-to-end locally

### Week 1 Deliverables

#### Day 1-2: Foundation ✅ **COMPLETED**
- [x] Project structure and planning
- [x] Architecture documentation
- [ ] Rust workspace setup
- [ ] CLI skeleton with Clap
- [ ] Embedded NATS server

#### Day 3-4: Core Functionality 🔄 **NEXT**
- [ ] HTTP API Gateway (Axum)
- [ ] CloudEvents ingestion
- [ ] NATS JetStream publishing
- [ ] WASM loader (Wasmtime)
- [ ] Function execution

#### Day 5-7: Developer Experience
- [ ] Auto-instrumentation (trace IDs)
- [ ] Event replay command
- [ ] `nexus new` templates
- [ ] Local development workflow
- [ ] Demo recording

### Success Criteria
- ✅ `nexus dev` starts in <5 seconds
- ✅ Deploy WASM function via HTTP
- ✅ Replay event by ID
- ✅ Trace IDs in all logs
- ✅ 90-second end-to-end demo

---

## Phase 2: Production Readiness (Week 2-4)

**Goal:** Make it production-worthy for single-node deployment

### Week 2: Advanced Runtime
- [ ] AssemblyScript runtime support
- [ ] Function warm pool (reduce cold starts)
- [ ] Concurrent execution with worker threads
- [ ] Function versioning
- [ ] Resource quotas (CPU, memory)

### Week 3: Multi-Tenancy & Security
- [ ] JWT authentication
- [ ] Namespace isolation
- [ ] Rate limiting per tenant
- [ ] Secret injection (env vars)
- [ ] HTTPS support

### Week 4: Extended Event Sources
- [ ] Kafka consumer adapter
- [ ] AWS SQS poller
- [ ] Cron/scheduled triggers
- [ ] Dead Letter Queue (DLQ)
- [ ] Event filtering rules

### Deliverables
- [ ] Docker Compose deployment
- [ ] External NATS cluster support
- [ ] Production deployment guide
- [ ] Performance benchmarks

---

## Phase 3: Distributed System (Month 2-3)

**Goal:** Scale to multi-node, multi-region

### Core Infrastructure
- [ ] Kubernetes Helm charts
- [ ] NATS global replication
- [ ] Distributed tracing (Jaeger)
- [ ] Prometheus metrics exporter
- [ ] Multi-region deployment

### Container Support
- [ ] gVisor runtime integration
- [ ] Node.js runtime (via WASM-JS)
- [ ] Python runtime (RustPython)
- [ ] Docker image registry

### Observability
- [ ] eBPF instrumentation (zero-code)
- [ ] Correlation engine (event → trace → logs)
- [ ] CPU/memory profiling
- [ ] Cost attribution per tenant
- [ ] Alert rules (PagerDuty, Slack)

### Deliverables
- [ ] Kubernetes operator
- [ ] Multi-cloud deployment guide
- [ ] Observability dashboard
- [ ] SLA monitoring

---

## Phase 4: Enterprise Platform (Month 4-6)

**Goal:** Enterprise-grade multi-cloud platform

### Policy & Compliance
- [ ] Policy engine (OPA/Rego)
- [ ] Cost simulator and limits
- [ ] Geographic restrictions
- [ ] Compliance rules (GDPR, HIPAA)
- [ ] Audit logging

### Developer Tools
- [ ] VS Code extension
- [ ] Web console (function designer)
- [ ] Flow designer (visual workflows)
- [ ] Cost calculator
- [ ] Function marketplace

### Advanced Features
- [ ] Workflow engine (Temporal integration)
- [ ] Event replay at scale (batch replay)
- [ ] Time-travel debugging
- [ ] Blue/green deployments
- [ ] Canary releases

### Multi-Cloud
- [ ] AWS EKS deployment
- [ ] Azure AKS deployment
- [ ] GCP GKE deployment
- [ ] Edge deployment (Fly.io, Cloudflare)
- [ ] Hybrid cloud (on-prem + cloud)

### Deliverables
- [ ] Enterprise documentation
- [ ] Migration tools (from Lambda/Cloud Functions)
- [ ] Professional support offering
- [ ] Managed service (SaaS)

---

## Phase 5: Ecosystem & Community (Month 7-12)

**Goal:** Build a thriving ecosystem

### SDKs & Libraries
- [ ] Rust SDK
- [ ] JavaScript/TypeScript SDK
- [ ] Python SDK
- [ ] Go SDK
- [ ] Function framework (Spring-like)

### Integrations
- [ ] GitHub Actions
- [ ] GitLab CI/CD
- [ ] Jenkins plugin
- [ ] Terraform provider
- [ ] Pulumi provider

### Ecosystem
- [ ] Public function registry
- [ ] Template marketplace
- [ ] Plugin system
- [ ] Custom runtime support
- [ ] Community extensions

### Community
- [ ] Documentation portal
- [ ] Video tutorials
- [ ] Blog and use cases
- [ ] Certification program
- [ ] Annual conference

---

## Feature Requests & Voting

Vote on features at: **[github.com/nexus-faas/core/discussions](https://github.com/nexus-faas/core/discussions)**

### Top Community Requests
1. 🔥 Python runtime support (120 votes)
2. 🔥 GitHub Actions integration (95 votes)
3. 🔥 Web UI console (87 votes)
4. Database CDC triggers (64 votes)
5. GraphQL API support (52 votes)

---

## Performance Goals

| Metric | MVP | Month 3 | Month 6 |
|--------|-----|---------|---------|
| Cold Start | 5ms | 3ms | 1ms |
| Hot Execution | 1ms | 500μs | 100μs |
| Throughput | 1K/s | 10K/s | 100K/s |
| Latency P99 | 10ms | 5ms | 2ms |
| Max Functions | 100 | 10K | 1M |

---

## Open Questions

### Technical Decisions Needed
- [ ] License: Stay Elastic v2 or dual-license SDKs as Apache 2.0?
- [ ] Multi-region: Active-active or active-passive replication?
- [ ] Container runtime: Support Docker or only gVisor?
- [ ] Pricing model: Free tier limits? Usage-based?

### Community Input Needed
- Which cloud provider should we prioritize? (AWS/Azure/GCP)
- Which language runtime is most important? (Node/Python/Go)
- Managed service or self-hosted focus?
- Open-source governance model?

**Discuss at:** [github.com/nexus-faas/core/discussions](https://github.com/nexus-faas/core/discussions)

---

## Release Schedule

### v0.1.0 - MVP (Week 1)
- Local development only
- WASM runtime
- Basic replay

### v0.2.0 - Alpha (Week 4)
- Multi-tenant support
- External NATS
- Docker Compose deployment

### v0.3.0 - Beta (Month 3)
- Kubernetes deployment
- Container runtime
- Distributed tracing

### v1.0.0 - GA (Month 6)
- Production-ready
- Multi-cloud support
- Enterprise features
- Professional support

---

## How to Contribute

See which features you can help with:
1. Check [Issues](https://github.com/nexus-faas/core/issues) labeled `help wanted`
2. Comment on roadmap items you want to work on
3. Submit RFC for major features
4. Join our [Discord](https://discord.gg/nexus-faas) to discuss

---

## Changelog

Track releases at: [CHANGELOG.md](./CHANGELOG.md)

---

**Last Updated:** 2025-11-26  
**Current Phase:** Phase 1 - MVP Development (Week 1, Day 1)

*This roadmap is a living document and will evolve based on community feedback.*
