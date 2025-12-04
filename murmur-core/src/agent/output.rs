//! Output streaming and parsing for Claude Code JSON stream format

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;

use crate::{Error, Result};

/// A message from the Claude Code stream-json output
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamMessage {
    /// System message at the start
    System {
        #[serde(default)]
        subtype: Option<String>,
        #[serde(default)]
        session_id: Option<String>,
    },

    /// Assistant text output
    Assistant {
        #[serde(default)]
        message: AssistantMessage,
    },

    /// Tool usage by the assistant
    ToolUse {
        tool: String,
        #[serde(default)]
        input: serde_json::Value,
    },

    /// Result from tool execution
    ToolResult {
        #[serde(default)]
        output: String,
        #[serde(default)]
        is_error: bool,
    },

    /// Final result with cost information
    Result {
        #[serde(default)]
        cost: Option<CostInfo>,
        #[serde(default)]
        duration_ms: Option<u64>,
        #[serde(default)]
        duration_api_ms: Option<u64>,
    },
}

/// Assistant message content
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AssistantMessage {
    #[serde(default)]
    pub content: String,
}

/// Cost information from the result
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CostInfo {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_tokens: Option<u64>,
    #[serde(default)]
    pub cache_write_tokens: Option<u64>,
}

/// Handler for processing stream messages
pub trait StreamHandler: Send {
    /// Called when a system message is received
    fn on_system(&mut self, _subtype: Option<&str>, _session_id: Option<&str>) {}

    /// Called when assistant text is received
    fn on_assistant_text(&mut self, text: &str);

    /// Called when the assistant uses a tool
    fn on_tool_use(&mut self, _tool: &str, _input: &serde_json::Value) {}

    /// Called when a tool returns a result
    fn on_tool_result(&mut self, _output: &str, _is_error: bool) {}

    /// Called when the stream completes
    fn on_complete(&mut self, _cost: Option<&CostInfo>, _duration_ms: Option<u64>) {}

    /// Called when a parse error occurs (allows handler to skip malformed lines)
    fn on_parse_error(&mut self, _line: &str, _error: &serde_json::Error) {}
}

/// Simple handler that prints assistant output to stdout
pub struct PrintHandler {
    /// Whether to show verbose tool information
    verbose: bool,
}

impl PrintHandler {
    /// Create a new print handler
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl StreamHandler for PrintHandler {
    fn on_system(&mut self, subtype: Option<&str>, _session_id: Option<&str>) {
        if self.verbose {
            if let Some(st) = subtype {
                eprintln!("[system: {}]", st);
            }
        }
    }

    fn on_assistant_text(&mut self, text: &str) {
        print!("{}", text);
    }

    fn on_tool_use(&mut self, tool: &str, input: &serde_json::Value) {
        if self.verbose {
            eprintln!("\n[tool: {} with input: {}]", tool, input);
        }
    }

    fn on_tool_result(&mut self, output: &str, is_error: bool) {
        if self.verbose {
            let prefix = if is_error { "error" } else { "result" };
            // Truncate long outputs
            let display = if output.len() > 200 {
                format!("{}... ({} chars)", &output[..200], output.len())
            } else {
                output.to_string()
            };
            eprintln!("[{}: {}]", prefix, display);
        }
    }

    fn on_complete(&mut self, cost: Option<&CostInfo>, duration_ms: Option<u64>) {
        println!(); // Ensure final newline
        if self.verbose {
            if let Some(c) = cost {
                eprintln!("[tokens: {} in, {} out]", c.input_tokens, c.output_tokens);
            }
            if let Some(d) = duration_ms {
                eprintln!("[duration: {}ms]", d);
            }
        }
    }

    fn on_parse_error(&mut self, line: &str, error: &serde_json::Error) {
        if self.verbose {
            eprintln!("[parse error on line '{}': {}]", line, error);
        }
    }
}

/// Stream output from a Claude Code process
pub struct OutputStreamer {
    reader: BufReader<ChildStdout>,
}

impl OutputStreamer {
    /// Create a new output streamer from a child process stdout
    pub fn new(stdout: ChildStdout) -> Self {
        Self {
            reader: BufReader::new(stdout),
        }
    }

    /// Stream output, calling the handler for each message
    ///
    /// Returns when the stream ends (process closes stdout)
    pub async fn stream<H: StreamHandler>(&mut self, handler: &mut H) -> Result<()> {
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = self.reader.read_line(&mut line).await.map_err(Error::Io)?;

            if bytes_read == 0 {
                // EOF
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match serde_json::from_str::<StreamMessage>(trimmed) {
                Ok(msg) => Self::dispatch_message(handler, msg),
                Err(e) => handler.on_parse_error(trimmed, &e),
            }
        }

        Ok(())
    }

    fn dispatch_message<H: StreamHandler>(handler: &mut H, msg: StreamMessage) {
        match msg {
            StreamMessage::System {
                subtype,
                session_id,
            } => {
                handler.on_system(subtype.as_deref(), session_id.as_deref());
            }
            StreamMessage::Assistant { message } => {
                handler.on_assistant_text(&message.content);
            }
            StreamMessage::ToolUse { tool, input } => {
                handler.on_tool_use(&tool, &input);
            }
            StreamMessage::ToolResult { output, is_error } => {
                handler.on_tool_result(&output, is_error);
            }
            StreamMessage::Result {
                cost, duration_ms, ..
            } => {
                handler.on_complete(cost.as_ref(), duration_ms);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assistant_message() {
        let json = r#"{"type":"assistant","message":{"content":"Hello world"}}"#;
        let msg: StreamMessage = serde_json::from_str(json).unwrap();
        match msg {
            StreamMessage::Assistant { message } => {
                assert_eq!(message.content, "Hello world");
            }
            _ => panic!("Expected Assistant message"),
        }
    }

    #[test]
    fn test_parse_tool_use() {
        let json = r#"{"type":"tool_use","tool":"Read","input":{"file":"/test.txt"}}"#;
        let msg: StreamMessage = serde_json::from_str(json).unwrap();
        match msg {
            StreamMessage::ToolUse { tool, input } => {
                assert_eq!(tool, "Read");
                assert_eq!(input["file"], "/test.txt");
            }
            _ => panic!("Expected ToolUse message"),
        }
    }

    #[test]
    fn test_parse_result() {
        let json = r#"{"type":"result","cost":{"input_tokens":100,"output_tokens":50},"duration_ms":1234}"#;
        let msg: StreamMessage = serde_json::from_str(json).unwrap();
        match msg {
            StreamMessage::Result {
                cost, duration_ms, ..
            } => {
                let c = cost.unwrap();
                assert_eq!(c.input_tokens, 100);
                assert_eq!(c.output_tokens, 50);
                assert_eq!(duration_ms, Some(1234));
            }
            _ => panic!("Expected Result message"),
        }
    }

    #[test]
    fn test_parse_system() {
        let json = r#"{"type":"system","subtype":"init","session_id":"abc123"}"#;
        let msg: StreamMessage = serde_json::from_str(json).unwrap();
        match msg {
            StreamMessage::System {
                subtype,
                session_id,
            } => {
                assert_eq!(subtype, Some("init".to_string()));
                assert_eq!(session_id, Some("abc123".to_string()));
            }
            _ => panic!("Expected System message"),
        }
    }
}
