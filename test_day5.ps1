# Day 5 Testing Script - Observability & Error Handling
# Tests structured logging, error responses, and metrics collection

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Day 5: Testing Observability Features" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$baseUrl = "http://localhost:8080"
$testsPassed = 0
$testsFailed = 0

function Test-Endpoint {
    param(
        [string]$Name,
        [string]$Url,
        [string]$Method = "GET",
        [object]$Body = $null,
        [int]$ExpectedStatus = 200
    )
    
    Write-Host "`n--- Testing: $Name ---" -ForegroundColor Yellow
    
    try {
        $params = @{
            Uri = $Url
            Method = $Method
            Headers = @{ "Content-Type" = "application/json" }
            TimeoutSec = 10
        }
        
        if ($Body) {
            $params.Body = ($Body | ConvertTo-Json -Compress)
        }
        
        try {
            $response = Invoke-WebRequest @params -UseBasicParsing
            $statusCode = $response.StatusCode
        } catch {
            $statusCode = $_.Exception.Response.StatusCode.value__
            $response = $_.Exception.Response
        }
        
        if ($statusCode -eq $ExpectedStatus) {
            Write-Host "✓ Status: $statusCode (Expected: $ExpectedStatus)" -ForegroundColor Green
            $script:testsPassed++
            return @{
                Success = $true
                StatusCode = $statusCode
                Content = if ($response.Content) { $response.Content } else { $null }
            }
        } else {
            Write-Host "✗ Status: $statusCode (Expected: $ExpectedStatus)" -ForegroundColor Red
            $script:testsFailed++
            return @{
                Success = $false
                StatusCode = $statusCode
                Content = if ($response.Content) { $response.Content } else { $null }
            }
        }
    } catch {
        Write-Host "✗ Error: $($_.Exception.Message)" -ForegroundColor Red
        $script:testsFailed++
        return @{ Success = $false; Error = $_.Exception.Message }
    }
}

# Test 1: Health Check with Uptime
Write-Host "`n=== Test 1: Health Check with Uptime ===" -ForegroundColor Cyan
$result = Test-Endpoint -Name "Health Check" -Url "$baseUrl/health"
if ($result.Success -and $result.Content) {
    $health = $result.Content | ConvertFrom-Json
    Write-Host "Status: $($health.status)" -ForegroundColor White
    Write-Host "Version: $($health.version)" -ForegroundColor White
    Write-Host "NATS Connected: $($health.nats_connected)" -ForegroundColor White
    Write-Host "Uptime: $($health.uptime_seconds) seconds" -ForegroundColor White
}

# Test 2: Metrics Endpoint (New)
Write-Host "`n=== Test 2: Metrics Endpoint ===" -ForegroundColor Cyan
$result = Test-Endpoint -Name "Get Metrics" -Url "$baseUrl/metrics"
if ($result.Success -and $result.Content) {
    $metrics = $result.Content | ConvertFrom-Json
    Write-Host "`nEvent Metrics:" -ForegroundColor White
    Write-Host "  Published: $($metrics.events.published)" -ForegroundColor White
    Write-Host "  Replayed: $($metrics.events.replayed)" -ForegroundColor White
    Write-Host "  Failed: $($metrics.events.failed)" -ForegroundColor White
    Write-Host "  Success Rate: $([math]::Round($metrics.events.success_rate, 2))%" -ForegroundColor White
    
    Write-Host "`nFunction Metrics:" -ForegroundColor White
    Write-Host "  Executed: $($metrics.functions.executed)" -ForegroundColor White
    Write-Host "  Succeeded: $($metrics.functions.succeeded)" -ForegroundColor White
    Write-Host "  Failed: $($metrics.functions.failed)" -ForegroundColor White
    Write-Host "  Success Rate: $([math]::Round($metrics.functions.success_rate, 2))%" -ForegroundColor White
    Write-Host "  Avg Execution Time: $([math]::Round($metrics.functions.avg_execution_time_ms, 2))ms" -ForegroundColor White
    
    Write-Host "`nSystem Metrics:" -ForegroundColor White
    Write-Host "  Uptime: $($metrics.system.uptime_seconds) seconds" -ForegroundColor White
    Write-Host "  NATS Connected: $($metrics.system.nats_connected)" -ForegroundColor White
}

# Test 3: Publish Event (Should increment metrics)
Write-Host "`n=== Test 3: Publish Event ===" -ForegroundColor Cyan
$eventPayload = @{
    user_id = 12345
    username = "test_user"
    email = "test@example.com"
}
$result = Test-Endpoint -Name "Publish Event" -Url "$baseUrl/webhook/user.created" -Method POST -Body $eventPayload
if ($result.Success -and $result.Content) {
    $response = $result.Content | ConvertFrom-Json
    $eventId = $response.event_id
    Write-Host "Event ID: $eventId" -ForegroundColor White
    Write-Host "Status: $($response.status)" -ForegroundColor White
    Write-Host "Type: $($response.event_type)" -ForegroundColor White
    
    # Wait for async function execution
    Write-Host "Waiting for function execution..." -ForegroundColor Gray
    Start-Sleep -Seconds 2
}

# Test 4: Get Non-existent Event (Test Error Response)
Write-Host "`n=== Test 4: Error Response - Event Not Found ===" -ForegroundColor Cyan
$fakeId = [guid]::NewGuid().ToString()
$result = Test-Endpoint -Name "Get Non-existent Event" -Url "$baseUrl/events/$fakeId" -ExpectedStatus 404

if ($result.Content) {
    try {
        $errorResponse = $result.Content | ConvertFrom-Json
        Write-Host "`nError Response Structure:" -ForegroundColor White
        Write-Host "  Code: $($errorResponse.error.code)" -ForegroundColor White
        Write-Host "  Message: $($errorResponse.error.message)" -ForegroundColor White
        if ($errorResponse.trace_id) {
            Write-Host "  Trace ID: $($errorResponse.trace_id)" -ForegroundColor White
        }
        if ($errorResponse.error.details) {
            Write-Host "  Details: $($errorResponse.error.details | ConvertTo-Json -Compress)" -ForegroundColor White
        }
    } catch {
        Write-Host "Could not parse error response" -ForegroundColor Red
    }
}

# Test 5: Publish Multiple Events
Write-Host "`n=== Test 5: Publish Multiple Events ===" -ForegroundColor Cyan
$eventTypes = @("order.placed", "product.updated", "payment.completed")
$publishedEvents = @()

foreach ($eventType in $eventTypes) {
    $payload = @{
        event_type = $eventType
        timestamp = (Get-Date).ToString("o")
        data = @{ test = "data_$eventType" }
    }
    
    $result = Test-Endpoint -Name "Publish $eventType" -Url "$baseUrl/webhook/$eventType" -Method POST -Body $payload
    if ($result.Success -and $result.Content) {
        $response = $result.Content | ConvertFrom-Json
        $publishedEvents += $response.event_id
        Write-Host "Published: $($response.event_id)" -ForegroundColor White
    }
}

Start-Sleep -Seconds 2

# Test 6: Check Updated Metrics
Write-Host "`n=== Test 6: Updated Metrics After Events ===" -ForegroundColor Cyan
$result = Test-Endpoint -Name "Get Updated Metrics" -Url "$baseUrl/metrics"
if ($result.Success -and $result.Content) {
    $metrics = $result.Content | ConvertFrom-Json
    Write-Host "`nUpdated Event Metrics:" -ForegroundColor White
    Write-Host "  Published: $($metrics.events.published)" -ForegroundColor Green
    Write-Host "  Replayed: $($metrics.events.replayed)" -ForegroundColor White
    Write-Host "  Failed: $($metrics.events.failed)" -ForegroundColor $(if ($metrics.events.failed -gt 0) { "Yellow" } else { "Green" })
    Write-Host "  Success Rate: $([math]::Round($metrics.events.success_rate, 2))%" -ForegroundColor Green
    
    Write-Host "`nUpdated Function Metrics:" -ForegroundColor White
    Write-Host "  Executed: $($metrics.functions.executed)" -ForegroundColor Green
    Write-Host "  Succeeded: $($metrics.functions.succeeded)" -ForegroundColor White
    Write-Host "  Failed: $($metrics.functions.failed)" -ForegroundColor $(if ($metrics.functions.failed -gt 0) { "Yellow" } else { "Green" })
    Write-Host "  Success Rate: $([math]::Round($metrics.functions.success_rate, 2))%" -ForegroundColor Green
    Write-Host "  Avg Execution Time: $([math]::Round($metrics.functions.avg_execution_time_ms, 2))ms" -ForegroundColor White
}

# Test 7: List Events
Write-Host "`n=== Test 7: List Events ===" -ForegroundColor Cyan
$result = Test-Endpoint -Name "List Events" -Url "$baseUrl/events?limit=10"
if ($result.Success -and $result.Content) {
    $response = $result.Content | ConvertFrom-Json
    Write-Host "Total Events: $($response.total)" -ForegroundColor White
    Write-Host "Returned: $($response.count)" -ForegroundColor White
    
    if ($response.events.Count -gt 0) {
        Write-Host "`nRecent Events:" -ForegroundColor White
        $response.events | Select-Object -First 5 | ForEach-Object {
            Write-Host "  - $($_.id): $($_.type)" -ForegroundColor Gray
        }
    }
}

# Test 8: Replay Event (Should increment replay metrics)
Write-Host "`n=== Test 8: Replay Event ===" -ForegroundColor Cyan
if ($publishedEvents.Count -gt 0) {
    $replayEventId = $publishedEvents[0]
    $result = Test-Endpoint -Name "Replay Event" -Url "$baseUrl/replay/$replayEventId" -Method POST
    
    if ($result.Success -and $result.Content) {
        $response = $result.Content | ConvertFrom-Json
        Write-Host "Event ID: $($response.event_id)" -ForegroundColor White
        Write-Host "Status: $($response.status)" -ForegroundColor White
        Write-Host "Message: $($response.message)" -ForegroundColor White
    }
    
    Start-Sleep -Seconds 2
    
    # Check metrics after replay
    $result = Test-Endpoint -Name "Metrics After Replay" -Url "$baseUrl/metrics"
    if ($result.Success -and $result.Content) {
        $metrics = $result.Content | ConvertFrom-Json
        Write-Host "`nMetrics After Replay:" -ForegroundColor White
        Write-Host "  Events Replayed: $($metrics.events.replayed)" -ForegroundColor Green
    }
}

# Test 9: Test Invalid Event Replay (404 Error)
Write-Host "`n=== Test 9: Invalid Event Replay ===" -ForegroundColor Cyan
$fakeId = [guid]::NewGuid().ToString()
$result = Test-Endpoint -Name "Replay Non-existent Event" -Url "$baseUrl/replay/$fakeId" -Method POST -ExpectedStatus 404

if ($result.Content) {
    try {
        $errorResponse = $result.Content | ConvertFrom-Json
        Write-Host "`nError Response:" -ForegroundColor White
        Write-Host "  Code: $($errorResponse.error.code)" -ForegroundColor White
        Write-Host "  Message: $($errorResponse.error.message)" -ForegroundColor White
        Write-Host "  Trace ID: $($errorResponse.trace_id)" -ForegroundColor White
    } catch {
        Write-Host "Could not parse error response" -ForegroundColor Red
    }
}

# Test 10: Final Metrics Summary
Write-Host "`n=== Test 10: Final Metrics Summary ===" -ForegroundColor Cyan
$result = Test-Endpoint -Name "Final Metrics" -Url "$baseUrl/metrics"
if ($result.Success -and $result.Content) {
    $metrics = $result.Content | ConvertFrom-Json
    
    Write-Host "`n╔════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║      FINAL METRICS SUMMARY             ║" -ForegroundColor Cyan
    Write-Host "╠════════════════════════════════════════╣" -ForegroundColor Cyan
    Write-Host "║ Events                                 ║" -ForegroundColor Cyan
    Write-Host "║   Published:     $($metrics.events.published.ToString().PadLeft(20)) ║" -ForegroundColor White
    Write-Host "║   Replayed:      $($metrics.events.replayed.ToString().PadLeft(20)) ║" -ForegroundColor White
    Write-Host "║   Failed:        $($metrics.events.failed.ToString().PadLeft(20)) ║" -ForegroundColor $(if ($metrics.events.failed -gt 0) { "Yellow" } else { "White" })
    Write-Host "║   Success Rate:  $("$([math]::Round($metrics.events.success_rate, 2))%".PadLeft(20)) ║" -ForegroundColor Green
    Write-Host "║                                        ║" -ForegroundColor Cyan
    Write-Host "║ Functions                              ║" -ForegroundColor Cyan
    Write-Host "║   Executed:      $($metrics.functions.executed.ToString().PadLeft(20)) ║" -ForegroundColor White
    Write-Host "║   Succeeded:     $($metrics.functions.succeeded.ToString().PadLeft(20)) ║" -ForegroundColor White
    Write-Host "║   Failed:        $($metrics.functions.failed.ToString().PadLeft(20)) ║" -ForegroundColor $(if ($metrics.functions.failed -gt 0) { "Yellow" } else { "White" })
    Write-Host "║   Success Rate:  $("$([math]::Round($metrics.functions.success_rate, 2))%".PadLeft(20)) ║" -ForegroundColor Green
    Write-Host "║   Avg Time:      $("$([math]::Round($metrics.functions.avg_execution_time_ms, 2))ms".PadLeft(20)) ║" -ForegroundColor White
    Write-Host "║                                        ║" -ForegroundColor Cyan
    Write-Host "║ System                                 ║" -ForegroundColor Cyan
    Write-Host "║   Uptime:        $("$($metrics.system.uptime_seconds)s".PadLeft(20)) ║" -ForegroundColor White
    Write-Host "║   NATS:          $($metrics.system.nats_connected.ToString().PadLeft(20)) ║" -ForegroundColor $(if ($metrics.system.nats_connected) { "Green" } else { "Red" })
    Write-Host "╚════════════════════════════════════════╝`n" -ForegroundColor Cyan
}

# Summary
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Test Summary" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Tests Passed: $testsPassed" -ForegroundColor Green
Write-Host "Tests Failed: $testsFailed" -ForegroundColor $(if ($testsFailed -gt 0) { "Red" } else { "Green" })

if ($testsFailed -eq 0) {
    Write-Host "`n" -NoNewline
    Write-Host "All Day 5 observability tests passed!" -ForegroundColor Green
} else {
    Write-Host "`n" -NoNewline
    Write-Host "Some tests failed. Review the output above." -ForegroundColor Red
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host ""
