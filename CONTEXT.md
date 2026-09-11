# Agent Workspace & Multitasking System

Glossary of canonical domain concepts for the remote VPS connector, dual-layer tab multiplexer, and autonomous agent execution environment.

## Language

### Tab Architecture

**UnifiedTab**:
An abstracted background process handle representing an asynchronous command prompt session running either locally on the developer's laptop or remotely on the VPS.
_Avoid_: Job, thread, daemon process

**LocalTab**:
A detached command prompt process executed in the developer's Linux/WSL environment, redirecting standard I/O into local disk task logs.
_Avoid_: Local terminal, subshell, worker process

**VpsTab**:
A detached background process executed inside the remote VPS workspace container, tracked by the VPS executor and queryable via the HTTP connector.
_Avoid_: Remote job, cloud worker, SSH session

### Multitasking & State

**PassiveSentinel**:
A file-based status mechanism (`.exit` file) recording the exit code of an asynchronous tab upon process termination, allowing agents to check completion non-blockingly at their own pace.
_Avoid_: Poller, webhook, push notification, interrupt signal

**AutoHybridExecution**:
An execution policy where lightweight commands (formatting, linting, single unit tests) run locally while resource-intensive commands (Rust release builds, full integration suites) are routed to the VPS based on a keyword rulebook.
_Avoid_: Cloud offloading, distributed compute, remote build farm

**KeywordRulebook**:
A deterministic pattern-matching table classifying commands into local execution (fast feedback) or VPS offloading (heavy compute).
_Avoid_: Dynamic heuristic, probabilistic routing

**RollingLimitRetention**:
A disk hygiene rule keeping a maximum of 50 most recent tab execution records, auto-pruning older exited tasks to prevent disk bloat.
_Avoid_: Infinite logging, raw log dump

### Communication & Channels

**InBandHttpChannel**:
The primary production transport (Port 3210, `/mcp`) used by the agent harness for all commands, file transfers, and multitasking tabs via Streamable HTTP/HTTPS without SSH.
_Avoid_: SSH tunnel, gateway proxy, direct socket

**OutOfBandBacktest**:
A developer-only SSH terminal connection (`fern@master`) used solely for manual verification, sanity checks, and baseline comparison against agent results.
_Avoid_: Agent SSH, runtime remote shell

**ShellLevelFreedom**:
Unrestricted execution capability granting the agent the power to run arbitrary Linux bash commands, scripts, compilers, and tools on the VPS through the HTTP connector.
_Avoid_: Sandboxed sandbox, restricted command whitelist
