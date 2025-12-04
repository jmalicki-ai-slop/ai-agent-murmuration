# WebSocket API Design

## Overview

Real-time communication between the Dispatch server and web clients using WebSocket connections. Enables live updates for agent status, issue changes, proposal voting, and workflow progress.

---

## Connection Lifecycle

```
Client                                Server
   |                                     |
   |  GET /ws (Upgrade: websocket)       |
   | ----------------------------------> |
   |                                     |
   |  101 Switching Protocols            |
   | <---------------------------------- |
   |                                     |
   |  {"type":"hello","version":"1.0"}   |
   | <---------------------------------- |
   |                                     |
   |  {"type":"subscribe","channels":[]} |
   | ----------------------------------> |
   |                                     |
   |  {"type":"subscribed","channels":[]}|
   | <---------------------------------- |
   |                                     |
   |  ... events ...                     |
   | <---------------------------------- |
   |                                     |
   |  {"type":"ping"}                    |
   | ----------------------------------> |
   |  {"type":"pong"}                    |
   | <---------------------------------- |
   |                                     |
   |  close                              |
   | ----------------------------------> |
```

---

## Message Format

All messages are JSON with a common envelope:

```typescript
interface WebSocketMessage {
  type: string;           // Message type
  id?: string;            // Optional message ID for request/response correlation
  timestamp?: string;     // ISO 8601 timestamp
  payload?: unknown;      // Type-specific payload
}
```

---

## Client-to-Server Messages

### Subscribe

Subscribe to one or more event channels.

```json
{
  "type": "subscribe",
  "id": "msg-001",
  "payload": {
    "channels": ["issues", "agents", "proposals"]
  }
}
```

**Available channels:**
- `issues` - Issue create/update/delete events
- `agents` - Agent status and output events
- `proposals` - Proposal and voting events
- `epics` - Epic and stage progress events
- `workflows` - TDD and review workflow events
- `system` - System-wide events (errors, announcements)
- `agent:{id}` - Events for specific agent
- `issue:{id}` - Events for specific issue
- `epic:{id}` - Events for specific epic
- `workflow:{id}` - Events for specific workflow

### Unsubscribe

```json
{
  "type": "unsubscribe",
  "id": "msg-002",
  "payload": {
    "channels": ["agents"]
  }
}
```

### Ping

Keep-alive message.

```json
{
  "type": "ping",
  "id": "msg-003"
}
```

### Command

Execute a command (requires appropriate permissions).

```json
{
  "type": "command",
  "id": "msg-004",
  "payload": {
    "action": "pause_agent",
    "agent_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**Available commands:**
- `pause_agent` - Pause an agent
- `resume_agent` - Resume a paused agent
- `cancel_agent` - Cancel an agent's task
- `vote` - Cast a vote on a proposal
- `force_proposal` - Force approve/reject a proposal (human override)
- `advance_epic` - Manually advance epic stage

---

## Server-to-Client Messages

### Hello

Sent immediately after connection established.

```json
{
  "type": "hello",
  "timestamp": "2024-01-15T10:30:00Z",
  "payload": {
    "version": "1.0",
    "server_id": "dispatch-001",
    "capabilities": ["subscribe", "command"]
  }
}
```

### Subscribed

Confirmation of subscription.

```json
{
  "type": "subscribed",
  "id": "msg-001",
  "timestamp": "2024-01-15T10:30:01Z",
  "payload": {
    "channels": ["issues", "agents", "proposals"]
  }
}
```

### Pong

Response to ping.

```json
{
  "type": "pong",
  "id": "msg-003",
  "timestamp": "2024-01-15T10:30:02Z"
}
```

### Command Result

Response to command execution.

```json
{
  "type": "command_result",
  "id": "msg-004",
  "timestamp": "2024-01-15T10:30:03Z",
  "payload": {
    "success": true,
    "message": "Agent paused"
  }
}
```

### Error

Error response.

```json
{
  "type": "error",
  "id": "msg-004",
  "timestamp": "2024-01-15T10:30:03Z",
  "payload": {
    "code": "PERMISSION_DENIED",
    "message": "Not authorized to pause agents"
  }
}
```

---

## Event Types

### Issue Events

```json
{
  "type": "event",
  "timestamp": "2024-01-15T10:30:00Z",
  "payload": {
    "channel": "issues",
    "event": "issue_updated",
    "data": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "github_number": 42,
      "title": "Fix authentication bug",
      "status": "in_progress",
      "priority": "high",
      "assigned_agent_id": "agent-123",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  }
}
```

**Issue event types:**
- `issue_created`
- `issue_updated`
- `issue_deleted`
- `issue_assigned`
- `issue_completed`
- `issue_blocked`

### Agent Events

```json
{
  "type": "event",
  "timestamp": "2024-01-15T10:30:00Z",
  "payload": {
    "channel": "agents",
    "event": "agent_output",
    "data": {
      "agent_id": "550e8400-e29b-41d4-a716-446655440000",
      "agent_type": "coder",
      "issue_id": "issue-123",
      "output_type": "tool_use",
      "content": {
        "tool": "Edit",
        "file": "src/main.rs",
        "status": "success"
      }
    }
  }
}
```

**Agent event types:**
- `agent_started` - Agent process spawned
- `agent_output` - Agent produced output (tool use, text, etc.)
- `agent_heartbeat` - Periodic heartbeat
- `agent_paused` - Agent was paused
- `agent_resumed` - Agent was resumed
- `agent_completed` - Agent finished task
- `agent_failed` - Agent encountered error
- `agent_timeout` - Agent timed out

### Proposal Events

```json
{
  "type": "event",
  "timestamp": "2024-01-15T10:30:00Z",
  "payload": {
    "channel": "proposals",
    "event": "vote_cast",
    "data": {
      "proposal_id": "550e8400-e29b-41d4-a716-446655440000",
      "voter_id": "agent-reviewer-1",
      "voter_type": "reviewer",
      "decision": "approve",
      "current_tally": {
        "approve": 3,
        "reject": 0,
        "abstain": 1
      },
      "threshold": "simple_majority",
      "deadline": "2024-01-16T10:30:00Z"
    }
  }
}
```

**Proposal event types:**
- `proposal_created`
- `vote_cast`
- `proposal_approved`
- `proposal_rejected`
- `proposal_deadline_extended`
- `proposal_forced` - Human override

### Epic Events

```json
{
  "type": "event",
  "timestamp": "2024-01-15T10:30:00Z",
  "payload": {
    "channel": "epics",
    "event": "stage_completed",
    "data": {
      "epic_id": "550e8400-e29b-41d4-a716-446655440000",
      "epic_title": "User Authentication System",
      "stage_name": "Design",
      "stage_index": 0,
      "next_stage": "Implementation",
      "gate_type": "approval",
      "gate_status": "passed"
    }
  }
}
```

**Epic event types:**
- `epic_created`
- `epic_updated`
- `stage_started`
- `stage_completed`
- `gate_reached` - Waiting for approval
- `gate_passed`
- `gate_blocked`
- `epic_completed`

### Workflow Events

```json
{
  "type": "event",
  "timestamp": "2024-01-15T10:30:00Z",
  "payload": {
    "channel": "workflows",
    "event": "phase_changed",
    "data": {
      "workflow_id": "550e8400-e29b-41d4-a716-446655440000",
      "workflow_type": "tdd",
      "previous_phase": "WriteTests",
      "current_phase": "TestReview",
      "iteration": 1,
      "max_iterations": 3,
      "coordinator_id": "agent-coord-1"
    }
  }
}
```

**Workflow event types:**
- `workflow_started`
- `phase_changed`
- `iteration_started`
- `feedback_received`
- `review_requested`
- `review_completed`
- `workflow_completed`
- `workflow_escalated` - Max iterations reached

### System Events

```json
{
  "type": "event",
  "timestamp": "2024-01-15T10:30:00Z",
  "payload": {
    "channel": "system",
    "event": "github_sync_completed",
    "data": {
      "issues_synced": 15,
      "issues_created": 2,
      "issues_updated": 5,
      "duration_ms": 1234
    }
  }
}
```

**System event types:**
- `github_sync_started`
- `github_sync_completed`
- `github_sync_failed`
- `rate_limit_warning`
- `rate_limit_exceeded`
- `server_shutdown` - Graceful shutdown notice

---

## Key Data Structures

```rust
// dispatch-server/src/websocket/mod.rs

use tokio::sync::broadcast;
use axum::extract::ws::{Message, WebSocket};

/// Event broadcast channel
pub type EventSender = broadcast::Sender<ServerEvent>;
pub type EventReceiver = broadcast::Receiver<ServerEvent>;

/// Server-sent event
#[derive(Debug, Clone, Serialize)]
pub struct ServerEvent {
    pub channel: Channel,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Available subscription channels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Issues,
    Agents,
    Proposals,
    Epics,
    Workflows,
    System,
    Agent(AgentId),
    Issue(IssueId),
    Epic(EpicId),
    Workflow(WorkflowId),
}

/// Client-to-server message
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Subscribe {
        id: Option<String>,
        payload: SubscribePayload,
    },
    Unsubscribe {
        id: Option<String>,
        payload: UnsubscribePayload,
    },
    Ping {
        id: Option<String>,
    },
    Command {
        id: Option<String>,
        payload: CommandPayload,
    },
}

/// Server-to-client message
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Hello {
        timestamp: DateTime<Utc>,
        payload: HelloPayload,
    },
    Subscribed {
        id: Option<String>,
        timestamp: DateTime<Utc>,
        payload: SubscribedPayload,
    },
    Pong {
        id: Option<String>,
        timestamp: DateTime<Utc>,
    },
    Event {
        timestamp: DateTime<Utc>,
        payload: EventPayload,
    },
    CommandResult {
        id: Option<String>,
        timestamp: DateTime<Utc>,
        payload: CommandResultPayload,
    },
    Error {
        id: Option<String>,
        timestamp: DateTime<Utc>,
        payload: ErrorPayload,
    },
}

#[derive(Debug, Serialize)]
pub struct HelloPayload {
    pub version: String,
    pub server_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubscribePayload {
    pub channels: Vec<Channel>,
}

#[derive(Debug, Serialize)]
pub struct EventPayload {
    pub channel: Channel,
    pub event: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidMessage,
    UnknownChannel,
    PermissionDenied,
    NotFound,
    InternalError,
    RateLimited,
}
```

---

## Connection Handler

```rust
// dispatch-server/src/websocket/handler.rs

pub struct WebSocketHandler {
    event_sender: EventSender,
    db: Database,
    config: Arc<Config>,
}

impl WebSocketHandler {
    pub async fn handle_connection(
        self,
        socket: WebSocket,
        auth: Option<AuthToken>,
    ) {
        let (mut ws_sender, mut ws_receiver) = socket.split();

        // Send hello
        let hello = ServerMessage::Hello {
            timestamp: Utc::now(),
            payload: HelloPayload {
                version: "1.0".into(),
                server_id: self.config.server_id.clone(),
                capabilities: vec!["subscribe".into(), "command".into()],
            },
        };
        let _ = ws_sender.send(Message::Text(
            serde_json::to_string(&hello).unwrap()
        )).await;

        // Connection state
        let mut subscriptions: HashSet<Channel> = HashSet::new();
        let mut event_receiver = self.event_sender.subscribe();

        // Ping interval
        let mut ping_interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                // Handle incoming client messages
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            self.handle_client_message(
                                &text,
                                &mut subscriptions,
                                &mut ws_sender,
                                &auth,
                            ).await;
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        _ => {}
                    }
                }

                // Forward matching events to client
                event = event_receiver.recv() => {
                    if let Ok(event) = event {
                        if subscriptions.contains(&event.channel) {
                            let msg = ServerMessage::Event {
                                timestamp: event.timestamp,
                                payload: EventPayload {
                                    channel: event.channel,
                                    event: event.event_type,
                                    data: event.data,
                                },
                            };
                            let _ = ws_sender.send(Message::Text(
                                serde_json::to_string(&msg).unwrap()
                            )).await;
                        }
                    }
                }

                // Send periodic pings
                _ = ping_interval.tick() => {
                    let _ = ws_sender.send(Message::Ping(vec![])).await;
                }
            }
        }
    }

    async fn handle_client_message(
        &self,
        text: &str,
        subscriptions: &mut HashSet<Channel>,
        ws_sender: &mut SplitSink<WebSocket, Message>,
        auth: &Option<AuthToken>,
    ) {
        let msg: ClientMessage = match serde_json::from_str(text) {
            Ok(m) => m,
            Err(e) => {
                let error = ServerMessage::Error {
                    id: None,
                    timestamp: Utc::now(),
                    payload: ErrorPayload {
                        code: ErrorCode::InvalidMessage,
                        message: e.to_string(),
                    },
                };
                let _ = ws_sender.send(Message::Text(
                    serde_json::to_string(&error).unwrap()
                )).await;
                return;
            }
        };

        match msg {
            ClientMessage::Subscribe { id, payload } => {
                for channel in payload.channels {
                    subscriptions.insert(channel);
                }
                let response = ServerMessage::Subscribed {
                    id,
                    timestamp: Utc::now(),
                    payload: SubscribedPayload {
                        channels: subscriptions.iter().cloned().collect(),
                    },
                };
                let _ = ws_sender.send(Message::Text(
                    serde_json::to_string(&response).unwrap()
                )).await;
            }
            ClientMessage::Unsubscribe { id, payload } => {
                for channel in &payload.channels {
                    subscriptions.remove(channel);
                }
                // Send updated subscriptions
            }
            ClientMessage::Ping { id } => {
                let response = ServerMessage::Pong {
                    id,
                    timestamp: Utc::now(),
                };
                let _ = ws_sender.send(Message::Text(
                    serde_json::to_string(&response).unwrap()
                )).await;
            }
            ClientMessage::Command { id, payload } => {
                let result = self.execute_command(&payload, auth).await;
                let response = match result {
                    Ok(msg) => ServerMessage::CommandResult {
                        id,
                        timestamp: Utc::now(),
                        payload: CommandResultPayload {
                            success: true,
                            message: msg,
                        },
                    },
                    Err(e) => ServerMessage::Error {
                        id,
                        timestamp: Utc::now(),
                        payload: ErrorPayload {
                            code: e.code(),
                            message: e.to_string(),
                        },
                    },
                };
                let _ = ws_sender.send(Message::Text(
                    serde_json::to_string(&response).unwrap()
                )).await;
            }
        }
    }
}
```

---

## Event Publishing

```rust
// dispatch-core/src/events.rs

/// Central event publisher
pub struct EventPublisher {
    sender: EventSender,
}

impl EventPublisher {
    pub fn new() -> (Self, EventReceiver) {
        let (sender, receiver) = broadcast::channel(1024);
        (Self { sender }, receiver)
    }

    pub fn publish(&self, event: ServerEvent) {
        // Ignore send errors (no subscribers)
        let _ = self.sender.send(event);
    }

    pub fn issue_updated(&self, issue: &Issue) {
        self.publish(ServerEvent {
            channel: Channel::Issues,
            event_type: "issue_updated".into(),
            data: serde_json::to_value(IssueEventData::from(issue)).unwrap(),
            timestamp: Utc::now(),
        });

        // Also publish to issue-specific channel
        self.publish(ServerEvent {
            channel: Channel::Issue(issue.id.clone()),
            event_type: "issue_updated".into(),
            data: serde_json::to_value(IssueEventData::from(issue)).unwrap(),
            timestamp: Utc::now(),
        });
    }

    pub fn agent_output(&self, agent: &Agent, output: &AgentOutput) {
        self.publish(ServerEvent {
            channel: Channel::Agents,
            event_type: "agent_output".into(),
            data: serde_json::to_value(AgentOutputEventData {
                agent_id: agent.id.clone(),
                agent_type: agent.agent_type,
                issue_id: agent.issue_id.clone(),
                output: output.clone(),
            }).unwrap(),
            timestamp: Utc::now(),
        });

        // Also publish to agent-specific channel
        self.publish(ServerEvent {
            channel: Channel::Agent(agent.id.clone()),
            event_type: "agent_output".into(),
            data: serde_json::to_value(output).unwrap(),
            timestamp: Utc::now(),
        });
    }

    // ... other event methods
}
```

---

## Reconnection Handling

Clients should implement automatic reconnection with exponential backoff:

```typescript
// Example client implementation

class DispatchWebSocket {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private baseDelay = 1000;
  private maxDelay = 30000;
  private subscriptions: string[] = [];

  connect(url: string) {
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      console.log('Connected');
      this.reconnectAttempts = 0;

      // Resubscribe to channels
      if (this.subscriptions.length > 0) {
        this.send({
          type: 'subscribe',
          payload: { channels: this.subscriptions }
        });
      }
    };

    this.ws.onclose = (event) => {
      if (!event.wasClean) {
        this.scheduleReconnect();
      }
    };

    this.ws.onerror = () => {
      this.scheduleReconnect();
    };

    this.ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      this.handleMessage(msg);
    };
  }

  private scheduleReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnect attempts reached');
      return;
    }

    const delay = Math.min(
      this.baseDelay * Math.pow(2, this.reconnectAttempts),
      this.maxDelay
    );

    this.reconnectAttempts++;

    setTimeout(() => {
      console.log(`Reconnecting (attempt ${this.reconnectAttempts})...`);
      this.connect(this.url);
    }, delay);
  }

  subscribe(channels: string[]) {
    this.subscriptions = [...new Set([...this.subscriptions, ...channels])];

    if (this.ws?.readyState === WebSocket.OPEN) {
      this.send({
        type: 'subscribe',
        payload: { channels }
      });
    }
  }

  private send(msg: object) {
    this.ws?.send(JSON.stringify(msg));
  }
}
```

---

## Authentication

WebSocket connections can be authenticated via:

1. **Query parameter**: `ws://localhost:8080/ws?token=<jwt>`
2. **Cookie**: Session cookie from REST API login
3. **First message**: Send auth token as first message after connect

```rust
// dispatch-server/src/websocket/auth.rs

pub async fn authenticate_ws(
    headers: &HeaderMap,
    query: &Query<WsQueryParams>,
) -> Option<AuthToken> {
    // Try query parameter
    if let Some(token) = &query.token {
        if let Ok(auth) = validate_token(token) {
            return Some(auth);
        }
    }

    // Try cookie
    if let Some(cookie) = headers.get(COOKIE) {
        if let Ok(session) = extract_session_cookie(cookie) {
            if let Ok(auth) = validate_session(&session).await {
                return Some(auth);
            }
        }
    }

    // No auth - allow with limited permissions
    None
}
```

---

## Rate Limiting

```rust
// dispatch-server/src/websocket/rate_limit.rs

pub struct WsRateLimiter {
    /// Max messages per second per connection
    messages_per_second: u32,
    /// Max subscriptions per connection
    max_subscriptions: usize,
    /// Max connections per IP
    max_connections_per_ip: usize,
}

impl WsRateLimiter {
    pub fn check_message_rate(&self, state: &mut ConnectionState) -> Result<()> {
        let now = Instant::now();
        state.message_times.retain(|t| now.duration_since(*t) < Duration::from_secs(1));

        if state.message_times.len() >= self.messages_per_second as usize {
            return Err(DispatchError::RateLimited {
                retry_after: Duration::from_secs(1),
            });
        }

        state.message_times.push(now);
        Ok(())
    }

    pub fn check_subscriptions(&self, count: usize) -> Result<()> {
        if count > self.max_subscriptions {
            return Err(DispatchError::TooManySubscriptions {
                max: self.max_subscriptions,
            });
        }
        Ok(())
    }
}
```

---

## Database Schema

```sql
-- Track active WebSocket connections for debugging/monitoring
CREATE TABLE ws_connections (
    id TEXT PRIMARY KEY,
    client_ip TEXT NOT NULL,
    user_id TEXT,                    -- NULL for anonymous
    connected_at TEXT NOT NULL,
    last_activity TEXT NOT NULL,
    subscriptions TEXT NOT NULL,     -- JSON array of channels
    message_count INTEGER DEFAULT 0
);

-- Index for cleanup queries
CREATE INDEX idx_ws_connections_activity ON ws_connections(last_activity);
```

---

## Axum Router Integration

```rust
// dispatch-server/src/routes.rs

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        // ... other routes
        .with_state(state)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<WsQueryParams>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let auth = authenticate_ws(&headers, &params).await;

    ws.on_upgrade(move |socket| {
        let handler = WebSocketHandler::new(
            state.event_sender.clone(),
            state.db.clone(),
            state.config.clone(),
        );
        handler.handle_connection(socket, auth)
    })
}
```

---

## Testing

```rust
#[cfg(test)]
mod tests {
    use tokio_tungstenite::connect_async;

    #[tokio::test]
    async fn test_websocket_connection() {
        let app = create_test_app().await;
        let addr = start_test_server(app).await;

        let (mut ws, _) = connect_async(format!("ws://{}/ws", addr))
            .await
            .expect("Failed to connect");

        // Should receive hello message
        let msg = ws.next().await.unwrap().unwrap();
        let hello: ServerMessage = serde_json::from_str(&msg.to_text().unwrap()).unwrap();

        assert!(matches!(hello, ServerMessage::Hello { .. }));
    }

    #[tokio::test]
    async fn test_subscription() {
        let (mut ws, event_sender) = setup_ws_test().await;

        // Subscribe to issues
        ws.send(Message::Text(serde_json::to_string(&ClientMessage::Subscribe {
            id: Some("1".into()),
            payload: SubscribePayload {
                channels: vec![Channel::Issues],
            },
        }).unwrap())).await.unwrap();

        // Should receive subscribed confirmation
        let msg = receive_msg(&mut ws).await;
        assert!(matches!(msg, ServerMessage::Subscribed { .. }));

        // Publish an event
        event_sender.send(ServerEvent {
            channel: Channel::Issues,
            event_type: "issue_created".into(),
            data: json!({"id": "test-123"}),
            timestamp: Utc::now(),
        }).unwrap();

        // Should receive the event
        let msg = receive_msg(&mut ws).await;
        assert!(matches!(msg, ServerMessage::Event { .. }));
    }
}
```

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-010 | WebSocket infrastructure | `dispatch-server/src/websocket/mod.rs`, `handler.rs` |
| PR-010a | Event publishing system | `dispatch-core/src/events.rs` |
| PR-010b | WebSocket authentication | `dispatch-server/src/websocket/auth.rs` |
| PR-010c | Client library (TypeScript) | `web/src/lib/websocket.ts` |
