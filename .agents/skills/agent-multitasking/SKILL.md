---
name: agent-multitasking
description: Autonomous command-prompt tab multiplexer and remote execution protocol. Enables AI agents to spawn background tasks on laptop or VPS, avoid blocking execution turns, and inspect logs on-demand.
---

# Agent Multitasking & Remote VPS Harness

This skill equips AI agents with the mental model and concrete tools to perform true multitasking using **Command Prompt Tabs** (dual-layer: Laptop & VPS).

## Core Law: Zero Blocking Turns

When an operation takes more than 3 seconds (compilation, package installation, test suites, container builds):
1. **NEVER** run it as a synchronous blocking command in your primary execution turn.
2. **SPAWN** a new background tab immediately using `tab_new` (via MCP) or `connector-cli tab new`.
3. **SWITCH** immediately to the next task in your plan (reading specs, editing unrelated files, writing documentation) without idling or sleeping.
4. **INSPECT** the tab output (`tab_attach` or `connector-cli tab attach <name>`) only when you actually need the result of that task.

## Available Native Tools (MCP & CLI)

You can invoke these tools either via MCP (`multitasking-tabs` server) or directly through shell commands:

### 1. Open New Background Tab (`tab_new`)
- **CLI**: `connector-cli tab new [name] <command...>` (Laptop)
- **CLI (VPS)**: `connector-cli tab new --vps [name] <command...>`
- **MCP Tool**: `tab_new({ name: "build-task", command: "cargo build", target: "laptop" | "vps" })`

### 2. Live Tab Dashboard (`tab_list`)
- **CLI**: `connector-cli tabs`
- **MCP Tool**: `tab_list({})`
- Displays all live tabs across Laptop and VPS, showing execution status (`🟢 running` / `⚪ exited`) and exit codes.

### 3. Attach / Peek Log Output (`tab_attach`)
- **CLI**: `connector-cli tab attach <name|id>`
- **MCP Tool**: `tab_attach({ name: "build-task" })`
- Retrieves live stdout and stderr from the background tab without killing or interrupting it.

### 4. Stop a Tab (`tab_kill`)
- **CLI**: `connector-cli tab kill <name|id>`
- **MCP Tool**: `tab_kill({ name: "build-task" })`

### 5. Clean Exited Tabs (`tab_clean`)
- **CLI**: `connector-cli tab clean`
- **MCP Tool**: `tab_clean({})`
- Prunes finished tasks to maintain a clean workspace dashboard.

### 6. Direct VPS Shell (`vps_exec`)
- **CLI**: `connector-cli vps "<command>"`
- **MCP Tool**: `vps_exec({ command: "uname -a" })`
- Direct point-to-point remote execution on VPS via SSH without API friction.
