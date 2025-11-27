# API Reference - Nexus Functions

## Overview

Nexus Functions provides a RESTful HTTP API for event ingestion, function execution, and system monitoring. All endpoints return JSON responses with structured error handling and request tracing.

## Base URL

```
http://localhost:8080
```

## Common Response Headers

All responses include:
- `Content-Type: application/json`
- Trace IDs are included in error responses

---

## Endpoints

### 1. Health Check

Check server status and uptime.

**Endpoint:** `GET /health`

**Response (200 OK):**
```json
{
  "status": "ok",
  "version": "0.1.0",
  "nats_connected": true,
  "uptime_seconds": 3600
}
```

**Example:**
```bash
curl http://localhost:8080/health
```

---

### 2. System Metrics

Retrieve comprehensive system metrics including event and function statistics.

**Endpoint:** `GET /metrics`

**Response (200 OK):**
```json
{
  "events": {
    "published": 150,
    "replayed": 12,
    "failed": 2,
    "success_rate": 98.68
  },
  "functions": {
    "executed": 162,
    "succeeded": 160,
    "failed": 2,
    "success_rate": 98.77,
    "avg_execution_time_ms": 15.5
  },
  "system": {
    "uptime_seconds": 7200,
    "nats_connected": true
  }
}
```

**Metrics Explanation:**
- `events.published`: Total events published via HTTP
- `events.replayed`: Events replayed for reprocessing
- `events.failed`: Events that failed to publish
- `events.success_rate`: Percentage of successful event operations
- `functions.executed`: Total function invocations
- `functions.succeeded`: Successful function executions
- `functions.failed`: Failed function executions
- `functions.success_rate`: Function execution success percentage
- `functions.avg_execution_time_ms`: Average function execution time
- `system.uptime_seconds`: Server uptime in seconds
- `system.nats_connected`: NATS connection status

**Example:**
```bash
curl http://localhost:8080/metrics
```

---

### 3. Publish Event (Webhook)

Publish events with type extracted from URL path.

**Endpoint:** `POST /webhook/{event_type}`

**Request Body:**
```json
{
  "user_id": 12345,
  "email": "user@example.com",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

**Response (200 OK):**
```json
{
  "event_id": "a1b2c3d4-e5f6-4789-a012-3456789abcde",
  "status": "published",
  "event_type": "com.nexus.user.created"
}
```

**Path Parameters:**
- `event_type`: The event type (e.g., `user.created`, `order.placed`)
  - Automatically prefixed with `com.nexus.`

**Example:**
```bash
curl -X POST http://localhost:8080/webhook/user.created \
  -H "Content-Type: application/json" \
  -d '{"user_id": 12345, "email": "user@example.com"}'
```

**Behavior:**
- Event is published to NATS JetStream
- Matching functions are automatically triggered (fire-and-forget)
- Metrics are updated (`events.published`, `functions.executed`)

---

### 4. Publish Event (Root)

Publish events with type in request body.

**Endpoint:** `POST /events`

**Request Body:**
```json
{
  "event_type": "user.created",
  "user_id": 12345,
  "email": "user@example.com"
}
```

**Response (200 OK):**
```json
{
  "event_id": "b2c3d4e5-f6a7-4890-b123-456789abcdef",
  "status": "published",
  "event_type": "com.nexus.user.created"
}
```

**Request Fields:**
- `event_type` (optional): Event type identifier (defaults to `generic.event`)
- Other fields: Event payload data

**Example:**
```bash
curl -X POST http://localhost:8080/events \
  -H "Content-Type: application/json" \
  -d '{"event_type": "order.placed", "order_id": 789, "amount": 99.99}'
```

---

### 5. List Events

Retrieve a list of published events with optional filtering.

**Endpoint:** `GET /events`

**Query Parameters:**
- `type` (optional): Filter by event type (e.g., `com.nexus.user.created`)
- `limit` (optional): Maximum number of events to return (default: 100)

**Response (200 OK):**
```json
{
  "events": [
    {
      "specversion": "1.0",
      "type": "com.nexus.user.created",
      "source": "/api/webhook",
      "id": "a1b2c3d4-e5f6-4789-a012-3456789abcde",
      "time": "2024-01-01T12:00:00Z",
      "data": {
        "user_id": 12345,
        "email": "user@example.com"
      }
    }
  ],
  "count": 1,
  "total": 150
}
```

**Response Fields:**
- `events`: Array of CloudEvents (v1.0 spec)
- `count`: Number of events in current response
- `total`: Total number of events in system

**Examples:**
```bash
# List all events (limit 100)
curl http://localhost:8080/events

# List specific event type
curl "http://localhost:8080/events?type=com.nexus.user.created"

# Limit results
curl "http://localhost:8080/events?limit=10"

# Combine filters
curl "http://localhost:8080/events?type=com.nexus.user.created&limit=5"
```

---

### 6. Get Event by ID

Retrieve a specific event by its ID.

**Endpoint:** `GET /events/{event_id}`

**Path Parameters:**
- `event_id`: The UUID of the event

**Response (200 OK):**
```json
{
  "specversion": "1.0",
  "type": "com.nexus.user.created",
  "source": "/api/webhook",
  "id": "a1b2c3d4-e5f6-4789-a012-3456789abcde",
  "time": "2024-01-01T12:00:00Z",
  "data": {
    "user_id": 12345,
    "email": "user@example.com"
  }
}
```

**Response (404 Not Found):**
```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Event not found: a1b2c3d4-e5f6-4789-a012-3456789abcde",
    "details": {
      "resource": "Event",
      "id": "a1b2c3d4-e5f6-4789-a012-3456789abcde"
    }
  },
  "trace_id": "4c378374-8fa0-4e60-bb60-c75bffd06062"
}
```

**Example:**
```bash
curl http://localhost:8080/events/a1b2c3d4-e5f6-4789-a012-3456789abcde
```

---

### 7. Replay Event

Replay a previously published event, re-triggering all matching functions.

**Endpoint:** `POST /replay/{event_id}`

**Path Parameters:**
- `event_id`: The UUID of the event to replay

**Response (200 OK):**
```json
{
  "event_id": "a1b2c3d4-e5f6-4789-a012-3456789abcde",
  "status": "replayed",
  "message": "Event type: com.nexus.user.created"
}
```

**Response (404 Not Found):**
```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Event not found: a1b2c3d4-e5f6-4789-a012-3456789abcde",
    "details": {
      "resource": "Event",
      "id": "a1b2c3d4-e5f6-4789-a012-3456789abcde"
    }
  },
  "trace_id": "5d489485-9fb1-5e71-ac71-d86c5ce17173"
}
```

**Behavior:**
- Event is republished to NATS
- All matching functions are re-executed with original payload
- Metrics are updated (`events.replayed`, `functions.executed`)

**Example:**
```bash
curl -X POST http://localhost:8080/replay/a1b2c3d4-e5f6-4789-a012-3456789abcde
```

---

### 8. Execute Function Manually

Manually execute all functions matching a specific event.

**Endpoint:** `POST /execute/{event_id}`

**Path Parameters:**
- `event_id`: The UUID of the event to execute against

**Response (200 OK):**
```json
{
  "event_id": "a1b2c3d4-e5f6-4789-a012-3456789abcde",
  "status": "executed",
  "functions_executed": [
    {
      "function_name": "user-welcome",
      "status": "success",
      "output_size": 42,
      "output": "{\"status\":\"ok\",\"message\":\"User welcomed\"}"
    }
  ]
}
```

**Example:**
```bash
curl -X POST http://localhost:8080/execute/a1b2c3d4-e5f6-4789-a012-3456789abcde
```

---

## Error Responses

All endpoints return structured error responses on failure.

### Error Response Format

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      "key": "value"
    }
  },
  "trace_id": "uuid-for-request-tracing"
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `NOT_FOUND` | 404 | Resource (event) not found |
| `INVALID_INPUT` | 400 | Invalid request data or parameters |
| `CONFIG_ERROR` | 500 | Server configuration issue |
| `NATS_ERROR` | 503 | NATS connection or operation failed |
| `WASM_ERROR` | 500 | Function execution failed |
| `INTERNAL_ERROR` | 500 | General server error |

### Example Error Responses

**404 Not Found:**
```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Event not found: abc-123",
    "details": {
      "resource": "Event",
      "id": "abc-123"
    }
  },
  "trace_id": "trace-uuid"
}
```

**503 Service Unavailable:**
```json
{
  "error": {
    "code": "NATS_ERROR",
    "message": "Failed to publish event: Connection timeout",
    "details": null
  },
  "trace_id": "trace-uuid"
}
```

---

## Request Tracing

All requests are automatically assigned a trace ID for distributed tracing. The trace ID is:
- Included in all error responses (`trace_id` field)
- Propagated through the system for observability
- Logged in structured JSON format

**Example Log Entry:**
```json
{
  "timestamp": "2025-11-27T06:40:06Z",
  "level": "INFO",
  "target": "nexus_core::server",
  "fields": {
    "message": "Event published successfully",
    "event_id": "a1b2c3d4-e5f6-4789-a012-3456789abcde",
    "event_type": "com.nexus.user.created"
  },
  "span": {
    "trace_id": "4c378374-8fa0-4e60-bb60-c75bffd06062"
  }
}
```

---

## Rate Limits

Currently, Nexus Functions does not enforce rate limits. For production deployments, consider:
- Deploying behind a reverse proxy (Nginx, Traefik)
- Using API gateway rate limiting
- NATS JetStream flow control

---

## Authentication

The current MVP does not include authentication. For production:
- Implement API key validation
- Use JWT tokens
- Configure mutual TLS

---

## WebSocket Support

WebSocket support for real-time event streaming is planned for future releases.

---

## SDK Support

Official SDKs coming soon:
- Rust
- Python
- JavaScript/TypeScript
- Go

---

## API Versioning

Current API version: `v1` (implicit)

Future versions will use URL path versioning:
- `/v1/events`
- `/v2/events`

---

## Support

- **Issues:** [GitHub Issues](https://github.com/athulkannan2000/nexus-functions/issues)
- **Documentation:** [Getting Started](./GETTING_STARTED.md)
- **Architecture:** [ARCHITECTURE.md](./ARCHITECTURE.md)
