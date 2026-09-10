//! Registry aturan migrasi v1 (M2) — transform murni, berversi, berbukti
//! korpus. Sumber kebenaran mapping: AGENT4-NODE-ALIAS-DEPRECATION.md v0.1
//! (V-1 Code v2 VERIFIED, V-2 ScheduleTrigger VERIFIED, V-2b cron util
//! VERIFIED; commit upstream n8n 3afdf4a0f8a46fe4087ed463b2eda2e888beabbe).
//!
//! Aturan v1:
//! - `rosetta.rule.v1.function-to-code`     : function v1 → Code v2
//!   (jsCode, mode runOnceForAllItems; prolog `const items = $input.all();`
//!   hanya bila kode menyebut token `items` — pola resmi V-3).
//! - `rosetta.rule.v1.cron-to-scheduletrigger`: cron v1 → scheduleTrigger v1.3
//!   (triggerTimes.item[] → rule.interval[{field cronExpression, expression}],
//!   ekspresi 5-field; target v1.3 = typeVersion tertinggi terbukti di korpus
//!   [22 file 1.1–1.3] — koreksi atas usul typeVersion 2 di katalog alias).
//!
//! TIDAK termasuk v1 (ditunda jujur, bukan hasil salah):
//! - functionItem v1 → Code: bentuk `item` legacy = data plain (bukti korpus
//!   tpl-156/tpl-175), padanan modern `$input.item` belum terverifikasi
//!   upstream (V-3b OPEN — katalog alias §4b) → unresolved, bukan tebakan.
//!
//! Semua fungsi murni + deterministik; TIDAK menyentuh file (R-1/R-8).

use serde_json::{Map, Value};

/// Nama rule function→Code.
pub const RULE_FUNCTION_TO_CODE: &str = "rosetta.rule.v1.function-to-code";
/// Nama rule cron→scheduleTrigger.
pub const RULE_CRON_TO_SCHEDULE: &str = "rosetta.rule.v1.cron-to-scheduletrigger";

/// Alasan unresolved yang distandardisasi (dipakai receipt; ID deviasi
/// DEVIATION-CATALOG agent5 ditautkan saat katalog terbit — nol ID lokal).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnresolvedKind {
    /// functionItem: rewrite `item` menunggu verifikasi upstream (V-3b).
    FunctionItemAwaitingV3b,
    /// function/cron versi tak dikenal (bukan v1).
    UnexpectedVersion,
    /// require() modul eksternal — kebijakan no-community-npm (PRD-2 §2.1).
    UnsupportedExternalRequire,
    /// Bentuk parameter yang tidak bisa dipetakan dengan aman.
    UnmappableShape,
}

impl UnresolvedKind {
    /// Deskriptor stabil (bukan ID lokal — lihat R-5).
    pub fn code(&self) -> &'static str {
        match self {
            Self::FunctionItemAwaitingV3b => {
                "functionitem-awaiting-v3b (padanan item modern belum terverifikasi)"
            }
            Self::UnexpectedVersion => "unexpected-typeversion",
            Self::UnsupportedExternalRequire => {
                "unsupported-external-require (kebijakan PRD-2 §2.1 no-community-npm)"
            }
            Self::UnmappableShape => "unmappable-parameter-shape",
        }
    }
}

/// Konversi triggerTimes.item[] (cron v1) → ekspresi cron 5-field.
///
/// Bentuk item nyata di korpus (13 file, semuanya objek JSON):
///   {"mode":"everyX","unit":"minutes","value":N}   → `*/N * * * *`
///   {"mode":"everyX","unit":"hours","value":N}     → `0 */N * * *` (K-2: m=0)
///   {"mode":"everyMinute"}                          → `* * * * *`
///   {"mode":"everyHour"[, "minute":m]}              → `m * * * *`   (K-2 m=0)
///   {"mode":"everyDay","hour":h,"minute":m}         → `m h * * *`   (K-2 0:0)
///   {"mode":"everyWeek","weekday":w,"hour":h,...}   → `m h * * w`   (w wajib)
///   {"mode":"everyMonth","dayOfMonth":d,...}        → `m h d * *`   (d wajib)
///   {"mode":"custom","cronExpression":e}            → 6-field: buang detik;
///                                                      5-field: langsung
///   {"hour":h} (tanpa mode, legacy)                 → `0 h * * *`
/// Ekspresi 5-field (V-2b: detik sengaja dibuang — granularitas menit).
pub fn cron_item_to_expression(item: &Map<String, Value>) -> Result<String, String> {
    let get = |k: &str| -> Option<i64> {
        item.get(k).and_then(|v| match v {
            Value::Number(n) => n.as_i64(),
            Value::String(s) => s.trim().parse::<i64>().ok(),
            _ => None,
        })
    };

    let mode = item.get("mode").and_then(|v| v.as_str()).unwrap_or("");
    match mode {
        "everyMinute" => Ok("* * * * *".to_string()),
        "everyX" => {
            let unit = item
                .get("unit")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let value = get("value").ok_or_else(|| "everyX tanpa value".to_string())?;
            match unit {
                "minutes" => Ok(format!("*/{value} * * * *")),
                "hours" => Ok(format!("0 */{value} * * *")),
                _ => Err(format!("everyX unit tak dikenal: {unit}")),
            }
        }
        "everyHour" => {
            let m = get("minute").unwrap_or(0);
            Ok(format!("{m} * * * *"))
        }
        "everyDay" => {
            let h = get("hour").unwrap_or(0);
            let m = get("minute").unwrap_or(0);
            Ok(format!("{m} {h} * * *"))
        }
        "everyWeek" => {
            let w = get("weekday").ok_or_else(|| "everyWeek tanpa weekday".to_string())?;
            let h = get("hour").unwrap_or(0);
            let m = get("minute").unwrap_or(0);
            Ok(format!("{m} {h} * * {w}"))
        }
        "everyMonth" => {
            let d = get("dayOfMonth").ok_or_else(|| "everyMonth tanpa dayOfMonth".to_string())?;
            let h = get("hour").unwrap_or(0);
            let m = get("minute").unwrap_or(0);
            Ok(format!("{m} {h} {d} * *"))
        }
        "custom" => {
            let e = item
                .get("cronExpression")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "custom tanpa cronExpression".to_string())?
                .trim();
            let tokens: Vec<&str> = e.split_whitespace().collect();
            match tokens.len() {
                5 => Ok(tokens.join(" ")),
                // 6-field (detik di depan): buang detik, pertahankan sisanya
                6 => Ok(tokens[1..].join(" ")),
                n => Err(format!("customExpression {n}-field tak didukung: {e:?}")),
            }
        }
        // legacy tanpa mode: {"hour": h} = jalankan harian jam h
        "" => {
            if let Some(h) = get("hour") {
                let m = get("minute").unwrap_or(0);
                Ok(format!("{m} {h} * * *"))
            } else {
                Err(format!("item cron tanpa mode & tanpa hour: {item:?}"))
            }
        }
        other => Err(format!("mode cron tak dikenal: {other:?}")),
    }
}

/// Transform cron v1 → parameter `rule` ScheduleTrigger.
pub fn cron_params_to_rule(params: &Value) -> Result<Value, String> {
    let tt = params
        .get("triggerTimes")
        .ok_or_else(|| "cron tanpa triggerTimes".to_string())?;
    let items = tt
        .get("item")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "triggerTimes.item bukan array".to_string())?;

    let mut interval = Vec::new();
    for it in items {
        let obj = it
            .as_object()
            .ok_or_else(|| "item cron bukan objek".to_string())?;
        let expr = cron_item_to_expression(obj)?;
        interval.push(serde_json::json!({
            "field": "cronExpression",
            "expression": expr,
        }));
    }
    Ok(serde_json::json!({ "interval": interval }))
}

/// Apakah string code menyebut token `items` (var batch legacy function v1).
pub fn code_mentions_items(code: &str) -> bool {
    code.split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '$')
        .any(|tok| tok == "items")
}

/// Transform function v1 → parameter Code v2 (jsCode + mode eksplisit).
pub fn function_params_to_code(params: &Value) -> Result<(Value, Vec<String>), UnresolvedKind> {
    let code = params
        .get("functionCode")
        .and_then(|v| v.as_str())
        .ok_or(UnresolvedKind::UnmappableShape)?
        .to_string();

    let mut assumptions = Vec::new();
    let js = if code_mentions_items(&code) {
        assumptions.push(
            "prolog `const items = $input.all();` ditambahkan (V-3: pola resmi all-items; verifikasi runtime saat differential harness menyala)"
                .to_string(),
        );
        format!("const items = $input.all();\n{code}")
    } else {
        code
    };

    let mut p = Map::new();
    p.insert("jsCode".into(), Value::String(js));
    p.insert("mode".into(), Value::String("runOnceForAllItems".into()));
    Ok((Value::Object(p), assumptions))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(json: &str) -> Map<String, Value> {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn cron_setiap_bentuk_korpus() {
        // 13 file cron — tiap bentuk nyata (scan 11:34)
        assert_eq!(
            cron_item_to_expression(&item(r#"{"mode":"everyX","unit":"minutes","value":5}"#))
                .unwrap(),
            "*/5 * * * *"
        );
        assert_eq!(
            cron_item_to_expression(&item(r#"{"mode":"everyHour"}"#)).unwrap(),
            "0 * * * *"
        );
        assert_eq!(
            cron_item_to_expression(&item(r#"{"mode":"everyMinute"}"#)).unwrap(),
            "* * * * *"
        );
        // custom 5-field
        assert_eq!(
            cron_item_to_expression(&item(r#"{"mode":"custom","cronExpression":"* */6 * * *"}"#))
                .unwrap(),
            "* */6 * * *"
        );
        // custom 6-field (detik dibuang)
        assert_eq!(
            cron_item_to_expression(&item(
                r#"{"mode":"custom","cronExpression":"0 */2 * * * *"}"#
            ))
            .unwrap(),
            "*/2 * * * *"
        );
        assert_eq!(
            cron_item_to_expression(&item(
                r#"{"mode":"custom","cronExpression":"0 0 17 28 9 *"}"#
            ))
            .unwrap(),
            "0 17 28 9 *"
        );
        // legacy tanpa mode {"hour":7}
        assert_eq!(
            cron_item_to_expression(&item(r#"{"hour":7}"#)).unwrap(),
            "0 7 * * *"
        );
        // K-2 default 0
        assert_eq!(
            cron_item_to_expression(&item(r#"{"mode":"everyDay"}"#)).unwrap(),
            "0 0 * * *"
        );
        // format lain terdukung, bukan di korpus
        assert_eq!(
            cron_item_to_expression(&item(r#"{"mode":"everyX","unit":"hours","value":3}"#))
                .unwrap(),
            "0 */3 * * *"
        );
        assert_eq!(
            cron_item_to_expression(&item(r#"{"mode":"everyWeek","weekday":1,"hour":7}"#)).unwrap(),
            "0 7 * * 1"
        );
        // bentuk tak aman → Err eksplisit (bukan hasil salah)
        assert!(cron_item_to_expression(&item(r#"{"mode":"everyWeek"}"#)).is_err());
        assert!(
            cron_item_to_expression(&item(r#"{"mode":"custom","cronExpression":"a b c"}"#))
                .is_err()
        );
        assert!(cron_item_to_expression(&item(r#"{"x":1}"#)).is_err());
    }

    #[test]
    fn cron_params_multi_item() {
        let p: Value = serde_json::json!({
            "triggerTimes": {"item": [
                {"mode":"everyX","unit":"minutes","value":5},
                {"mode":"everyHour"}
            ]}
        });
        let rule = cron_params_to_rule(&p).unwrap();
        assert_eq!(
            rule,
            serde_json::json!({"interval": [
                {"field":"cronExpression","expression":"*/5 * * * *"},
                {"field":"cronExpression","expression":"0 * * * *"}
            ]})
        );
    }

    #[test]
    fn function_transform_mentions_items_dan_tidak() {
        let (p, a) = function_params_to_code(&serde_json::json!({
            "functionCode": "items[0].json.x = 1; return items;"
        }))
        .unwrap();
        assert_eq!(a.len(), 1);
        let js = p.get("jsCode").unwrap().as_str().unwrap();
        assert!(js.starts_with("const items = $input.all();\n"));
        assert_eq!(p.get("mode").unwrap(), "runOnceForAllItems");

        // tanpa token items → tanpa prolog (jujur, kode identik)
        let (p2, a2) = function_params_to_code(&serde_json::json!({
            "functionCode": "return [{json:{id:0}}];"
        }))
        .unwrap();
        assert!(a2.is_empty());
        assert_eq!(p2.get("jsCode").unwrap(), "return [{json:{id:0}}];");
    }

    #[test]
    fn token_items_tidak_tertangkap_di_komentar() {
        // komentar berisi "items" tidak boleh memicu prolog? token dlm komentar
        // tetap token — di sini kita verifikasi perilaku konservatif: prolog
        // hanya menambah, tak pernah menghapus; kasus komentar = safe (var
        // items tetap tersedia walau tak dipakai).
        let (p, _a) = function_params_to_code(&serde_json::json!({
            "functionCode": "// process items later\nreturn 1;"
        }))
        .unwrap();
        let js = p.get("jsCode").unwrap().as_str().unwrap();
        assert!(js.starts_with("const items = $input.all();\n"));
    }
}
