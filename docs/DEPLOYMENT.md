# Deployment Guide

This guide covers various deployment options for Nexus Functions, from local development to production Kubernetes clusters.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Local Development](#local-development)
- [Docker Deployment](#docker-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Cloud Platforms](#cloud-platforms)
- [Production Checklist](#production-checklist)

## Prerequisites

### For all deployments
- NATS server (JetStream enabled) or use embedded NATS
- Rust 1.70+ (for building from source)

### For Docker deployment
- Docker 20.10+
- Docker Compose 2.0+

### For Kubernetes deployment
- Kubernetes cluster 1.24+
- kubectl configured
- NGINX Ingress Controller (recommended)
- cert-manager (optional, for TLS)

## Local Development

### Option 1: Native Binary

```powershell
# Install NATS Server
# Download from https://nats.io/download/

# Start NATS with JetStream
nats-server -js -c nats-server.conf

# Build and run Nexus Functions
cargo build --release
.\target\release\nexus.exe dev
```

### Option 2: Using the CLI

```powershell
# Install nexus CLI
cargo install --path cli

# Run development server
nexus dev
```

The server will start on `http://localhost:8080` with embedded NATS.

### Verify Installation

```powershell
# Check health
curl http://localhost:8080/health

# Check metrics
curl http://localhost:8080/metrics
```

## Docker Deployment

### Build Docker Image

```bash
# Build the image
docker build -t nexus-functions:latest .

# Or use docker compose to build
docker compose build
```

### Run with Docker Compose

#### Development Environment

```bash
# Start all services
docker compose -f docker-compose.dev.yml up -d

# View logs
docker compose -f docker-compose.dev.yml logs -f

# Stop services
docker compose -f docker-compose.dev.yml down
```

#### Production Environment

```bash
# Start with production config
docker compose up -d

# View logs
docker compose logs -f nexus

# Stop services
docker compose down

# Stop and remove volumes
docker compose down -v
```

### Configuration

Create a `nexus.yaml` file in the project root:

```yaml
version: v1
server:
  host: 0.0.0.0
  port: 8080

nats:
  url: nats://nats:4222
  stream: NEXUS_EVENTS
  
functions:
  directory: ./functions
```

### Docker Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (debug, info, warn, error) |
| `NEXUS_HOST` | `0.0.0.0` | Server bind address |
| `NEXUS_PORT` | `8080` | Server port |
| `NATS_URL` | `nats://localhost:4222` | NATS server URL |
| `NEXUS_FUNCTIONS_DIR` | `/app/functions` | WASM functions directory |

## Kubernetes Deployment

### Quick Start

```bash
# Deploy everything
kubectl apply -f k8s/

# Check status
kubectl get all -n nexus-functions

# Get service URL
kubectl get ingress -n nexus-functions
```

### Step-by-Step Deployment

#### 1. Create Namespace

```bash
kubectl apply -f k8s/namespace.yaml
```

#### 2. Configure Settings

Edit `k8s/configmap.yaml` for your environment:

```yaml
data:
  RUST_LOG: "info"
  NATS_URL: "nats://nats-service:4222"
  # Add custom configuration
```

Apply configuration:

```bash
kubectl apply -f k8s/configmap.yaml
```

#### 3. Deploy NATS and Nexus

```bash
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
```

#### 4. Configure Ingress

Update `k8s/ingress.yaml` with your domain:

```yaml
spec:
  rules:
  - host: nexus.your-domain.com
```

Apply ingress:

```bash
kubectl apply -f k8s/ingress.yaml
```

#### 5. Enable Auto-scaling (Optional)

```bash
kubectl apply -f k8s/hpa.yaml
```

### Monitoring

#### View Logs

```bash
# Nexus logs
kubectl logs -n nexus-functions -l app.kubernetes.io/name=nexus-functions -f

# NATS logs
kubectl logs -n nexus-functions -l app.kubernetes.io/name=nats -f
```

#### Port Forwarding

```bash
# Access Nexus API locally
kubectl port-forward -n nexus-functions svc/nexus-functions-service 8080:80

# Access NATS monitoring
kubectl port-forward -n nexus-functions svc/nats-service 8222:8222
```

#### Check Health

```bash
curl http://localhost:8080/health
curl http://localhost:8080/metrics
```

### Scaling

#### Manual Scaling

```bash
kubectl scale deployment nexus-functions -n nexus-functions --replicas=5
```

#### Auto-scaling

HPA automatically scales based on CPU/memory (see `k8s/hpa.yaml`):

```bash
# View HPA status
kubectl get hpa -n nexus-functions

# Describe HPA
kubectl describe hpa nexus-functions-hpa -n nexus-functions
```

### Updating

#### Rolling Update

```bash
# Update image
kubectl set image deployment/nexus-functions nexus=ghcr.io/athulkannan2000/nexus-functions:v2.0.0 -n nexus-functions

# Check rollout status
kubectl rollout status deployment/nexus-functions -n nexus-functions

# Rollback if needed
kubectl rollout undo deployment/nexus-functions -n nexus-functions
```

## Cloud Platforms

### AWS EKS

```bash
# Create cluster
eksctl create cluster --name nexus-cluster --region us-west-2

# Deploy
kubectl apply -f k8s/

# Get load balancer URL
kubectl get svc -n nexus-functions
```

### Azure AKS

```bash
# Create cluster
az aks create --resource-group nexus-rg --name nexus-cluster

# Get credentials
az aks get-credentials --resource-group nexus-rg --name nexus-cluster

# Deploy
kubectl apply -f k8s/
```

### Google GKE

```bash
# Create cluster
gcloud container clusters create nexus-cluster --zone us-central1-a

# Get credentials
gcloud container clusters get-credentials nexus-cluster --zone us-central1-a

# Deploy
kubectl apply -f k8s/
```

### DigitalOcean Kubernetes

```bash
# Create cluster via UI or doctl
doctl kubernetes cluster create nexus-cluster

# Deploy
kubectl apply -f k8s/
```

## Production Checklist

### Security

- [ ] Configure TLS/HTTPS with valid certificates
- [ ] Set up network policies
- [ ] Use secrets for sensitive configuration
- [ ] Enable authentication for NATS
- [ ] Configure RBAC for Kubernetes
- [ ] Scan Docker images for vulnerabilities
- [ ] Use non-root user in containers (already configured)
- [ ] Enable security contexts in pods

### Reliability

- [ ] Configure resource requests and limits
- [ ] Set up health checks (liveness, readiness, startup)
- [ ] Configure HorizontalPodAutoscaler
- [ ] Set up PodDisruptionBudgets
- [ ] Configure pod anti-affinity
- [ ] Implement retry logic in functions
- [ ] Set up multi-region deployment (if needed)

### Observability

- [ ] Set up centralized logging (ELK, Loki, CloudWatch)
- [ ] Configure metrics collection (Prometheus)
- [ ] Set up distributed tracing (Jaeger, Tempo)
- [ ] Create dashboards (Grafana)
- [ ] Configure alerting (AlertManager, PagerDuty)
- [ ] Monitor NATS JetStream metrics
- [ ] Set up log aggregation

### Data Management

- [ ] Configure persistent storage for NATS
- [ ] Set up backup strategy for NATS data
- [ ] Configure retention policies
- [ ] Plan for disaster recovery
- [ ] Test restore procedures

### Performance

- [ ] Enable WASM module caching
- [ ] Configure appropriate replica counts
- [ ] Optimize resource limits
- [ ] Set up CDN for static assets (if applicable)
- [ ] Configure connection pooling
- [ ] Enable compression
- [ ] Monitor cold start times

### CI/CD

- [ ] Set up automated testing
- [ ] Configure container image scanning
- [ ] Implement GitOps workflow
- [ ] Set up staging environment
- [ ] Configure blue-green or canary deployments
- [ ] Automate rollback procedures

### Documentation

- [ ] Document deployment procedures
- [ ] Create runbooks for common issues
- [ ] Document incident response procedures
- [ ] Maintain architecture diagrams
- [ ] Document API endpoints and usage

## Troubleshooting

### Docker Issues

#### Container won't start

```bash
# Check logs
docker logs nexus-functions

# Inspect container
docker inspect nexus-functions

# Check NATS connectivity
docker exec nexus-functions curl -f http://nats:8222/healthz
```

#### Build fails

```bash
# Clear Docker cache
docker builder prune

# Build with no cache
docker build --no-cache -t nexus-functions:latest .
```

### Kubernetes Issues

#### Pods not starting

```bash
# Describe pod
kubectl describe pod -n nexus-functions <pod-name>

# Check events
kubectl get events -n nexus-functions --sort-by='.lastTimestamp'

# Check logs
kubectl logs -n nexus-functions <pod-name> --previous
```

#### NATS connection issues

```bash
# Test NATS health
kubectl exec -n nexus-functions -it nats-0 -- wget -qO- http://localhost:8222/healthz

# Check service endpoints
kubectl get endpoints -n nexus-functions
```

#### Image pull errors

```bash
# Create image pull secret
kubectl create secret docker-registry regcred \
  --docker-server=ghcr.io \
  --docker-username=<username> \
  --docker-password=<token> \
  -n nexus-functions

# Add to deployment
kubectl patch deployment nexus-functions -n nexus-functions -p '{"spec":{"template":{"spec":{"imagePullSecrets":[{"name":"regcred"}]}}}}'
```

## Support

For issues and questions:
- GitHub Issues: https://github.com/athulkannan2000/nexus-functions/issues
- Documentation: https://github.com/athulkannan2000/nexus-functions/docs
