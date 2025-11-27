use uuid::Uuid;

/// Request context for distributed tracing
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub trace_id: String,
    pub event_id: Option<String>,
    pub function_name: Option<String>,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            event_id: None,
            function_name: None,
        }
    }

    pub fn with_event_id(mut self, event_id: String) -> Self {
        self.event_id = Some(event_id);
        self
    }

    pub fn with_function_name(mut self, function_name: String) -> Self {
        self.function_name = Some(function_name);
        self
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Add context to current span
pub fn with_context(ctx: &RequestContext) {
    tracing::Span::current().record("trace_id", &ctx.trace_id.as_str());
    
    if let Some(event_id) = &ctx.event_id {
        tracing::Span::current().record("event_id", event_id.as_str());
    }
    
    if let Some(function_name) = &ctx.function_name {
        tracing::Span::current().record("function_name", function_name.as_str());
    }
}

/// Get trace ID from current span (if available)
pub fn get_trace_id() -> Option<String> {
    // In a real implementation, this would extract from span context
    // For now, return None as we need proper span context extraction
    None
}
