# n8n-upgraded-rust

Rust implementation of n8n workflow automation engine with enhanced determinism, auditability, and replay capabilities.

## Language

### Core Domain (Canonical from kernel - DO NOT rename)

**Item**:
A single JSON value flowing through the workflow, wrapped in metadata for lineage tracking.
_Avoid_: Record, document, message

**ItemList**:
A collection of items, either inline (in-memory) or spilled (on-disk for large datasets).
Variants: `ItemList::Inline`, `ItemList::Spilled`
_Avoid_: Array, list, batch

**SpilledList**:
An ItemList that has been written to disk via SpillStore, identified by ContentId.
_Avoid_: Offloaded list, persisted list

**ContentId**:
Deterministic identifier for spilled content, enabling replay and deduplication.
_Avoid_: Hash, digest, fingerprint

**SpillStore**:
Trait for writing/reading items to/from disk when they exceed memory thresholds.
_Avoid_: Offload store, persist store, cache

**Checkpoint**:
A saved execution state that can be restored for replay or recovery.
_Avoid_: Snapshot, save point

**SideEffect**:
Classification of node behavior (idempotent vs non-idempotent) for retry semantics.
_Avoid_: Mutation, impact

**ParameterSchema**:
JSON Schema defining valid parameters for a node type.
_Avoid_: Config schema, options schema

**PriorOutputs**:
Reference to outputs from previous nodes, available via `{{ $node.name }}` expressions.
_Avoid_: Previous results, upstream data

**NodeDescriptor**:
Metadata describing a node type (name, version, parameters, outputs).
_Avoid_: Node spec, node definition

### Top-Level Concepts

**Workflow**:
A JSON document defining a directed acyclic graph of nodes connected by edges, executed from trigger to completion.
_Avoid_: Pipeline, flow, process

**Node**:
A single processing unit within a workflow, identified by name and type, consuming input items and producing output items.
_Avoid_: Step, task, action

**Execution**:
A single run of a workflow, identified by ExecutionId, producing a deterministic sequence of node outputs.
_Avoid_: Run, job, invocation

**Trigger**:
A special node type that initiates workflow execution (e.g., manualTrigger, scheduleTrigger, webhookTrigger).
_Avoid_: Starter, initiator

**Connection**:
A directed edge between two nodes, defining data flow from source node output to destination node input.
_Avoid_: Link, edge, wire

**Lineage**:
Audit trail tracking which items produced which other items through node transformations.
_Avoid_: Provenance, history, trace

**Attempt**:
A single execution of a node within an execution, tracked separately for replay and error recovery.
_Avoid_: Try, run, iteration

## Architecture

**Kernel traits** (kernel-asli crate):
Core abstractions defining contracts all implementations must satisfy.
Canonical terms: Item, ItemList, SpilledList, ContentId, SpillStore, Checkpoint, SideEffect, ParameterSchema, PriorOutputs, NodeDescriptor

**Executor** (executor crate):
Minimal engine walking workflow graph, invoking nodes through kernel Node trait, returning final items.

**Storage** (storage crate):
SQLite-based persistence with WAL (spill_intent) for crash safety and lineage tracking for audit.

**Rosetta** (rosetta crate):
Compiler normalizing n8n workflow JSON to canonical form for deterministic digest computation.

## Boundaries

**Workflow model** owns: JSON parsing, validation, publish/unpublish lifecycle.
**Executor** owns: Graph traversal, node invocation, item flow.
**Storage** owns: Persistence, crash recovery, audit trail.
**Kernel** owns: Contracts only, no implementation.

## Sources

- PRD-FRONTEND-RUST.md (sha 997cd83271ff) — canonical domain terms from kernel
- Codebase exploration (executor, storage, rosetta crates)
