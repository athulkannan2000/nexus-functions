use std::io::{stdin, stdout, Read, Write};

#[no_mangle]
pub extern "C" fn handle_event() {
    // Read event payload from stdin
    let mut input = Vec::new();
    match stdin().read_to_end(&mut input) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("[ERROR] Failed to read input: {}", e);
            return;
        }
    }

    // Get trace ID from environment
    let trace_id = std::env::var("TRACE_ID").unwrap_or_else(|_| "unknown".to_string());
    
    // Log the event
    eprintln!("[trace={}] Processing hello event", trace_id);
    
    // Parse input (simplified - in production you'd use serde_json)
    let input_str = String::from_utf8_lossy(&input);
    eprintln!("[trace={}] Received: {}", trace_id, input_str);

    // Generate response
    let response = format!(
        r#"{{"message": "Hello from Nexus Functions!", "timestamp": "{}", "trace_id": "{}"}}"#,
        chrono::Utc::now().to_rfc3339(),
        trace_id
    );

    // Write response to stdout
    match stdout().write_all(response.as_bytes()) {
        Ok(_) => {
            eprintln!("[trace={}] Response sent successfully", trace_id);
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to write output: {}", e);
        }
    }
}

// Stub for chrono - in real implementation would use actual chrono crate
mod chrono {
    pub struct Utc;
    impl Utc {
        pub fn now() -> Self {
            Self
        }
        pub fn to_rfc3339(&self) -> String {
            "2025-11-26T00:00:00Z".to_string()
        }
    }
}
