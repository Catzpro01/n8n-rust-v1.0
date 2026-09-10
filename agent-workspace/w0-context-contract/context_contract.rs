//! W0-CONTEXT-CONTRACT — CT-01..CT-08 (plan: `docs/W0-CONTEXT-CONTRACT-TEST.md`)
//!
//! Tanpa tokio (aturan freeze kernel): mini-executor `block_on` mengikuti pola
//! `tests/contract.rs`. Mock = executable spec untuk implementasi sungguhan.
//! Uji yang butuh varian error yang belum ada di kanonik: `#[ignore]` + alasan.

use async_trait::async_trait;
use kernel::context::*;
use kernel::error::KernelError;
use kernel::id::{ContentId, NodeId, SpillPath};
use kernel::item::{Item, ItemList, SpillCodec, SpilledList};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Mini executor (pola contract.rs) — kernel tak boleh bergantung tokio
// ---------------------------------------------------------------------------
fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    use std::sync::Arc as StdArc;
    use std::task::{Context, Poll, Wake, Waker};
    struct Noop;
    impl Wake for Noop {
        fn wake(self: StdArc<Self>) {}
    }
    let waker = Waker::from(StdArc::new(Noop));
    let mut cx = Context::from_waker(&waker);
    let mut fut = std::pin::pin!(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

// ---------------------------------------------------------------------------
// CT-01 PriorOutputs
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
struct MockPrior {
    names: HashMap<String, PriorNodeRef>,
    ids: HashMap<NodeId, PriorNodeRef>,
    items_for: HashMap<(String, u8), ItemList>,
    available: Vec<String>,
}

impl MockPrior {
    fn add(&mut self, name: &str, id: u64, branch_lengths: Vec<usize>) {
        let r = PriorNodeRef { id: NodeId::new(id.to_string()), name: name.to_string(), branch_lengths };
        self.names.insert(name.to_string(), r.clone());
        self.ids.insert(NodeId::new(id.to_string()), r.clone());
    }
}

#[async_trait]
impl PriorOutputs for MockPrior {
    fn by_name(&self, name: &str) -> Option<PriorNodeRef> {
        self.names.get(name).cloned()
    }
    fn by_id(&self, id: &NodeId) -> Option<PriorNodeRef> {
        self.ids.get(id).cloned()
    }
    async fn item(&self, node: &str, input_index: u8, i: usize) -> Result<Item, KernelError> {
        if let Some(r) = self.names.get(node) {
            let len = r.branch_lengths.get(input_index as usize).copied().unwrap_or(0);
            if i >= len {
                return Err(KernelError::IndexOutOfBounds { index: i, len });
            }
            return Ok(Item::new(json!({"node": node, "i": i})));
        }
        Err(KernelError::NodeNotFound { node: node.to_string() })
    }
    fn items(&self, node: &str, input_index: u8) -> Result<ItemList, KernelError> {
        self.items_for
            .get(&(node.to_string(), input_index))
            .cloned()
            .ok_or_else(|| KernelError::NodeNotFound { node: node.to_string() })
    }
    fn available_nodes(&self) -> Vec<String> {
        self.available.clone()
    }
}

#[test]
fn c_t01a_name_is_case_sensitive() {
    let mut p = MockPrior::default();
    p.add("HTTP Request", 1, vec![2]);
    assert!(p.by_name("HTTP Request").is_some(), "exact name must resolve");
    assert!(p.by_name("http request").is_none(), "n8n display names are case-sensitive");
}

#[test]
fn c_t01b_by_name_none_vs_item_err() {
    let mut p = MockPrior::default();
    p.add("N", 1, vec![2]);
    assert!(p.by_name("MISSING").is_none(), "by_name unknown -> None, bukan Err");
    let r = block_on(p.item("MISSING", 0, 0));
    assert!(matches!(r, Err(KernelError::NodeNotFound { .. })), "item unknown -> Err(NodeNotFound)");
}

#[test]
fn c_t01c_by_id_and_by_name_same_ref() {
    let mut p = MockPrior::default();
    p.add("Same", 7, vec![3]);
    let by_name = p.by_name("Same").unwrap();
    let by_id = p.by_id(&NodeId::new(7u64.to_string())).unwrap();
    assert_eq!(by_name.id, by_id.id);
    assert_eq!(by_name.name, by_id.name);
    assert_eq!(by_name.branch_lengths, by_id.branch_lengths);
}

#[test]
fn c_t01d_out_of_bounds_index_is_err_not_panic() {
    let mut p = MockPrior::default();
    p.add("N", 1, vec![2]);
    let r = block_on(p.item("N", 0, 5));
    assert!(matches!(r, Err(KernelError::IndexOutOfBounds { index: 5, len: 2 })),
        "index overflow -> Err, bukan panic");
}

#[test]
fn c_t01e_items_can_be_spilled_without_materialization() {
    let mut p = MockPrior::default();
    let spilled = ItemList::Spilled(SpilledList {
        path: SpillPath::new("/tmp/spill-ctx-test.bin"),
        len: 100_000,
        total_bytes: 4_000_000,
        codec: SpillCodec::Json,
    });
    p.items_for.insert(("big".to_string(), 0), spilled.clone());
    let got = p.items("big", 0).unwrap();
    assert!(matches!(got, ItemList::Spilled(_)), "items() harus mampu mengembalikan Spilled");
    assert_eq!(got.len(), 100_000, "len() O(1) kedua varian — tidak menyentuh store");
    assert!(got.ram_footprint() < 256,
        "Spilled ram_footprint = metadata-only (~64B+path), dapat {}", got.ram_footprint());
    assert_eq!(got.ram_footprint(), spilled.ram_footprint());
}

#[test]
fn c_t01f_available_nodes_only_finished() {
    let mut p = MockPrior::default();
    p.add("finished-1", 1, vec![1]);
    p.available = vec!["finished-1".to_string(), "finished-2".to_string()];
    let avail = p.available_nodes();
    assert!(!avail.contains(&"running-node".to_string()), "node berjalan tidak boleh terdaftar");
    assert!(avail.contains(&"finished-1".to_string()));
}

// ---------------------------------------------------------------------------
// CT-02 ExpressionEngine
// ---------------------------------------------------------------------------

#[derive(Default)]
struct MockExpr {
    validate_calls: Mutex<Vec<String>>,
    eval_calls: Mutex<Vec<String>>,
}

#[async_trait]
impl ExpressionEngine for MockExpr {
    async fn validate(&self, expr: &str) -> Result<(), KernelError> {
        self.validate_calls.lock().unwrap().push(expr.to_string());
        Ok(()) // CT-02a: TIDAK mengeksekusi; CT-02b: sintaks saja, bukan evaluasi
    }
    async fn eval(&self, expr: &str, _scope: &ExpressionScope<'_>) -> Result<Value, KernelError> {
        self.eval_calls.lock().unwrap().push(expr.to_string());
        if expr.is_empty() {
            return Err(KernelError::Expression { expr: expr.to_string(), reason: "empty".into() });
        }
        Ok(json!({ "expr": expr }))
    }
    async fn eval_batch(&self, exprs: &[&str], scope: &ExpressionScope<'_>) -> Result<Vec<Value>, KernelError> {
        let mut out = Vec::with_capacity(exprs.len());
        for e in exprs {
            out.push(self.eval(e, scope).await?);
        }
        Ok(out)
    }
}

fn dummy_exec() -> ExecutionMeta {
    ExecutionMeta {
        id: kernel::id::ExecutionId::new(1),
        workflow_id: kernel::id::WorkflowId::new(1),
        workflow_version: 1,
        mode: ExecutionMode::Test,
        timezone: "UTC".into(),
        execution_order: ExecutionOrder::V1,
        started_at: 0,
    }
}
fn dummy_wf() -> WorkflowMeta {
    WorkflowMeta { id: kernel::id::WorkflowId::new(1), name: "wf".into(), version: 1, active: false }
}
struct DummyEnv;
impl EnvAccess for DummyEnv {
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
}
fn mk_scope<'a>(
    json: &'a Value,
    input: &'a ItemList,
    prior: &'a dyn PriorOutputs,
    exec: &'a ExecutionMeta,
    wf: &'a WorkflowMeta,
    vars: &'a Value,
    env: &'a dyn EnvAccess,
) -> ExpressionScope<'a> {
    ExpressionScope {
        json,
        input,
        binary: None,
        prior,
        execution: exec,
        workflow: wf,
        now_unix_ms: 0,
        vars,
        env,
        item_index: 0,
    }
}

#[test]
fn c_t02a_validate_does_not_execute() {
    let e = MockExpr::default();
    block_on(e.validate("{{ $json.foo.bar }}")).unwrap();
    assert!(e.eval_calls.lock().unwrap().is_empty(), "validate TIDAK boleh memicu eval");
    assert_eq!(e.validate_calls.lock().unwrap().len(), 1);
}

#[test]
fn c_t02b_validate_syntax_only_not_eval() {
    let e = MockExpr::default();
    assert!(block_on(e.validate("{{ $json.tidakAda.foo }}")).is_ok(), "sintaks valid -> Ok walau eval runtime akan gagal");
}

#[test]
fn c_t02c_eval_batch_equals_individual_within_scope() {
    let json = json!({});
    let input = ItemList::empty();
    let prior = MockPrior::default();
    let exec = dummy_exec();
    let wf = dummy_wf();
    let vars = json!({});
    let env = DummyEnv;
    let scope = mk_scope(&json, &input, &prior, &exec, &wf, &vars, &env);
    let e = MockExpr::default();
    let exprs = vec!["{{ 1 + 1 }}", "{{ $json.x }}"];
    let batch = block_on(e.eval_batch(&exprs, &scope)).unwrap();
    let mut indiv = Vec::new();
    for x in &exprs {
        indiv.push(block_on(e.eval(x, &scope)).unwrap());
    }
    assert_eq!(batch, indiv, "eval_batch == eval per ekspresi pada scope sama");
}

#[test]
fn c_t02d_eval_batch_empty_is_ok() {
    let json = json!({});
    let input = ItemList::empty();
    let prior = MockPrior::default();
    let exec = dummy_exec();
    let wf = dummy_wf();
    let vars = json!({});
    let env = DummyEnv;
    let scope = mk_scope(&json, &input, &prior, &exec, &wf, &vars, &env);
    let e = MockExpr::default();
    assert_eq!(block_on(e.eval_batch(&[], &scope)).unwrap(), Vec::<Value>::new());
}

#[test]
fn c_t02e_error_carries_original_expr_text() {
    let json = json!({});
    let input = ItemList::empty();
    let prior = MockPrior::default();
    let exec = dummy_exec();
    let wf = dummy_wf();
    let vars = json!({});
    let env = DummyEnv;
    let scope = mk_scope(&json, &input, &prior, &exec, &wf, &vars, &env);
    let e = MockExpr::default();
    let err = block_on(e.eval("", &scope)).unwrap_err();
    match err {
        KernelError::Expression { expr, .. } => assert_eq!(expr, "", "expr harus memuat teks asli"),
        other => panic!("harus Expression, dapat {other:?}"),
    }
}

#[test]
fn c_t02f_scope_shape_compiles_with_all_documented_fields() {
    let _ = |s: &ExpressionScope<'_>| {
        let _: &Value = s.json;
        let _: &ItemList = s.input;
        let _: Option<&Value> = s.binary;
        let _: &dyn PriorOutputs = s.prior;
        let _: &ExecutionMeta = s.execution;
        let _: &WorkflowMeta = s.workflow;
        let _: i64 = s.now_unix_ms;
        let _: &Value = s.vars;
        let _: &dyn EnvAccess = s.env;
        let _: usize = s.item_index;
    };
    fn assert_no_enumerate<E: EnvAccess>(env: &E) {
        let _ = env.get("X");
    }
    assert_no_enumerate(&DummyEnv);
}

// ---------------------------------------------------------------------------
// CT-03 EnvAccess whitelist
// ---------------------------------------------------------------------------

struct WhitelistEnv(HashMap<String, String>);
impl EnvAccess for WhitelistEnv {
    fn get(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
}

#[test]
fn c_t03a_whitelist_denies_secrets() {
    let mut m = HashMap::new();
    m.insert("N8N_ALLOWED".to_string(), "1".to_string());
    let env = WhitelistEnv(m);
    assert_eq!(env.get("N8N_ALLOWED"), Some("1".to_string()));
    assert_eq!(env.get("AWS_SECRET_ACCESS_KEY"), None, "secret env wajib None");
    assert_eq!(env.get("PATH"), None, "selain whitelist -> None");
    assert_eq!(env.get("AWS_SECRET_ACCESS_KEY"), None);
}

/// CT-03b: bentuk API — hanya `get`. Stable Rust tidak punya negative-impl
/// test; uji ini mengunci bahwa SEMUA penggunaan env lewat `get` dan menjadi
/// titik review bila seseorang menambah `keys()`/`iter()` (doc :345 MUST).
#[test]
fn c_t03b_api_shape_env_access_only_get() {
    let env = WhitelistEnv(HashMap::new());
    let _: Option<String> = env.get("anything");
}

// ---------------------------------------------------------------------------
// CT-04 CredentialProvider (least privilege D92)
// ---------------------------------------------------------------------------

#[derive(Default)]
struct MockCreds {
    declared: HashSet<String>,
    values: HashMap<String, Value>,
    requested: Mutex<Vec<String>>,
}

#[async_trait]
impl CredentialProvider for MockCreds {
    async fn get(&self, kind: &str) -> Result<CredentialValue, KernelError> {
        self.requested.lock().unwrap().push(kind.to_string());
        if !self.declared.contains(kind) {
            return Err(KernelError::Invalid { message: format!("credential kind not declared: {kind}") });
        }
        Ok(CredentialValue::new(self.values.get(kind).cloned().unwrap_or(Value::Null)))
    }
}

#[test]
fn c_t04a_undeclared_kind_is_err() {
    let mut c = MockCreds::default();
    c.declared.insert("api".to_string());
    let r = block_on(c.get("not-declared"));
    assert!(r.is_err(), "node meminta kind tidak dideklarasikan -> Err (bukan Ok kosong)");
}

#[test]
fn c_t04b_declared_kind_ok() {
    let mut c = MockCreds::default();
    c.declared.insert("api".to_string());
    c.values.insert("api".to_string(), json!({"token": "v"}));
    let v = block_on(c.get("api")).unwrap();
    assert_eq!(v.get("token"), Some(&json!("v")));
}

#[test]
fn c_t04c_exact_set_of_requested_kinds() {
    let mut c = MockCreds::default();
    for k in ["a", "b", "c"] {
        c.declared.insert(k.to_string());
        c.values.insert(k.to_string(), json!({}));
    }
    let _ = block_on(c.get("a")).unwrap();
    let _ = block_on(c.get("c")).unwrap();
    // D92: akses TIDAK BOLEH lebih dari yang dideklarasikan/diminta.
    let got: HashSet<String> = c.requested.lock().unwrap().iter().cloned().collect();
    assert!(got.contains("a") && got.contains("c"));
    let extra: HashSet<String> = got.difference(&c.declared).cloned().collect();
    assert!(extra.is_empty(), "akses kredensial di luar deklarasi: {extra:?}");
}

/// CT-04d (keputusan §3.1 = opsi (a)): `CredentialValue::as_value()` DIHAPUS;
/// `Logger::log` menerima `&LogFields` — bukan `&Value`. Alarm perubahan API:
/// bila jalur lama dikembalikan, pemanggilan di bawah gagal kompilasi.
#[test]
fn c_t04d_logger_accepts_logfields_not_raw_value() {
    let l = CapturingLogger::default();
    let cred = CredentialValue::new(json!({"apiKey": "sk-SECRET-abc123"}));
    let mut fields = LogFields::new();
    fields.put_credential("cred", &cred);
    fields.put("kind", &json!("resolved"));
    l.log(LogLevel::Info, "resolved credential", &fields);
    let entries = l.entries.lock().unwrap();
    let text = serde_json::to_string(&entries[0].2).unwrap();
    assert!(!text.contains("sk-SECRET-abc123"), "LogFields tidak boleh memuat nilai mentah kredensial");
    assert!(text.contains("redacted"), "put_credential wajib menulis penanda redaksi: {text}");
}

// ---------------------------------------------------------------------------
// CT-05 HttpClient
// ---------------------------------------------------------------------------

#[derive(Default)]
struct MockHttp {
    seen_headers: Mutex<Vec<Vec<String>>>,
    seen_query: Mutex<Vec<Vec<String>>>,
}

#[async_trait]
impl HttpClient for MockHttp {
    async fn send(&self, req: HttpRequest) -> Result<HttpResponse, KernelError> {
        self.seen_headers.lock().unwrap().push(req.headers.keys().cloned().collect());
        self.seen_query.lock().unwrap().push(req.query.keys().cloned().collect());
        if let Some(RequestBody::Blob { .. }) = &req.body {
            // CT-05d: pengirim request tidak pernah meng-load blob ke RAM.
        }
        Ok(HttpResponse {
            status: 200,
            headers: BTreeMap::new(),
            body: ResponseBody::Empty,
            duration_ms: 0,
        })
    }
}

fn http_req() -> HttpRequest {
    HttpRequest {
        method: "GET".into(),
        url: "https://x".into(),
        headers: BTreeMap::new(),
        query: BTreeMap::new(),
        body: None,
        max_response_bytes: 1024,
        timeout_ms: 30_000,
    }
}

#[test]
fn c_t05e_headers_query_deterministic_order() {
    let h = MockHttp::default();
    let mut req = http_req();
    req.headers.insert("zz".into(), "1".into());
    req.headers.insert("aa".into(), "2".into());
    req.headers.insert("mm".into(), "3".into());
    req.query.insert("q2".into(), "b".into());
    req.query.insert("q1".into(), "a".into());
    let _ = block_on(h.send(req));
    let seen = h.seen_headers.lock().unwrap();
    assert_eq!(seen[0], vec!["aa", "mm", "zz"], "headers BTreeMap -> urutan tetap (differential L3)");
    let seen_q = h.seen_query.lock().unwrap();
    assert_eq!(seen_q[0], vec!["q1", "q2"]);
}

#[test]
fn c_t05d_blob_request_never_materializes() {
    let store = Arc::new(MockBlobStore::default());
    let (cid, len) = block_on(store.put(&vec![0u8; 8_000_000], "application/octet-stream")).unwrap();
    assert_eq!(len, 8_000_000);
    let mut req = http_req();
    req.body = Some(RequestBody::Blob { content_id: cid });
    let h = MockHttp::default();
    let _ = block_on(h.send(req));
    assert_eq!(store.get_calls.load(Ordering::SeqCst), 0,
        "pengirim request TIDAK boleh memanggil BlobStore::get (kontrak never fully in RAM)");
}

/// CT-05a/CT-05b — DITAHAN: plafon memori butuh `KernelError::ResourceExhausted`
/// dan `Timeout { elapsed_ms }` yang belum ada di kanonik (a7d0357 error.rs).
/// Rekomendasi gatekeeper: tambah varian tsb (agent1, kernel owner) lalu
/// aktifkan. `ResponseBody::TooLarge` sudah ada sebagai penanda arah yang benar.
#[ignore = "menunggu varian KernelError::ResourceExhausted/Timeout (agent1/matt)"]
#[test]
fn c_t05a_oversized_response_is_resource_error() {}

// ---------------------------------------------------------------------------
// CT-06 BlobStore
// ---------------------------------------------------------------------------

#[derive(Default)]
struct MockBlobStore {
    map: Mutex<HashMap<u64, (Vec<u8>, String)>>,
    next: AtomicU64,
    get_calls: AtomicU64,
}

#[async_trait]
impl BlobStore for MockBlobStore {
    async fn put(&self, bytes: &[u8], mime: &str) -> Result<(ContentId, u64), KernelError> {
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        self.map.lock().unwrap().insert(id, (bytes.to_vec(), mime.to_string()));
        Ok((ContentId::new(id), bytes.len() as u64))
    }
    async fn get(&self, id: ContentId) -> Result<Vec<u8>, KernelError> {
        self.get_calls.fetch_add(1, Ordering::SeqCst);
        self.map.lock().unwrap().get(&id.get()).map(|(b, _)| b.clone())
            .ok_or_else(|| KernelError::Invalid { message: format!("blob not found: {}", id.get()) })
    }
    async fn delete(&self, id: ContentId) -> Result<(), KernelError> {
        self.map.lock().unwrap().remove(&id.get());
        Ok(())
    }
    fn size(&self, id: ContentId) -> Option<u64> {
        self.map.lock().unwrap().get(&id.get()).map(|(b, _)| b.len() as u64)
    }
}

#[test]
fn c_t06a_roundtrip_byte_identical() {
    let s = MockBlobStore::default();
    let data: Vec<u8> = (0..=255u8).collect();
    let (id, len) = block_on(s.put(&data, "application/x-test")).unwrap();
    assert_eq!(len, 256);
    assert_eq!(block_on(s.get(id)).unwrap(), data, "byte-per-byte");
}

#[test]
fn c_t06b_put_size_consistent() {
    let s = MockBlobStore::default();
    let (id, len) = block_on(s.put(b"hello", "text/plain")).unwrap();
    assert_eq!(len, 5);
    assert_eq!(s.size(id), Some(5));
}

#[test]
fn c_t06c_unknown_id_no_panic() {
    let s = MockBlobStore::default();
    assert!(block_on(s.get(ContentId::new(999))).is_err());
    assert_eq!(s.size(ContentId::new(999)), None);
}

#[test]
fn c_t06d_delete_then_get_err() {
    let s = MockBlobStore::default();
    let (id, _) = block_on(s.put(b"x", "a")).unwrap();
    block_on(s.delete(id)).unwrap();
    assert!(block_on(s.get(id)).is_err());
    assert_eq!(s.size(id), None);
}

#[test]
fn c_t06e_delete_idempotent() {
    let s = MockBlobStore::default();
    let (id, _) = block_on(s.put(b"x", "a")).unwrap();
    assert!(block_on(s.delete(id)).is_ok());
    assert!(block_on(s.delete(id)).is_ok(), "delete kedua tidak boleh Err");
    assert!(block_on(s.delete(ContentId::new(123456))).is_ok());
}

#[test]
fn c_t06f_content_id_policy_documented() {
    // K-8 belum diputus. Mock NON-CASD (id per-put) — perilaku DIRECAM, bukan
    // diputus. Bila W2-CASD-DEDUP masuk -> byte identik -> id sama; uji dibalik.
    let s = MockBlobStore::default();
    let (id1, _) = block_on(s.put(b"same-bytes", "a")).unwrap();
    let (id2, _) = block_on(s.put(b"same-bytes", "a")).unwrap();
    assert_ne!(id1, id2, "mock non-CASD (K-8 belum diputus) — keputusan = W2-CASD-DEDUP");
}

// ---------------------------------------------------------------------------
// CT-07 CancellationToken
// ---------------------------------------------------------------------------

struct TestCancel { cancelled: bool, reason: Option<String> }
impl CancellationToken for TestCancel {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn reason(&self) -> Option<&str> { self.reason.as_deref() }
}

struct CancelAfterN { remaining: std::sync::atomic::AtomicUsize, reason: String }
impl CancellationToken for CancelAfterN {
    fn is_cancelled(&self) -> bool {
        self.remaining.fetch_sub(1, Ordering::SeqCst) == 0
    }
    fn reason(&self) -> Option<&str> { Some(&self.reason) }
}

#[test]
fn c_t07a_no_cancel_never_cancelled() {
    assert!(!NoCancel.is_cancelled());
}
#[test]
fn c_t07b_no_cancel_default_reason_none() {
    assert_eq!(NoCancel.reason(), None, "default method reason() harus None");
}
#[test]
fn c_t07c_cancelled_token_reports_reason() {
    let t = TestCancel { cancelled: true, reason: Some("timeout".into()) };
    assert!(t.is_cancelled());
    assert_eq!(t.reason(), Some("timeout"));
}
#[test]
fn c_t07d_cancellation_observed_mid_iteration() {
    let token = CancelAfterN { remaining: std::sync::atomic::AtomicUsize::new(3), reason: "watchdog".into() };
    let mut iterations = 0usize;
    loop {
        if token.is_cancelled() {
            assert_eq!(token.reason(), Some("watchdog"));
            break;
        }
        iterations += 1;
        assert!(iterations < 1000, "loop harus berhenti saat token dibatalkan (A-11 polite half)");
    }
    assert!(iterations <= 3, "berhenti dalam batas wajar (poll ke-{iterations})");
}

// ---------------------------------------------------------------------------
// CT-08 Logger (keputusan §3.1 = opsi (a): LogFields)
// ---------------------------------------------------------------------------

#[derive(Default)]
struct CapturingLogger {
    entries: Mutex<Vec<(LogLevel, String, Value)>>,
}
impl Logger for CapturingLogger {
    fn log(&self, level: LogLevel, message: &str, fields: &LogFields) {
        self.entries.lock().unwrap().push((level, message.to_string(), fields.to_value()));
    }
}

#[test]
fn c_t08a_noop_logger_never_panics() {
    let l = NoopLogger;
    let f = LogFields::new();
    for lv in [LogLevel::Trace, LogLevel::Debug, LogLevel::Info, LogLevel::Warn, LogLevel::Error] {
        l.log(lv, "msg", &f);
    }
}

#[test]
fn c_t08b_log_level_serde_lowercase_wire_format() {
    let cases = [("trace", LogLevel::Trace), ("debug", LogLevel::Debug), ("info", LogLevel::Info),
                 ("warn", LogLevel::Warn), ("error", LogLevel::Error)];
    for (s, lv) in cases {
        let ser = serde_json::to_string(&lv).unwrap();
        assert_eq!(ser, format!("\"{s}\""), "rename_all=\"lowercase\" = kontrak wire-format");
        let de: LogLevel = serde_json::from_str(&ser).unwrap();
        assert_eq!(de, lv);
    }
    assert!(serde_json::from_str::<LogLevel>("\"Info\"").is_err(), "pola lama (Capitalized) harus gagal");
}

#[test]
fn c_t08c_credential_redaction_by_construction() {
    let l = CapturingLogger::default();
    let cred = CredentialValue::new(json!({"user": "alice", "apiKey": "sk-live-9876543210"}));
    let mut fields = LogFields::new();
    fields.put("trace_id", &json!("abc-123"));
    fields.put_credential("github", &cred);
    l.log(LogLevel::Warn, "credential resolved", &fields);
    let entries = l.entries.lock().unwrap();
    let val = &entries[0].2;
    let text = serde_json::to_string(val).unwrap();
    assert!(!text.contains("sk-live-9876543210"), "kredensial TIDAK boleh plaintext: {text}");
    assert!(!text.contains("alice"), "field kredensial apa pun TIDAK boleh bocor: {text}");
    assert_eq!(val["trace_id"], json!("abc-123"), "data publik tetap tercatat");
    assert_eq!(val["github"]["redacted"], json!(true));
}

/// Alarm §3.1: `CredentialValue::as_value()` telah DIHAPUS (digantikan
/// LogFields). Bila dikembalikan, review wajib menolak (doc :394-395).
#[test]
fn c_t08d_credential_value_has_no_raw_log_escape() {
    let cred = CredentialValue::new(json!({"k": "v"}));
    let _ = cred.get("k"); // trusted integration (D92) — bukan jalur log
    let mut f = LogFields::new();
    f.put_credential("secret", &cred);
    assert!(f.to_value()["secret"].is_object());
}
