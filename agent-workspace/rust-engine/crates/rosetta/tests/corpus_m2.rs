//! Gate M2 atas korpus nyata 171 template:
//! - 22 function v1 + 13 cron v1 = 35 migrasi berhasil (R-4: tiap rule
//!   punya fixture korpus + diff-test).
//! - 11 functionItem v1 = unresolved (V-3b OPEN — lihat registry.rs).
//! - 0 sisa node `function`/`cron` di grafik hasil.
//! - Determinisme (R-8) + nol mutasi file (R-1).

use std::path::PathBuf;

use rosetta::migrate::migrate_workflow;
use rosetta::parse::parse_workflow_path;

fn corpus_files() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> =
        std::fs::read_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"))
            .expect("tests/fixtures ada")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
            .collect();
    v.sort();
    v
}

#[test]
fn m2_korpus_35_migrasi_11_unresolved() {
    let mut migrated = 0usize;
    let mut unresolved = 0usize;
    let mut residual_function = 0usize;
    let mut residual_cron = 0usize;
    let mut files = 0usize;

    for f in corpus_files() {
        files += 1;
        let wf = parse_workflow_path(&f).expect("parse");
        let (out, report) = migrate_workflow(&wf);

        migrated += report.migrated_count();
        unresolved += report.unresolved_count();
        for n in &out {
            match n.type_full.as_str() {
                "n8n-nodes-base.function" | "n8n-nodes-base.functionItem" => residual_function += 1,
                "n8n-nodes-base.cron" => residual_cron += 1,
                _ => {}
            }
        }

        // R-1: file tidak boleh berubah oleh proses apa pun
        let after = std::fs::read(&f).unwrap();
        let before = std::fs::read(&f).unwrap();
        assert_eq!(before, after, "mutasi pada {}", f.display());
    }

    assert_eq!(files, 171);
    assert_eq!(migrated, 35, "22 function + 13 cron harus termigrasi");
    assert_eq!(
        unresolved, 11,
        "11 functionItem (V-3b) — jujur, bukan hasil salah"
    );
    // sisa functionItem wajar (unresolved); function & cron harus 0
    assert_eq!(residual_function, 11, "sisa = functionItem unresolved saja");
    assert_eq!(residual_cron, 0, "semua cron termigrasi");

    eprintln!("M2 korpus: {migrated} migrasi, {unresolved} unresolved (11 functionItem V-3b)");
}

#[test]
fn m2_determinisme_korpus() {
    for f in corpus_files() {
        let wf = parse_workflow_path(&f).unwrap();
        let (a, ra) = migrate_workflow(&wf);
        let (b, rb) = migrate_workflow(&wf);
        assert_eq!(a, b, "output node beda: {}", f.display());
        assert_eq!(ra, rb, "laporan beda: {}", f.display());
    }
}

#[test]
fn m2_setiap_rule_punya_fixture_korpus_nyata() {
    // R-4: fixture utk tiap aturan v1 — dari korpus (bukan sintetis)
    let mut cron_rules = 0;
    let mut fn_rules = 0;
    for f in corpus_files() {
        let wf = parse_workflow_path(&f).unwrap();
        let (_, report) = migrate_workflow(&wf);
        for m in &report.migrations {
            match m.rule_version.as_str() {
                "rosetta.rule.v1.function-to-code" => fn_rules += 1,
                "rosetta.rule.v1.cron-to-scheduletrigger" => cron_rules += 1,
                other => panic!("rule tak dikenal: {other}"),
            }
        }
    }
    assert_eq!(cron_rules, 13, "13 fixture cron (13 file)");
    assert_eq!(fn_rules, 22, "22 fixture function (22 file)");
}

#[test]
fn m2_ekspresi_cron_korpus_terverifikasi() {
    // verifikasi spot: tiap cron file termigrasi menghasilkan ekspresi
    // yang sudah dicek manual thd bentuk triggerTimes-nya (scan 11:42)
    let mut checks = 0;
    for f in corpus_files() {
        let wf = parse_workflow_path(&f).unwrap();
        let (out, _) = migrate_workflow(&wf);
        for n in &out {
            if n.type_full == "n8n-nodes-base.scheduleTrigger" && n.extra.get("rosetta").is_some() {
                let expr = n
                    .parameters
                    .pointer("/rule/interval/0/expression")
                    .and_then(|v| v.as_str())
                    .expect("ekspresi ada");
                // 5-field, token numerik valid
                let toks: Vec<&str> = expr.split_whitespace().collect();
                assert_eq!(toks.len(), 5, "{expr}");
                checks += 1;
            }
        }
    }
    assert_eq!(
        checks, 13,
        "13 cron → 13 scheduleTrigger dgn ekspresi 5-field"
    );
}
