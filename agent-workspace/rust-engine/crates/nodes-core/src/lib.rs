// nodes-core: Set node (TB-01 literal core + TB-02 dual-form).
// Upstream mirror: SetV2 manual path at n8n 2.39.0 — SetV2.node.ts execute,
// manual.mode.ts execute (<3.3 legacy branch), helpers/utils.ts composeReturnItem,
// prepareReturnItem, validateEntry. Ticket TB-02 rules (binding, over upstream
// where they differ): accept BOTH params.fields.values[] (legacy) and
// params.assignments.assignments[] (canonical) by SHAPE (upstream gates by
// typeVersion; the ticket commands dual-form); include none+all; duplicateItem
// false proceeds, true is explicitly rejected; unknown top-level params are a
// fail-closed error naming the parameter (never silently ignored).
// Pattern: node instances carry configured parameters (engine constructs one per
// execution from workflow JSON). Params ride on &self; no kernel change.
use async_trait::async_trait;
use kernel::context::NodeContext;
use kernel::error::{ErrorCode, NodeError};
use kernel::id::NodeKind;
use kernel::item::{Item, ItemList, PairedItem};
use kernel::node::{Node, NodeDescriptor, NodeOutput};
use kernel::params::{ParameterField, ParameterKind, ParameterSchema};
use serde_json::Value;

pub struct SetNode {
    descriptor: NodeDescriptor,
    params: Value,
}

impl SetNode {
    pub fn with_params(params: Value) -> Self {
        let mut descriptor =
            NodeDescriptor::new(NodeKind::new("n8n-nodes-base.set"), 2, "Set");
        descriptor.params = ParameterSchema::empty()
            .field(ParameterField::new("mode", "Mode", ParameterKind::Options).required())
            .field(ParameterField::new(
                "fields",
                "Fields",
                ParameterKind::Json,
            ))
            .field(ParameterField::new(
                "assignments",
                "Assignments",
                ParameterKind::Json,
            ))
            .field(ParameterField::new(
                "include",
                "Include",
                ParameterKind::Options,
            ));
        Self {
            descriptor,
            params,
        }
    }
}

// Top-level params known on the Set v2 manual path. Anything else is rejected
// (SET_UNKNOWN_PARAM) — real n8n exports only ever carry known keys here, and an
// unknown key means a newer schema we must not pretend to understand.
const KNOWN_PARAMS: &[&str] = &[
    "mode",
    "assignments",
    "fields",
    "include",
    "includeOtherFields",
    "includeFields",
    "excludeFields",
    "duplicateItem",
    "duplicateCount",
    "options",
    "jsonOutput",
    "credentials",
];

#[async_trait]
impl Node for SetNode {
    fn descriptor(&self) -> &NodeDescriptor {
        &self.descriptor
    }

    async fn execute(&self, ctx: &mut NodeContext<'_>) -> Result<NodeOutput, NodeError> {
        if let Some(obj) = self.params.as_object() {
            for key in obj.keys() {
                if !KNOWN_PARAMS.contains(&key.as_str()) {
                    return Err(coded(
                        "SET_UNKNOWN_PARAM",
                        &format!("Set got unknown parameter {key:?}; refusing to guess"),
                    ));
                }
            }
        }

        let mode = self
            .params
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or("");
        if mode != "manual" {
            return Err(coded(
                "SET_MODE_UNSUPPORTED",
                &format!("Set supports mode=manual only, got {mode:?}"),
            ));
        }

        // duplicateItem (SetV2.node.ts:312,361): upstream only duplicates when the
        // workflow execution mode is manual (UI run) — an execution-mode mapping
        // this engine does not model yet. false = proceed; true = explicit reject.
        let duplicate = self.params.get("duplicateItem").unwrap_or(&Value::Null);
        match duplicate {
            Value::Null => {}
            Value::Bool(false) => {}
            Value::Bool(true) => {
                return Err(coded(
                    "SET_DUPLICATE_UNSUPPORTED",
                    "duplicateItem=true needs execution-mode mapping \
                     (upstream duplicates only on UI-manual runs); TB-02 supports false",
                ));
            }
            other => {
                return Err(bad(&format!(
                    "duplicateItem must be boolean, got {}",
                    describe(other)
                )));
            }
        }
        // duplicateCount is only read inside the duplicate branch upstream; with
        // duplicateItem=false it is validated-but-inert, exactly like upstream.
        if let Some(dc) = self.params.get("duplicateCount") {
            if dc.as_u64().is_none() {
                return Err(bad("duplicateCount must be a non-negative integer"));
            }
        }

        // include (composeReturnItem): none = fresh object, all = deep copy of
        // input. Default 'all' (SetV2.node.ts:343). selected/except need field
        // lists + dot-notation helpers — explicitly rejected (ticket: min none+all).
        let include = self
            .params
            .get("include")
            .and_then(Value::as_str)
            .unwrap_or("all");
        match include {
            "none" | "all" => {}
            other => {
                return Err(coded(
                    "SET_INCLUDE_UNSUPPORTED",
                    &format!("include={other:?} needs field-list support; TB-02 supports none+all"),
                ));
            }
        }
        // Read-pattern mirror: upstream only reads these inside branches we do not
        // take on the v2 path, so they are validated-but-inert here.
        if let Some(v) = self.params.get("includeOtherFields") {
            if !v.is_boolean() {
                return Err(bad("includeOtherFields must be boolean"));
            }
        }
        for key in ["includeFields", "excludeFields"] {
            if let Some(v) = self.params.get(key) {
                if !v.is_string() {
                    return Err(bad(&format!("{key} must be a comma-separated string")));
                }
            }
        }

        // options collection (dotNotation, includeBinary, ...) is deferred: empty
        // (or absent) proceeds, anything set is rejected by name. Dotted-name
        // nesting is likewise deferred (see rejection below), so a flat write of
        // a dotted name can never silently diverge from dot-notation semantics.
        if let Some(opts) = self.params.get("options") {
            match opts.as_object() {
                Some(map) if map.is_empty() => {}
                Some(map) => {
                    let mut keys: Vec<&str> =
                        map.keys().map(String::as_str).collect();
                    keys.sort_unstable();
                    return Err(coded(
                        "SET_OPTIONS_UNSUPPORTED",
                        &format!("Set options not supported in TB-02: {keys:?}"),
                    ));
                }
                None => return Err(bad("options must be an object")),
            }
        }
        // Set uses no credentials; empty/absent is export noise, anything set is
        // rejected rather than silently trusted.
        match self.params.get("credentials") {
            None | Some(Value::Null) => {}
            Some(Value::Object(map)) if map.is_empty() => {}
            Some(other) => {
                return Err(coded(
                    "SET_CREDENTIALS_REJECTED",
                    &format!(
                        "Set takes no credentials, got {}",
                        describe(other)
                    ),
                ));
            }
        }
        // jsonOutput belongs to raw mode; under manual it is unread upstream.
        if let Some(v) = self.params.get("jsonOutput") {
            if !v.is_string() {
                return Err(bad("jsonOutput must be a string"));
            }
        }

        // Dual-form selection (ticket rule, by shape): the non-empty form wins;
        // both non-empty is ambiguous and fails closed; both empty = zero
        // assignments, still a valid Set (include filtering applies).
        let canonical = self
            .params
            .get("assignments")
            .map(|a| {
                a.get("assignments")
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        bad("assignments must be an object with an assignments array")
                    })
            })
            .transpose()?;
        let legacy = self
            .params
            .get("fields")
            .map(|f| {
                f.get("values")
                    .and_then(Value::as_array)
                    .ok_or_else(|| bad("fields must be an object with a values array"))
            })
            .transpose()?;
        let no_entries: Vec<Value> = Vec::new();
        let (entries, is_canonical): (&Vec<Value>, bool) = match (canonical, legacy) {
            (Some(c), Some(l)) if !c.is_empty() && !l.is_empty() => {
                return Err(coded(
                    "SET_AMBIGUOUS_FORM",
                    "both fields.values and assignments.assignments are non-empty; refusing to pick",
                ));
            }
            (Some(c), _) if !c.is_empty() => (c, true),
            (_, Some(l)) if !l.is_empty() => (l, false),
            (Some(c), _) => (c, true),
            (_, Some(l)) => (l, false),
            (None, None) => (&no_entries, true),
        };
        let mut assignments: Vec<(String, Value)> = Vec::with_capacity(entries.len());
        for e in entries {
            let name = e
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| bad("assignment without name"))?;
            if name.contains('.') {
                return Err(coded(
                    "SET_DOTTED_NAME_UNSUPPORTED",
                    &format!(
                        "assignment name {name:?} needs dot-notation nesting (deferred); refusing flat write"
                    ),
                ));
            }
            let (kind, raw) = if is_canonical {
                let kind = e
                    .get("type")
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad(&format!("assignment {name:?} without type")))?;
                let raw = e.get("value").cloned().unwrap_or(Value::Null);
                (kind.to_string(), raw)
            } else {
                let kind = e
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("stringValue");
                let slot = e.get(kind).cloned().unwrap_or(Value::Null);
                if slot.is_null() && e.get(kind).is_none() {
                    return Err(bad(&format!(
                        "assignment {name:?} declares type {kind:?} but carries no {kind} value"
                    )));
                }
                (
                    kind.strip_suffix("Value").unwrap_or(kind).to_string(),
                    slot,
                )
            };
            // Fail-closed expressions (TB-03 territory): any literal string starting
            // with '=' is refused, never evaluated or passed through.
            if let Value::String(s) = &raw {
                if s.starts_with('=') {
                    return Err(NodeError::Expression {
                        expr: s.clone(),
                        reason: "Set accepts literals only; expressions are TB-03"
                            .to_string(),
                    });
                }
            }
            let value = validate_entry(name, &kind, raw)?;
            assignments.push((name.to_string(), value));
        }

        let input = ctx
            .input(0)
            .map_err(|e| NodeError::Internal {
                message: format!("input 0: {e}"),
            })?;
        let items = input.materialize_all().await.map_err(|e| NodeError::Internal {
            message: format!("materialize: {e}"),
        })?;
        let mut out = Vec::with_capacity(items.len());
        for (i, item) in items.iter().enumerate() {
            // composeReturnItem: fresh base for none, deep copy for all; then set
            // each assignment. Binary is NOT carried: v2 default strips it upstream
            // (composeReturnItem: v<3.4 copies binary only with includeBinary, and
            // non-empty options are rejected above).
            let mut obj = if include == "all" {
                item.json
                    .as_object()
                    .cloned()
                    .ok_or_else(|| bad("Set input item json must be an object"))?
            } else {
                serde_json::Map::new()
            };
            for (k, v) in &assignments {
                obj.insert(k.clone(), v.clone());
            }
            out.push(Item {
                json: Value::Object(obj),
                binary: None,
                paired_item: Some(PairedItem {
                    item_id: i as u32,
                    input_node_index: None,
                }),
            });
        }
        Ok(NodeOutput::single(ItemList::Inline(out)))
    }
}

// Minimal mirror of upstream validateEntry (utils.ts) for the literal subset.
// Upstream's full coercion table lives in n8n-workflow (not in this checkout),
// so anything beyond the cases below fails closed instead of guessing.
fn validate_entry(name: &str, kind: &str, value: Value) -> Result<Value, NodeError> {
    if value.is_null() {
        return Ok(Value::Null);
    }
    match kind {
        "string" => {
            let s = match &value {
                Value::String(s) => s.clone(),
                Value::Array(_) | Value::Object(_) => value.to_string(),
                Value::Number(_) | Value::Bool(_) => value.to_string(),
                Value::Null => return Ok(Value::Null),
            };
            Ok(Value::String(s))
        }
        "number" => match value {
            Value::Number(_) => Ok(value),
            other => Err(bad(&format!(
                "'{name}' expects a number but got {}",
                describe(&other)
            ))),
        },
        "boolean" => match value {
            Value::Bool(_) => Ok(value),
            other => Err(bad(&format!(
                "'{name}' expects a boolean but got {}",
                describe(&other)
            ))),
        },
        "array" => match value {
            Value::Array(_) => Ok(value),
            Value::String(s) => match serde_json::from_str::<Value>(&s) {
                Ok(v @ Value::Array(_)) => Ok(v),
                _ => Err(bad(&format!("'{name}' expects an array but got {s:?}"))),
            },
            other => Err(bad(&format!(
                "'{name}' expects an array but got {}",
                describe(&other)
            ))),
        },
        "object" => match value {
            Value::Object(_) => Ok(value),
            Value::String(s) => match serde_json::from_str::<Value>(&s) {
                Ok(v @ Value::Object(_)) => Ok(v),
                _ => Err(bad(&format!("'{name}' expects an object but got {s:?}"))),
            },
            other => Err(bad(&format!(
                "'{name}' expects an object but got {}",
                describe(&other)
            ))),
        },
        other => Err(coded(
            "SET_TYPE_UNSUPPORTED",
            &format!("assignment {name:?} has unsupported type {other:?}"),
        )),
    }
}

fn describe(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn coded(code: &str, message: &str) -> NodeError {
    NodeError::Permanent {
        message: message.to_string(),
        code: ErrorCode(code.to_string()),
    }
}

fn bad(message: &str) -> NodeError {
    coded("SET_BAD_PARAMS", message)
}
