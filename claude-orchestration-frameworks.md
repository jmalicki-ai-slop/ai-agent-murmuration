# Claude Orchestration Frameworks Comparison

A comprehensive comparison of multi-agent orchestration frameworks for Claude, focusing on specialized agents, task management, and web interfaces.

---

## Executive Summary

| Framework | Web UI | Multi-Agent | Ticketing/Tasks | Self-Hosted | Mobile | Best For |
|-----------|--------|-------------|-----------------|-------------|--------|----------|
| **Shrimp Task Manager** | No | Limited | Strong | Yes | Via Happy/Web UI | Task decomposition & persistence |
| **Claude Flow** | No | Strong (64 agents) | Moderate | Yes | Via Happy/Web UI | Enterprise swarm intelligence |
| **wshobson/agents** | No | Strong (85 agents) | Via orchestrators | Yes | Via Happy/Web UI | Comprehensive agent library |
| **SuperClaude** | No | 16 agents | Via /sc:pm | Yes | Via Happy/Web UI | Development workflow enhancement |
| **Claude MPM** | Dashboard | 15 agents | Yes (Ticketing Agent) | Yes | Dashboard (web) | Project management focus |
| **Agentwise** | Web Dashboard | 8+ agents | Yes | Yes | Dashboard (web) | Real-time monitoring |
| **Remote-Code** | Full Web UI | Yes | Yes | Yes | Web (native in dev) | Remote access & mobile support |
| **BMAD** | Web UI (Planning) | Agile team roles | Story-based | Yes | Planning UI (web) | Agile methodology |

---

## Detailed Framework Analysis

### 1. Shrimp Task Manager (MCP Server)

**GitHub:** https://github.com/cjo4m06/mcp-shrimp-task-manager

**Overview:**
An MCP server emphasizing chain-of-thought, reflection, and style consistency. Converts natural language into structured dev tasks with dependency tracking.

**Pros:**
- Persistent memory across sessions (no context loss)
- Intelligent task decomposition with dependency tracking
- Long-term memory with historical reference for planning
- Lightweight MCP server integration
- Continuous mode for autonomous sequential execution
- Works with Claude Desktop and Claude Code

**Cons:**
- No web interface
- Limited multi-agent coordination (single agent focus)
- No specialized agent roles built-in
- No real-time dashboard or monitoring
- Requires manual MCP configuration

**Best For:** Individual developers wanting persistent task management and structured workflows without the overhead of full orchestration platforms.

---

### 2. Claude Flow

**GitHub:** https://github.com/ruvnet/claude-flow

**Overview:**
Enterprise-grade AI orchestration platform with hive-mind swarm intelligence, supporting up to 64 specialized agents.

**Pros:**
- Scalable swarm architecture (up to 64 agents)
- Two coordination modes: Swarm (fast) and Hive-Mind (persistent)
- 100+ MCP tools included
- 25 natural language-activated skills
- Hybrid memory system (AgentDB + SQLite)
- Excellent performance (96x-164x faster vector search)
- Dynamic Agent Architecture with fault tolerance
- Pre/post operation hooks for automation

**Cons:**
- No web UI (CLI-focused)
- Steep learning curve for enterprise features
- Complex configuration for advanced setups
- Heavy resource requirements for full swarm deployment
- Documentation can be overwhelming

**Best For:** Enterprise teams needing scalable multi-agent swarms with persistent memory and complex workflow orchestration.

---

### 3. wshobson/agents

**GitHub:** https://github.com/wshobson/agents

**Overview:**
Comprehensive agent library with 85 specialized agents, 15 workflow orchestrators, 47 skills, organized into 63 plugins.

**Pros:**
- Massive agent library (85 specialized agents)
- Granular plugin system (average 3.4 components per plugin)
- Progressive disclosure (metadata → instructions → resources)
- Strategic model distribution (47 Haiku, 97 Sonnet agents)
- 15 workflow orchestrators for complex multi-step processes
- Covers 14+ categories (architecture, security, data/AI, DevOps, etc.)
- Minimal token usage through selective loading

**Cons:**
- No web interface
- No built-in ticketing system
- Requires understanding of plugin marketplace
- Can be overwhelming to choose appropriate agents
- No real-time monitoring dashboard

**Best For:** Teams wanting a comprehensive library of specialized agents with flexible, modular activation.

---

### 4. SuperClaude Framework

**GitHub:** https://github.com/SuperClaude-Org/SuperClaude_Framework

**Overview:**
Meta-programming configuration framework with 30 slash commands, 16 agents, and 7 behavioral modes.

**Pros:**
- Easy slash command interface (/sc:research, /sc:implement, etc.)
- 16 specialized agents with automatic context-based coordination
- 7 adaptive behavioral modes
- 8 integrated MCP servers (Tavily, Playwright, etc.)
- Deep research with multi-hop reasoning
- Token efficiency mode (30-50% savings)
- Case-based learning across sessions
- Simple installation via pipx

**Cons:**
- No web interface
- No explicit ticketing/task management system
- Limited multi-agent parallelism
- TypeScript plugin system still in development
- Requires MCP servers for full performance

**Best For:** Individual developers wanting enhanced Claude Code with structured commands and intelligent research capabilities.

---

### 5. Claude MPM (Multi-Agent Project Manager)

**GitHub:** https://github.com/bobmatnyc/claude-mpm

**Overview:**
Orchestration framework with 15+ specialized agents, real-time monitoring dashboard, and PM-driven task routing.

**Pros:**
- **Built-in Ticketing Agent** for issue tracking
- Real-time monitoring dashboard (--monitor flag)
- 15 specialized agents including:
  - Development: Engineer, Research, Documentation, QA, Security
  - Language-specific: Python Engineer, Rust Engineer
  - Infrastructure: Ops, Version Control, Data Engineer
  - Web: Web UI, Web QA
  - Management: Ticketing Agent, Project Organizer
- PM agent for intelligent task routing
- Interface-based service contracts
- Streamlined codebase (removed 3,700 lines of complexity)

**Cons:**
- Dashboard requires optional [monitor] extra
- No full web UI (dashboard is monitoring-focused)
- Smaller agent count than some alternatives
- Less focus on swarm/parallel execution
- Rich TUI may not suit all workflows

**Best For:** Teams wanting PM-driven workflows with ticketing integration and real-time monitoring.

---

### 6. Agentwise

**GitHub:** https://vibecodingwithphil.github.io/agentwise/

**Overview:**
Multi-agent orchestration with live WebSocket dashboard, 8+ parallel agents, and self-improving architecture.

**Pros:**
- **Live Web Dashboard** with WebSocket integration
- Real-time token usage and performance tracking
- 8+ specialized parallel agents (Frontend, Backend, Database, DevOps, Testing)
- Dynamic agent generation (/generate-agent)
- Self-improving with persistent knowledge base
- 27+ MCP server integrations
- Organized context management
- Modern dark-themed UI

**Cons:**
- Requires --dangerously-skip-permissions flag
- Fewer agents than larger frameworks
- Less mature than established alternatives
- Node.js 18+ requirement
- Documentation could be more comprehensive

**Best For:** Teams wanting real-time visual monitoring with parallel agent execution and continuous learning.

---

### 7. Remote-Code

**GitHub:** https://github.com/vanna-ai/remote-code

**Overview:**
Full web-based development environment for managing multiple AI agents from anywhere, with mobile support.

**Pros:**
- **Full Web UI** accessible from any device (desktop, tablet, mobile)
- Self-hosted with data sovereignty
- Cloudflare tunnel & ngrok support for secure remote access
- Agents operate in isolated tmux sessions
- Live terminal access via WebSocket
- Mid-execution command injection
- Project and task organization across repositories
- Automatic agent detection and configuration
- Mobile-responsive interface

**Cons:**
- Less specialized agent types than dedicated frameworks
- Requires Go, SQLite, and tmux infrastructure
- Tunnel setup adds complexity
- Fewer built-in orchestration patterns
- Mobile app still in development

**Best For:** Remote teams or developers needing web-accessible agent management from any device.

---

### 8. BMAD (Breakthrough Method for Agile AI Driven Development)

**GitHub:** https://github.com/24601/BMAD-AT-CLAUDE

**Overview:**
Full agile team simulation with specialized AI agents for Product Manager, Architect, Scrum Master, and more.

**Pros:**
- **Agile methodology** built-in (PRD, Architecture docs, Story files)
- Specialized role-based agents:
  - Product Manager
  - Architect
  - Scrum Master
  - Developer
  - QA
- Web UI for planning workflow
- Story-based task management
- Domain-agnostic (works beyond software)
- Expansion packs for creative writing, business strategy, etc.

**Cons:**
- Methodology-opinionated (must follow BMAD process)
- Less flexible for non-agile workflows
- Smaller community than some alternatives
- Web UI primarily for planning (not real-time monitoring)
- Requires buying into the full BMAD approach

**Best For:** Teams wanting structured agile methodology with clear role separation and story-based development.

---

## Mobile & Remote Access

Most Claude orchestration frameworks are CLI-focused without native mobile support. However, several solutions enable mobile access:

### Third-Party Mobile Solutions

#### 1. Happy (Recommended for Mobile)

**Website:** https://happy.engineering/ | **GitHub:** https://github.com/slopus/happy

Native mobile client for Claude Code with end-to-end encryption.

**Platforms:** iOS (App Store), Android, Web

**Key Features:**
- Signal-protocol encryption (keys never leave device)
- Push notifications for permissions/errors
- Voice control for hands-free coding
- Seamless device switching
- Works with any orchestration framework

**Installation:** `npm install -g happy-coder` then run `happy` instead of `claude`

**Pros:**
- Native apps for both iOS and Android
- Best security (E2E encryption)
- Voice control support
- Works with any Claude Code setup

**Cons:**
- Requires CLI running on a computer
- Third-party (not Anthropic)
- Depends on their relay server (encrypted)

---

#### 2. Claude Code UI (Siteboon)

**GitHub:** https://github.com/siteboon/claudecodeui

Open-source mobile-responsive web UI for Claude Code.

**Platforms:** Any browser (mobile-optimized)

**Key Features:**
- Touch-friendly with swipe gestures
- Bottom tab navigation for thumb access
- File explorer with syntax highlighting
- Git integration (stage, commit, branch)
- Session management across projects
- TaskMaster AI integration

**Installation:**
```bash
git clone https://github.com/siteboon/claudecodeui.git
cd claudecodeui && npm install
npm run dev
```

**Pros:**
- Fully open source
- Rich feature set (file explorer, Git, sessions)
- Self-hosted (full control)
- No native app needed

**Cons:**
- Requires self-hosting setup
- No push notifications
- Web-only (no native features)

---

#### 3. CodeRemote

**Website:** https://coderemote.dev/

Commercial self-hosted solution using Tailscale VPN.

**Platforms:** Any browser

**Key Features:**
- Peer-to-peer VPN (no cloud servers)
- Fully self-hosted on your machine
- Secure by default (Tailscale encryption)

**Pros:**
- No relay servers (direct P2P)
- Enterprise-grade security
- Simple setup with Tailscale

**Cons:**
- Commercial product
- Requires Tailscale account
- Less feature-rich than alternatives

---

#### 4. Official Claude Code on Web

**Access:** claude.ai (Pro/Max plans)

Anthropic's native solution for web/mobile Claude Code.

**Platforms:** Web, iOS app

**Key Features:**
- Runs on Anthropic-managed infrastructure
- iOS app support (Android via web browser)
- Remote MCP servers sync across devices
- No self-hosting required

**Pros:**
- Official Anthropic solution
- No setup required
- Syncs across all devices
- iOS native app

**Cons:**
- Requires Pro ($20/mo) or Max ($100-200/mo) plan
- No dedicated Android app
- Code runs on Anthropic infrastructure (not local)
- Less control than self-hosted solutions

---

### Framework Mobile Compatibility Matrix

| Framework | Native App | Mobile Web | Remote Access | Best Mobile Solution |
|-----------|-----------|------------|---------------|---------------------|
| Shrimp Task Manager | No | No | No | Happy or Claude Code UI |
| Claude Flow | No | No | No | Happy or Claude Code UI |
| wshobson/agents | No | No | No | Happy or Claude Code UI |
| SuperClaude | No | No | No | Happy or Claude Code UI |
| Claude MPM | No | Dashboard only | Via --monitor | Happy + MPM Dashboard |
| Agentwise | No | Dashboard | WebSocket | Agentwise Dashboard |
| Remote-Code | In development | Yes | Cloudflare/ngrok | Native (mobile-responsive) |
| BMAD | No | Planning UI | Via Claude web | BMAD Web + Happy |

---

## Task Coordination, Ticketing & Workflow Orchestration

This section provides deep analysis of how each framework handles task management, agent coordination, and workflow orchestration—critical for teams wanting structured multi-agent development.

### Coordination Patterns Overview

| Pattern | Description | Best For | Frameworks |
|---------|-------------|----------|------------|
| **PM-Driven Routing** | Central PM agent analyzes tasks and routes to specialists | Teams with diverse task types | Claude MPM |
| **Story-Based (Agile)** | Scrum Master breaks epics into developer stories | Agile teams, large features | BMAD |
| **Dependency Graph** | Tasks tracked with explicit dependencies | Complex projects with ordering needs | Shrimp, Claude Flow |
| **Swarm/Parallel** | Multiple agents work simultaneously | High-throughput, independent tasks | Claude Flow, Agentwise |
| **Orchestrator-Worker** | Lead agent coordinates specialized subagents | Research, multi-step analysis | Anthropic pattern, wshobson |

---

### Detailed Framework Analysis: Task Management

#### Shrimp Task Manager: Dependency-Driven Task Decomposition

**Task Model:**
- Natural language → structured atomic tasks
- Explicit dependency tracking between tasks
- Persistent state across sessions

**Workflow:**
```
1. plan task: [description]     → AI analyzes & creates subtasks
2. list tasks                   → View all tasks with status
3. execute task [id]            → Run single task
4. continuous mode              → Auto-execute all tasks sequentially
5. reflect task [id]            → Review and improve task
```

**Task States:** pending → in_progress → completed/blocked

**Strengths:**
- Best-in-class dependency management
- Long-term memory (references past similar tasks)
- Research mode for technical investigation
- Web-based Task Viewer with drag-and-drop

**Limitations:**
- Single-agent focus (no parallel execution)
- No built-in agent specialization
- No ticketing/issue integration

**Best For:** Solo developers needing persistent, structured task tracking with intelligent decomposition.

---

#### Claude MPM: PM-Driven Agent Routing

**Task Model:**
- PM agent as central coordinator
- Smart routing to 15+ specialized agents
- Session-based context preservation

**Agent Specialization:**
| Role | Agent | Responsibility |
|------|-------|----------------|
| **Ticketing** | ticketing-agent | Issue tracking, ticket management |
| **Development** | engineer, python-engineer, rust-engineer | Code implementation |
| **Quality** | qa-agent, security-agent | Testing, vulnerability assessment |
| **Operations** | ops-agent, version-control | Deployment, Git management |
| **Documentation** | documentation-agent | Docs creation and maintenance |
| **Research** | research-agent | Code analysis, investigation |
| **Organization** | project-organizer | File structure, project layout |

**Workflow:**
```
1. User submits task to PM agent
2. PM analyzes task requirements
3. PM routes to appropriate specialist(s)
4. Specialist executes with domain expertise
5. Results flow back through PM
6. Session context preserved for continuation
```

**Ticketing Integration:**
- Dedicated Ticketing Agent for issue tracking
- Integrates with project knowledge base
- Session logs at 70%/85%/95% token thresholds

**Strengths:**
- Built-in ticketing system
- Intelligent task routing
- Real-time monitoring dashboard (--monitor)
- Session resume capability

**Limitations:**
- Sequential execution (limited parallelism)
- Requires learning agent specializations
- Dashboard requires optional install

**Best For:** Teams wanting PM-style task management with ticketing and specialized agent roles.

---

#### Claude Flow: Swarm-Based Parallel Orchestration

**Task Model:**
- Workflow definitions with dependency graphs
- Up to 64 agents in coordinated swarms
- Five execution strategies

**Execution Strategies:**
| Strategy | Description | Use Case |
|----------|-------------|----------|
| **Parallel** | Independent tasks run simultaneously (up to 8 concurrent) | Independent features |
| **Sequential** | Tasks execute in dependency order | Ordered workflows |
| **Adaptive** | Dynamically adjusts based on resources | Variable load |
| **Balanced** | Distributes load evenly across agents | Long-running projects |
| **Stream-Chained** | Real-time output piping between agents (40-60% faster) | Pipeline processing |

**Decomposition Approaches:**
- **Functional:** By domain (frontend, backend, infra, testing)
- **Layer-Based:** By architecture tier (presentation, business, data)
- **Feature-Based:** By user capability (auth, payments, admin)

**Advanced Patterns:**
- **Map-Reduce:** Parallel processing with result aggregation
- **Fork-Join:** Split work, then consolidate outputs
- **Pipeline:** Chain operations through sequential stages

**Specialized Task Agents:**
- `pr-manager` - Pull request management
- `code-review-swarm` - Multi-agent code review
- `issue-tracker` - Issue tracking
- `release-manager` - Release coordination
- `workflow-automation` - Custom workflow automation

**Strengths:**
- Highest parallelism (64 agents)
- Sophisticated execution strategies
- Built-in checkpoints and rollback
- Stream-chaining for fast pipelines

**Limitations:**
- Complex configuration
- High token consumption (15x vs chat)
- Steep learning curve
- No web UI

**Best For:** Enterprise teams needing high-throughput parallel execution with complex workflow orchestration.

---

#### BMAD: Agile Story-Based Coordination

**Task Model:**
- Agile methodology with AI agents as team roles
- Epic → Story decomposition
- Hyper-detailed story files

**Agent Roles:**
| Phase | Agent | Responsibility |
|-------|-------|----------------|
| **Analysis** | Analyst | Requirements gathering, research |
| **Planning** | PM | PRD creation, feature prioritization |
| **Solutioning** | Architect | Technical design, architecture docs |
| **Implementation** | Scrum Master | Epic sharding, story creation |
| **Development** | Developer | Code implementation |
| **Quality** | Test Architect | Test strategy, QA |
| **Design** | UX Expert | User experience design |

**Story Structure:**
```markdown
# Story: [Title]
## Description
[What needs to be built]

## Dependencies
- Story: [other-story-id]
- Files: [relevant files]
- Models: [data models]

## Tasks
1. [ ] Task 1 (max 1 dev-day)
2. [ ] Task 2
3. [ ] Task 3

## Prerequisites Checklist
- [ ] Prerequisite 1
- [ ] Prerequisite 2

## Acceptance Criteria
- [ ] Criteria 1
- [ ] Criteria 2
```

**Workflow Phases:**
```
Phase 1: Analysis    → Analyst gathers requirements
Phase 2: Planning    → PM creates PRD
Phase 3: Solutioning → Architect designs system
Phase 4: Implementation → Scrum Master creates stories
                       → Developer implements
                       → QA validates
```

**Epic Sharding Process:**
1. PRD broken into focused epics
2. Scrum Master transforms epics into stories
3. Each story contains full context for Dev agent
4. Stories are independent, testable, incremental

**Strengths:**
- True agile methodology
- Self-contained stories (no context loss)
- Clear role separation
- Works across Claude, Cursor, Copilot, Gemini

**Limitations:**
- Methodology-opinionated
- Requires full buy-in to process
- No real-time monitoring
- Sequential agent handoffs

**Best For:** Teams wanting structured agile workflow with clear phase separation and comprehensive documentation.

---

#### wshobson/agents: Orchestrator-Based Workflows

**Task Model:**
- 15 workflow orchestrators coordinate specialized agents
- Plugin-based activation (minimal token usage)
- Progressive disclosure (metadata → instructions → resources)

**Workflow Orchestrators:**
| Orchestrator | Coordinates | Purpose |
|--------------|-------------|---------|
| Full-Stack Feature | Backend → DB → Frontend → Test → Security → Deploy | End-to-end features |
| Security Hardening | Security auditors, penetration testing | Security assessments |
| ML Pipeline | Data engineers, ML specialists | Machine learning workflows |
| Incident Response | Ops, Security, Documentation | Production incidents |
| Code Review | QA, Security, Architecture | Comprehensive review |

**Example: Full-Stack Feature Orchestration:**
```
1. Backend Architect    → API design
2. Database Architect   → Schema design
3. Frontend Developer   → UI implementation
4. Test Automator       → Test suite
5. Security Auditor     → Vulnerability check
6. Deployment Engineer  → Release preparation
```

**Agent Assignment:**
- 47 Haiku agents (fast, deterministic)
- 97 Sonnet agents (complex reasoning)
- Auto-selection based on task context

**Strengths:**
- Largest agent library (85 agents)
- Pre-built orchestration workflows
- Minimal token overhead
- Covers 14+ categories

**Limitations:**
- No built-in ticketing
- No web interface
- Requires plugin marketplace knowledge
- Can be overwhelming

**Best For:** Teams wanting comprehensive pre-built workflows with specialized agents across many domains.

---

#### Agentwise: Self-Improving Parallel Coordination

**Task Model:**
- 8+ parallel agents with knowledge persistence
- Dynamic agent generation
- WebSocket-based real-time coordination

**Core Commands:**
| Command | Purpose |
|---------|---------|
| `/task` | Add feature to active project |
| `/create` | Create new project |
| `/generate-agent` | Create custom specialized agent |

**Parallel Agent Types:**
- Frontend Agent
- Backend Agent
- Database Agent
- DevOps Agent
- Testing Agent
- Custom agents (generated on demand)

**Self-Improvement System:**
- Agents learn from every task
- Persistent knowledge base
- Continuous adaptation
- Cross-project learning

**Strengths:**
- Self-improving agents
- Real-time WebSocket dashboard
- Dynamic agent creation
- Live performance monitoring

**Limitations:**
- Fewer built-in agents than alternatives
- Less mature ecosystem
- Limited documentation
- Requires --dangerously-skip-permissions

**Best For:** Teams wanting adaptive, self-improving agents with real-time visibility.

---

### Task Coordination Comparison Matrix

| Feature | Shrimp | Claude MPM | Claude Flow | BMAD | wshobson | Agentwise |
|---------|--------|------------|-------------|------|----------|-----------|
| **Coordination Model** | Dependency graph | PM routing | Swarm parallel | Agile stories | Orchestrators | Parallel + learning |
| **Max Parallel Agents** | 1 | Limited | 64 | Sequential | Via orchestrator | 8+ |
| **Task Decomposition** | Automatic | PM-driven | Configurable | Epic sharding | Orchestrator-defined | Dynamic |
| **Dependency Tracking** | Strong | Moderate | Strong | Story-based | Workflow-based | Moderate |
| **Ticketing/Issues** | No | Yes (agent) | Yes (agent) | Stories | No | Via tasks |
| **Persistent Memory** | Yes | Sessions | Hive-Mind | Story files | No | Knowledge base |
| **Real-time Monitoring** | Task Viewer | Dashboard | No | No | No | WebSocket dashboard |
| **Self-Improvement** | Historical reference | No | No | No | No | Yes |
| **Methodology** | Flexible | PM-style | Enterprise | Agile | Domain-specific | Adaptive |

---

### Token Economics of Multi-Agent Systems

Understanding token costs is critical for choosing the right coordination approach:

| System Type | Token Multiplier | When to Use |
|-------------|------------------|-------------|
| Single agent chat | 1x (baseline) | Simple queries, quick tasks |
| Single agent with tools | ~4x | Complex tasks, file operations |
| Multi-agent orchestration | ~15x | High-value, parallelizable tasks |

**Cost-Effective Strategies:**
1. **Use Shrimp** for persistent task tracking without agent overhead
2. **Use Claude MPM** when you need routing but not heavy parallelism
3. **Use Claude Flow** only for truly parallelizable, high-value tasks
4. **Use BMAD** when documentation value justifies coordination cost

---

### Choosing Your Coordination Approach

**Decision Tree:**

```
Do you need persistent task memory across sessions?
├── Yes → Consider Shrimp Task Manager
└── No ↓

Do you need ticketing/issue tracking?
├── Yes → Consider Claude MPM or Remote-Code
└── No ↓

Do you need parallel execution (>3 agents)?
├── Yes → Consider Claude Flow or Agentwise
└── No ↓

Do you follow Agile methodology?
├── Yes → Consider BMAD
└── No ↓

Do you need specialized domain agents?
├── Yes → Consider wshobson/agents
└── No → Start with SuperClaude for enhanced Claude Code
```

---

## Feature Comparison Matrix

### Multi-Agent Capabilities

| Feature | Shrimp | Claude Flow | wshobson | SuperClaude | Claude MPM | Agentwise | Remote-Code | BMAD |
|---------|--------|-------------|----------|-------------|------------|-----------|-------------|------|
| Parallel Agents | No | 64 max | 85 available | 16 | 15 | 8+ | Yes | ~5 roles |
| Agent Specialization | No | High | Very High | High | High | Moderate | Moderate | Role-based |
| Dynamic Agent Creation | No | Yes | No | No | No | Yes | No | No |
| Self-Improving | No | No | No | Case-based | No | Yes | No | No |

### Task Management

| Feature | Shrimp | Claude Flow | wshobson | SuperClaude | Claude MPM | Agentwise | Remote-Code | BMAD |
|---------|--------|-------------|----------|-------------|------------|-----------|-------------|------|
| Task Decomposition | Strong | Yes | Via orchestrators | Via commands | PM-driven | Yes | Yes | Story-based |
| Dependency Tracking | Yes | Yes | Yes | Limited | Yes | Yes | Yes | Yes |
| Persistent Memory | Yes | Hive-Mind mode | No | Case-based | Yes | Yes | SQLite | Yes |
| Ticketing System | No | No | No | No | Yes | Yes | Yes | Stories |

### Web Interface & Accessibility

| Feature | Shrimp | Claude Flow | wshobson | SuperClaude | Claude MPM | Agentwise | Remote-Code | BMAD |
|---------|--------|-------------|----------|-------------|------------|-----------|-------------|------|
| Web UI | Task Viewer | No | No | No | Dashboard | Dashboard | Full Web | Planning UI |
| Real-time Monitoring | No | No | No | No | Yes | Yes (WebSocket) | Yes | No |
| Native Mobile App | No | No | No | No | No | No | In dev | No |
| Mobile Web | Via Happy | Via Happy | Via Happy | Via Happy | Dashboard | Dashboard | Yes | Planning UI |
| Remote Access | Via Happy | Via Happy | Via Happy | Via Happy | --monitor | WebSocket | Tunnels | Via Claude web |
| Third-Party Mobile | Happy, Claude Code UI | Happy, Claude Code UI | Happy, Claude Code UI | Happy, Claude Code UI | Happy + Dashboard | Native dashboard | Native | Happy + BMAD |

---

## Recommendations by Use Case

### For Your Requirements (Multi-Agent + Ticketing + Specialized Roles + Web UI)

Based on your stated needs:
1. Multiple agents working together through ticketing
2. Specialized agents (coders, PMs, reviewers for quality/security/packaging)
3. Web interface for remote access

**Top Recommendations:**

1. **Remote-Code** - Best web interface with mobile support and self-hosting
2. **Claude MPM** - Best ticketing integration with PM-driven routing
3. **Agentwise** - Best real-time dashboard with self-improving agents

**Hybrid Approach:**
Consider combining:
- **Shrimp Task Manager** for task decomposition and persistence
- **Claude MPM** or **Agentwise** for specialized agent orchestration
- **Remote-Code** for web interface layer

### By Team Size

| Team Size | Recommended Frameworks |
|-----------|----------------------|
| Solo Developer | Shrimp, SuperClaude |
| Small Team (2-5) | Claude MPM, Agentwise |
| Medium Team (5-15) | Claude Flow, Remote-Code |
| Enterprise (15+) | Claude Flow (Hive-Mind), Custom hybrid |

### By Primary Need

| Need | Best Choice |
|------|-------------|
| Task Persistence | Shrimp Task Manager |
| Agent Scale | Claude Flow (64) or wshobson (85) |
| Web Dashboard | Remote-Code or Agentwise |
| Agile Methodology | BMAD |
| Code Review Focus | wshobson/agents (security auditor, QA agents) |
| Project Management | Claude MPM |
| Ticketing System | Claude MPM (built-in) or Remote-Code |
| Parallel Execution | Claude Flow or Agentwise |

### For Mobile/Remote Access

**Best Native Mobile Experience:**
1. **Happy** + any framework - Native iOS/Android apps with E2E encryption, voice control
2. **Remote-Code** - Mobile-responsive web (native app in development)

**Best Mobile Web Experience:**
1. **Remote-Code** - Purpose-built for remote access with Cloudflare/ngrok tunnels
2. **Agentwise** - WebSocket dashboard works on mobile browsers
3. **Claude Code UI (Siteboon)** - Mobile-optimized open-source UI for any framework

**For Android Specifically:**
| Solution | Type | Best For |
|----------|------|----------|
| **Happy** | Native Android app | Best overall mobile experience |
| **Remote-Code** | Mobile web | Self-hosted remote access |
| **Claude Code UI** | Mobile web | Open-source, feature-rich |
| **Official Claude.ai** | Mobile web | Pro/Max plan users |
| **Agentwise Dashboard** | Mobile web | Real-time monitoring |

**Recommended Mobile + Framework Combinations:**
- **Solo dev, task-focused:** Shrimp + Happy
- **Team with ticketing:** Claude MPM + Happy (for mobile) + Dashboard (for monitoring)
- **Remote-first team:** Remote-Code (native mobile web)
- **Agile team:** BMAD + Happy + Claude web
- **Enterprise parallel:** Claude Flow + Happy (mobile oversight)

---

## Installation Quick Reference

```bash
# Shrimp Task Manager
npx -y @smithery/cli install @cjo4m06/mcp-shrimp-task-manager --client claude

# SuperClaude
pipx install superclaude && superclaude install

# Claude Flow
# See: https://github.com/ruvnet/claude-flow

# wshobson/agents
/plugin marketplace add wshobson/agents

# Agentwise
npm install agentwise

# Remote-Code
# See: https://github.com/vanna-ai/remote-code
```

---

## Sources

### Orchestration Frameworks
- [Shrimp Task Manager](https://github.com/cjo4m06/mcp-shrimp-task-manager)
- [Claude Flow](https://github.com/ruvnet/claude-flow)
- [Claude Flow Wiki - Workflow Orchestration](https://github.com/ruvnet/claude-flow/wiki/Workflow-Orchestration)
- [wshobson/agents](https://github.com/wshobson/agents)
- [SuperClaude Framework](https://github.com/SuperClaude-Org/SuperClaude_Framework)
- [Claude MPM](https://github.com/bobmatnyc/claude-mpm)
- [Agentwise](https://vibecodingwithphil.github.io/agentwise/)
- [Remote-Code](https://github.com/vanna-ai/remote-code)
- [BMAD Method](https://github.com/24601/BMAD-AT-CLAUDE)
- [BMAD MCP Server](https://pypi.org/project/bmad-mcp-server/)

### Mobile Solutions
- [Happy - Claude Code Mobile Client](https://happy.engineering/)
- [Happy GitHub](https://github.com/slopus/happy)
- [Claude Code UI (Siteboon)](https://github.com/siteboon/claudecodeui)
- [CodeRemote](https://coderemote.dev/)
- [Claude Code on Web](https://www.anthropic.com/news/claude-code-on-the-web)

### Documentation & Guides
- [Claude Code Agentrooms](https://claudecode.run/)
- [Claude Code Frameworks Guide 2025](https://www.medianeth.dev/blog/claude-code-frameworks-subagents-2025)
- [Anthropic Multi-Agent Research System](https://www.anthropic.com/engineering/multi-agent-research-system)
- [Claude Ticket Routing Guide](https://docs.claude.com/en/docs/about-claude/use-case-guides/ticket-routing)
- [Building Agents with Claude Agent SDK](https://www.anthropic.com/engineering/building-agents-with-the-claude-agent-sdk)
