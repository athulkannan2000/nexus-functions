# Testing Guide for Nexus Functions

This guide provides step-by-step instructions for testing all Day 7 deployment infrastructure once Docker and Kubernetes are installed.

## Prerequisites

### Required Software
- **Docker Desktop** 20.10+ or Docker Engine
- **Docker Compose** 2.0+ (included with Docker Desktop)
- **kubectl** (for Kubernetes testing)
- **minikube** or cloud K8s cluster (optional, for K8s testing)

### Installation Links
- Docker Desktop: https://www.docker.com/products/docker-desktop/
- kubectl: https://kubernetes.io/docs/tasks/tools/
- minikube: https://minikube.sigs.k8s.io/docs/start/

## Test 1: Docker Build

### Objective
Verify that the multi-stage Dockerfile builds successfully and creates a functional image.

### Steps

```powershell
# Navigate to project root
cd f:\Infinity\02_Work\01_Projects\Nexus-Functions\path\folder

# Build the Docker image
docker build -t nexus-functions:test .

# Expected output:
# - Successfully downloads Rust base image
# - Installs WASM targets
# - Builds dependencies (cached layer)
# - Compiles Nexus Functions
# - Creates runtime image (~200MB)
# - Build completes without errors
```

### Validation

```powershell
# Verify image exists
docker images | Select-String "nexus-functions"

# Check image size (should be ~150-250MB)
docker inspect nexus-functions:test --format='{{.Size}}' | ForEach-Object {[math]::Round($_ / 1MB, 2)}

# Inspect image layers
docker history nexus-functions:test
```

### Expected Results
- ✅ Image builds successfully
- ✅ Image size is ~200MB (not 2GB+)
- ✅ No security warnings
- ✅ Build completes in 5-15 minutes (first build)
- ✅ Subsequent builds use cached layers

### Common Issues

**Issue:** Build fails with "COPY failed"
```powershell
# Solution: Ensure you're in the correct directory
cd f:\Infinity\02_Work\01_Projects\Nexus-Functions\path\folder
```

**Issue:** Out of disk space
```powershell
# Solution: Clean Docker cache
docker system prune -a
```

## Test 2: Docker Compose - Development

### Objective
Test the development Docker Compose configuration with hot-reload support.

### Steps

```powershell
# Start services
docker compose -f docker-compose.dev.yml up -d

# Expected output:
# - Creating network "nexus-network"
# - Creating container "nexus-nats-dev"
# - Creating container "nexus-functions-dev"
# - Both containers healthy
```

### Validation

```powershell
# Check container status
docker compose -f docker-compose.dev.yml ps

# View logs
docker compose -f docker-compose.dev.yml logs nexus

# Test health endpoint
curl http://localhost:8080/health

# Expected response:
# {
#   "status": "healthy",
#   "version": "0.1.0",
#   "nats_connected": true,
#   "uptime_seconds": 10
# }

# Test metrics endpoint
curl http://localhost:8080/metrics

# Test NATS monitoring
curl http://localhost:8222/varz
```

### Cleanup

```powershell
# Stop services
docker compose -f docker-compose.dev.yml down

# Remove volumes (optional)
docker compose -f docker-compose.dev.yml down -v
```

### Expected Results
- ✅ Both containers start successfully
- ✅ Health checks pass
- ✅ NATS connection established
- ✅ HTTP endpoints respond
- ✅ Logs show INFO level messages

## Test 3: Docker Compose - Production

### Objective
Test the production Docker Compose configuration with persistent storage.

### Steps

```powershell
# Create functions directory if it doesn't exist
if (!(Test-Path ".\functions")) { New-Item -ItemType Directory -Path ".\functions" }

# Start services
docker compose up -d

# Wait for services to be healthy
Start-Sleep -Seconds 10
```

### Validation

```powershell
# Check container status
docker compose ps

# Verify volumes
docker volume ls | Select-String "nexus"

# Test health
curl http://localhost:8080/health

# Publish test event
$body = @{
    specversion = "1.0"
    type = "com.example.test"
    source = "/test"
    id = "test-123"
    data = @{ message = "Hello Nexus" }
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://localhost:8080/events" -Method POST -Body $body -ContentType "application/json"

# Check metrics
curl http://localhost:8080/metrics
```

### Persistence Test

```powershell
# Stop services
docker compose down

# Restart (data should persist)
docker compose up -d
Start-Sleep -Seconds 10

# Verify events are still there
curl http://localhost:8080/events
```

### Cleanup

```powershell
# Stop and remove everything including volumes
docker compose down -v
```

### Expected Results
- ✅ Persistent volume created
- ✅ Services restart successfully
- ✅ Data persists across restarts
- ✅ Production logging (INFO level)

## Test 4: Kubernetes Manifests Validation

### Objective
Validate that all Kubernetes manifests are syntactically correct.

### Steps

```powershell
# Validate all manifests (dry-run)
kubectl apply -f k8s/ --dry-run=client

# Expected output: No errors

# Validate with server-side dry-run (requires cluster)
kubectl apply -f k8s/ --dry-run=server

# Check individual resources
kubectl apply -f k8s/namespace.yaml --dry-run=client
kubectl apply -f k8s/configmap.yaml --dry-run=client
kubectl apply -f k8s/deployment.yaml --dry-run=client
kubectl apply -f k8s/service.yaml --dry-run=client
kubectl apply -f k8s/ingress.yaml --dry-run=client
kubectl apply -f k8s/hpa.yaml --dry-run=client
```

### Expected Results
- ✅ All manifests pass validation
- ✅ No YAML syntax errors
- ✅ No API version errors
- ✅ No missing required fields

## Test 5: Minikube Deployment

### Objective
Deploy Nexus Functions to a local Kubernetes cluster using minikube.

### Prerequisites

```powershell
# Start minikube
minikube start --cpus=4 --memory=8192

# Enable ingress addon
minikube addons enable ingress

# Enable metrics-server for HPA
minikube addons enable metrics-server
```

### Deployment Steps

```powershell
# Create namespace
kubectl apply -f k8s/namespace.yaml

# Apply ConfigMaps
kubectl apply -f k8s/configmap.yaml

# Deploy NATS and Nexus
kubectl apply -f k8s/deployment.yaml

# Create Services
kubectl apply -f k8s/service.yaml

# Wait for pods to be ready
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=nexus-functions -n nexus-functions --timeout=300s

# Check status
kubectl get all -n nexus-functions
```

### Validation

```powershell
# Check pod status
kubectl get pods -n nexus-functions

# Expected:
# NAME                              READY   STATUS    RESTARTS   AGE
# nats-0                            1/1     Running   0          2m
# nexus-functions-xxxxxxxxx-xxxxx   1/1     Running   0          2m
# nexus-functions-xxxxxxxxx-xxxxx   1/1     Running   0          2m

# View logs
kubectl logs -n nexus-functions -l app.kubernetes.io/name=nexus-functions --tail=50

# Port-forward to access
kubectl port-forward -n nexus-functions svc/nexus-functions-service 8080:80

# In another terminal, test endpoints
curl http://localhost:8080/health
curl http://localhost:8080/metrics
```

### HPA Testing

```powershell
# Apply HPA
kubectl apply -f k8s/hpa.yaml

# Check HPA status
kubectl get hpa -n nexus-functions

# Generate load (in another terminal)
while ($true) { Invoke-RestMethod -Uri "http://localhost:8080/health"; Start-Sleep -Milliseconds 100 }

# Watch scaling
kubectl get hpa -n nexus-functions -w
```

### Ingress Testing

```powershell
# Update ingress host to use minikube IP
$minikubeIp = minikube ip
# Edit k8s/ingress.yaml and replace host with: nexus.$minikubeIp.nip.io

# Apply ingress
kubectl apply -f k8s/ingress.yaml

# Get ingress URL
kubectl get ingress -n nexus-functions

# Test via ingress
curl -H "Host: nexus.$minikubeIp.nip.io" http://$minikubeIp/health
```

### Cleanup

```powershell
# Delete all resources
kubectl delete -f k8s/

# Or delete namespace (removes everything)
kubectl delete namespace nexus-functions

# Stop minikube
minikube stop
```

### Expected Results
- ✅ All pods reach Running state
- ✅ Health checks pass
- ✅ Services are accessible
- ✅ HPA scales pods based on load
- ✅ Ingress routes traffic correctly

## Test 6: CI/CD Pipeline

### Objective
Verify that the GitHub Actions workflow is properly configured.

### Prerequisites
- GitHub repository with push access
- GitHub Actions enabled

### Local Validation

```powershell
# Install act (GitHub Actions local runner) - optional
# https://github.com/nektos/act

# Validate workflow syntax using act
act -l

# Expected output: List of jobs in the workflow

# Dry-run the test job (without executing)
act -n push
```

### GitHub Testing

```powershell
# Push code to trigger workflow
git push origin main

# View workflow status on GitHub:
# https://github.com/athulkannan2000/nexus-functions/actions

# Check workflow run
# - Test job should pass
# - Security audit should pass
# - Build job should create Docker image
```

### Manual Workflow Trigger

1. Go to: https://github.com/athulkannan2000/nexus-functions/actions
2. Select "CI/CD Pipeline" workflow
3. Click "Run workflow"
4. Select branch: main
5. Click "Run workflow" button

### Expected Results
- ✅ Workflow triggers on push
- ✅ Test job passes (fmt, clippy, tests)
- ✅ Security audit passes
- ✅ Docker image builds successfully
- ✅ Image pushed to ghcr.io (on main branch)

## Test 7: End-to-End Integration

### Objective
Complete end-to-end test of the entire system.

### Scenario: Deploy, Publish Event, Replay

```powershell
# 1. Start local deployment
docker compose up -d
Start-Sleep -Seconds 15

# 2. Verify health
$health = Invoke-RestMethod -Uri "http://localhost:8080/health"
Write-Host "Health: $($health.status)"

# 3. Publish event
$event = @{
    specversion = "1.0"
    type = "com.example.user.created"
    source = "/api/signup"
    id = "e2e-test-001"
    data = @{
        user_id = "user123"
        email = "test@example.com"
        name = "Test User"
    }
} | ConvertTo-Json

$response = Invoke-RestMethod -Uri "http://localhost:8080/events" -Method POST -Body $event -ContentType "application/json"
Write-Host "Event published: $($response.id)"

# 4. List events
$events = Invoke-RestMethod -Uri "http://localhost:8080/events"
Write-Host "Total events: $($events.events.Count)"

# 5. Get specific event
$eventId = $response.id
$eventDetail = Invoke-RestMethod -Uri "http://localhost:8080/events/$eventId"
Write-Host "Event type: $($eventDetail.event.type)"

# 6. Check metrics
$metrics = Invoke-RestMethod -Uri "http://localhost:8080/metrics"
Write-Host "Events published: $($metrics.events.published)"

# 7. Replay event
$replayResult = Invoke-RestMethod -Uri "http://localhost:8080/replay/$eventId" -Method POST
Write-Host "Replay status: $($replayResult.status)"

# 8. Verify replay in metrics
$metricsAfter = Invoke-RestMethod -Uri "http://localhost:8080/metrics"
Write-Host "Events replayed: $($metricsAfter.events.replayed)"

# 9. Cleanup
docker compose down
```

### Expected Results
- ✅ All API endpoints respond correctly
- ✅ Event is stored and retrievable
- ✅ Metrics are updated
- ✅ Event replay works
- ✅ No errors in logs

## Test 8: Performance Baseline

### Objective
Establish performance baseline for the deployment.

### Load Testing with PowerShell

```powershell
# Start deployment
docker compose up -d
Start-Sleep -Seconds 15

# Simple load test
$results = @()
1..100 | ForEach-Object {
    $start = Get-Date
    Invoke-RestMethod -Uri "http://localhost:8080/health" | Out-Null
    $end = Get-Date
    $duration = ($end - $start).TotalMilliseconds
    $results += $duration
}

# Calculate statistics
$avg = ($results | Measure-Object -Average).Average
$min = ($results | Measure-Object -Minimum).Minimum
$max = ($results | Measure-Object -Maximum).Maximum
$p95 = $results | Sort-Object | Select-Object -Index ([math]::Floor($results.Count * 0.95))

Write-Host "Performance Results:"
Write-Host "  Average: $([math]::Round($avg, 2))ms"
Write-Host "  Min: $([math]::Round($min, 2))ms"
Write-Host "  Max: $([math]::Round($max, 2))ms"
Write-Host "  P95: $([math]::Round($p95, 2))ms"

# Cleanup
docker compose down
```

### Expected Baseline (Docker)
- Average latency: <50ms
- P95 latency: <100ms
- Min latency: <20ms

## Troubleshooting Common Issues

### Issue: Port Already in Use

```powershell
# Find process using port 8080
Get-Process -Id (Get-NetTCPConnection -LocalPort 8080).OwningProcess

# Kill process if needed
Stop-Process -Id <process-id> -Force
```

### Issue: Docker Build Fails

```powershell
# Clean Docker cache
docker builder prune -a

# Remove old images
docker rmi nexus-functions:latest

# Rebuild from scratch
docker build --no-cache -t nexus-functions:latest .
```

### Issue: Containers Won't Start

```powershell
# Check logs
docker compose logs

# Check specific service
docker compose logs nexus

# Restart services
docker compose restart
```

### Issue: Kubernetes Pods Pending

```powershell
# Describe pod to see events
kubectl describe pod -n nexus-functions <pod-name>

# Check node resources
kubectl top nodes

# Check events
kubectl get events -n nexus-functions --sort-by='.lastTimestamp'
```

### Issue: Can't Access Services

```powershell
# Check if port-forward is active
Get-Process | Select-String "kubectl"

# Re-establish port-forward
kubectl port-forward -n nexus-functions svc/nexus-functions-service 8080:80
```

## Test Checklist

Before considering deployment infrastructure complete, verify:

### Docker
- [ ] Dockerfile builds without errors
- [ ] Image size is reasonable (~200MB)
- [ ] Multi-stage build uses caching
- [ ] Health check works
- [ ] Non-root user is used

### Docker Compose
- [ ] Development config starts successfully
- [ ] Production config starts successfully
- [ ] Services communicate properly
- [ ] Volumes persist data
- [ ] Health checks pass

### Kubernetes
- [ ] All manifests validate
- [ ] Pods start successfully
- [ ] Services are accessible
- [ ] Ingress routes traffic
- [ ] HPA scales pods
- [ ] Persistent volumes work

### CI/CD
- [ ] Workflow syntax is valid
- [ ] Test job passes
- [ ] Security audit passes
- [ ] Docker build job succeeds
- [ ] Images push to registry

### Integration
- [ ] End-to-end scenario works
- [ ] All API endpoints respond
- [ ] Event replay functions
- [ ] Metrics are accurate
- [ ] Logs are structured

## Next Steps

After completing all tests:

1. **Document Results:** Note any issues or observations
2. **Update Configuration:** Adjust resource limits based on performance
3. **Security Scan:** Run vulnerability scans on Docker images
4. **Load Testing:** Use k6 or similar for comprehensive load tests
5. **Monitoring:** Set up Prometheus and Grafana
6. **Production Deploy:** Follow deployment guide for cloud deployment

## Resources

- [Docker Documentation](https://docs.docker.com/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [GitHub Actions Documentation](https://docs.github.com/actions)
- [NATS Documentation](https://docs.nats.io/)
- [Nexus Functions Deployment Guide](./docs/DEPLOYMENT.md)
