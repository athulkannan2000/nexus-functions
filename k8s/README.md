# Kubernetes Deployment Guide

This directory contains Kubernetes manifests for deploying Nexus Functions to a Kubernetes cluster.

## Prerequisites

- Kubernetes cluster (v1.24+)
- kubectl configured
- NGINX Ingress Controller (for ingress)
- cert-manager (optional, for TLS)
- Metrics Server (for HPA)

## Quick Start

### 1. Deploy everything

```bash
kubectl apply -f k8s/
```

### 2. Deploy step-by-step

```bash
# Create namespace
kubectl apply -f k8s/namespace.yaml

# Create ConfigMaps
kubectl apply -f k8s/configmap.yaml

# Deploy NATS StatefulSet
kubectl apply -f k8s/deployment.yaml

# Create Services
kubectl apply -f k8s/service.yaml

# Create Ingress (update host in ingress.yaml first)
kubectl apply -f k8s/ingress.yaml

# Create HPA (optional)
kubectl apply -f k8s/hpa.yaml
```

## Manifests Overview

| File | Description |
|------|-------------|
| `namespace.yaml` | Creates `nexus-functions` namespace |
| `configmap.yaml` | Configuration for Nexus and NATS |
| `deployment.yaml` | StatefulSet for NATS, Deployment for Nexus |
| `service.yaml` | Services for NATS and Nexus |
| `ingress.yaml` | Ingress for external access |
| `hpa.yaml` | HorizontalPodAutoscaler for Nexus |

## Configuration

### Update Ingress Host

Edit `k8s/ingress.yaml`:

```yaml
spec:
  rules:
  - host: nexus.your-domain.com  # Change this
```

### Update Docker Image

Edit `k8s/deployment.yaml`:

```yaml
containers:
- name: nexus
  image: ghcr.io/your-username/nexus-functions:latest  # Change this
```

### Resource Limits

Adjust in `k8s/deployment.yaml`:

```yaml
resources:
  requests:
    memory: "256Mi"
    cpu: "100m"
  limits:
    memory: "1Gi"
    cpu: "1000m"
```

## Monitoring

### Check deployment status

```bash
kubectl get all -n nexus-functions
```

### View logs

```bash
# Nexus Functions logs
kubectl logs -n nexus-functions -l app.kubernetes.io/name=nexus-functions -f

# NATS logs
kubectl logs -n nexus-functions -l app.kubernetes.io/name=nats -f
```

### Check health

```bash
# Port-forward to access locally
kubectl port-forward -n nexus-functions svc/nexus-functions-service 8080:80

# Test health endpoint
curl http://localhost:8080/health
```

## Scaling

### Manual scaling

```bash
kubectl scale deployment nexus-functions -n nexus-functions --replicas=5
```

### Auto-scaling

HPA automatically scales based on CPU/memory:

```bash
# View HPA status
kubectl get hpa -n nexus-functions

# Describe HPA
kubectl describe hpa nexus-functions-hpa -n nexus-functions
```

## TLS/HTTPS

### Using cert-manager

1. Install cert-manager:

```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml
```

2. Create ClusterIssuer:

```yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
```

3. Uncomment TLS section in `k8s/ingress.yaml`

## Persistence

NATS uses a StatefulSet with PersistentVolumeClaim:

- Storage: 10Gi (configurable in `deployment.yaml`)
- AccessMode: ReadWriteOnce
- Retention: 7 days (configurable in ConfigMap)

## Cleanup

```bash
# Delete everything
kubectl delete -f k8s/

# Or delete namespace (removes everything)
kubectl delete namespace nexus-functions
```

## Production Checklist

- [ ] Update Docker image repository
- [ ] Configure persistent storage class
- [ ] Set appropriate resource limits
- [ ] Configure TLS/HTTPS
- [ ] Enable monitoring (Prometheus/Grafana)
- [ ] Configure backup strategy for NATS data
- [ ] Set up logging aggregation
- [ ] Configure network policies
- [ ] Review security contexts
- [ ] Set up CI/CD pipeline

## Troubleshooting

### Pods not starting

```bash
kubectl describe pod -n nexus-functions <pod-name>
```

### NATS connection issues

```bash
# Check NATS health
kubectl port-forward -n nexus-functions svc/nats-service 8222:8222
curl http://localhost:8222/healthz
```

### Image pull errors

```bash
# Create image pull secret if using private registry
kubectl create secret docker-registry regcred \
  --docker-server=<registry-server> \
  --docker-username=<username> \
  --docker-password=<password> \
  -n nexus-functions
```

Then add to deployment:

```yaml
spec:
  imagePullSecrets:
  - name: regcred
```
