//! AUTO-GENERATED oleh openapi-codegen 0.1.0 — JANGAN SUNTING MANUAL.
//! spec: frankfurter.openapi.json sha256(raw-bytes): e38804965823bcbd7db06471aa6a2b8b19b4dfc5f26f95d14aae8cfad1fb46a6
//! operasi: 5 emitted, 0 rejected (manifest: frankfurter.manifest.json)
#![allow(clippy::vec_init_then_push)] // kode ter-generate: push-per-node = anti stack-overflow (temuan M3)

use kernel::id::NodeKind;
use kernel::node::{NodeDescriptor, SideEffect};
use kernel::params::{ParameterField, ParameterKind, ParameterOption, ParameterSchema};
use serde_json::json;

pub fn registry() -> Vec<NodeDescriptor> {
    let mut v = Vec::with_capacity(5);
    v.push(node_0000());
    v.push(node_0001());
    v.push(node_0002());
    v.push(node_0003());
    v.push(node_0004());
    v
}

fn node_0000() -> NodeDescriptor {
    let mut d = NodeDescriptor::new(
        NodeKind::new("generated.frankfurter.getcurrencies"),
        1,
        "Get available currencies",
    );
    d.hints.side_effect = SideEffect::Idempotent;
    d.description = Some("Returns a list of available currencies with their full names".into());
    d.params = ParameterSchema::empty();
    d
}

fn node_0001() -> NodeDescriptor {
    let mut d = NodeDescriptor::new(
        NodeKind::new("generated.frankfurter.getdate"),
        1,
        "Get rates for a past date",
    );
    d.hints.side_effect = SideEffect::Idempotent;
    d.description = Some("Returns historical rates for the working day closest to the specified date".into());
    d.params = ParameterSchema { fields: vec![
        ParameterField::new("date", "Date", ParameterKind::String).required().expression(),
        ParameterField { options: Some(vec![ParameterOption { name: "AUD".into(), value: json!("AUD"), description: None }, ParameterOption { name: "BGN".into(), value: json!("BGN"), description: None }, ParameterOption { name: "BRL".into(), value: json!("BRL"), description: None }, ParameterOption { name: "CAD".into(), value: json!("CAD"), description: None }, ParameterOption { name: "CHF".into(), value: json!("CHF"), description: None }, ParameterOption { name: "CNY".into(), value: json!("CNY"), description: None }, ParameterOption { name: "CZK".into(), value: json!("CZK"), description: None }, ParameterOption { name: "DKK".into(), value: json!("DKK"), description: None }, ParameterOption { name: "EUR".into(), value: json!("EUR"), description: None }, ParameterOption { name: "GBP".into(), value: json!("GBP"), description: None }, ParameterOption { name: "HKD".into(), value: json!("HKD"), description: None }, ParameterOption { name: "HUF".into(), value: json!("HUF"), description: None }, ParameterOption { name: "IDR".into(), value: json!("IDR"), description: None }, ParameterOption { name: "ILS".into(), value: json!("ILS"), description: None }, ParameterOption { name: "INR".into(), value: json!("INR"), description: None }, ParameterOption { name: "ISK".into(), value: json!("ISK"), description: None }, ParameterOption { name: "JPY".into(), value: json!("JPY"), description: None }, ParameterOption { name: "KRW".into(), value: json!("KRW"), description: None }, ParameterOption { name: "MXN".into(), value: json!("MXN"), description: None }, ParameterOption { name: "MYR".into(), value: json!("MYR"), description: None }, ParameterOption { name: "NOK".into(), value: json!("NOK"), description: None }, ParameterOption { name: "NZD".into(), value: json!("NZD"), description: None }, ParameterOption { name: "PHP".into(), value: json!("PHP"), description: None }, ParameterOption { name: "PLN".into(), value: json!("PLN"), description: None }, ParameterOption { name: "RON".into(), value: json!("RON"), description: None }, ParameterOption { name: "SEK".into(), value: json!("SEK"), description: None }, ParameterOption { name: "SGD".into(), value: json!("SGD"), description: None }, ParameterOption { name: "THB".into(), value: json!("THB"), description: None }, ParameterOption { name: "TRY".into(), value: json!("TRY"), description: None }, ParameterOption { name: "USD".into(), value: json!("USD"), description: None }, ParameterOption { name: "ZAR".into(), value: json!("ZAR"), description: None }]), ..ParameterField::new("base", "Base", ParameterKind::Options).with_default(json!("EUR")) },
        ParameterField::new("symbols", "Symbols", ParameterKind::Json).expression(),
    ] };
    d
}

fn node_0002() -> NodeDescriptor {
    let mut d = NodeDescriptor::new(
        NodeKind::new("generated.frankfurter.getlatest"),
        1,
        "Get the latest rates",
    );
    d.hints.side_effect = SideEffect::Idempotent;
    d.description = Some("Returns the last working day's rates".into());
    d.params = ParameterSchema { fields: vec![
        ParameterField { options: Some(vec![ParameterOption { name: "AUD".into(), value: json!("AUD"), description: None }, ParameterOption { name: "BGN".into(), value: json!("BGN"), description: None }, ParameterOption { name: "BRL".into(), value: json!("BRL"), description: None }, ParameterOption { name: "CAD".into(), value: json!("CAD"), description: None }, ParameterOption { name: "CHF".into(), value: json!("CHF"), description: None }, ParameterOption { name: "CNY".into(), value: json!("CNY"), description: None }, ParameterOption { name: "CZK".into(), value: json!("CZK"), description: None }, ParameterOption { name: "DKK".into(), value: json!("DKK"), description: None }, ParameterOption { name: "EUR".into(), value: json!("EUR"), description: None }, ParameterOption { name: "GBP".into(), value: json!("GBP"), description: None }, ParameterOption { name: "HKD".into(), value: json!("HKD"), description: None }, ParameterOption { name: "HUF".into(), value: json!("HUF"), description: None }, ParameterOption { name: "IDR".into(), value: json!("IDR"), description: None }, ParameterOption { name: "ILS".into(), value: json!("ILS"), description: None }, ParameterOption { name: "INR".into(), value: json!("INR"), description: None }, ParameterOption { name: "ISK".into(), value: json!("ISK"), description: None }, ParameterOption { name: "JPY".into(), value: json!("JPY"), description: None }, ParameterOption { name: "KRW".into(), value: json!("KRW"), description: None }, ParameterOption { name: "MXN".into(), value: json!("MXN"), description: None }, ParameterOption { name: "MYR".into(), value: json!("MYR"), description: None }, ParameterOption { name: "NOK".into(), value: json!("NOK"), description: None }, ParameterOption { name: "NZD".into(), value: json!("NZD"), description: None }, ParameterOption { name: "PHP".into(), value: json!("PHP"), description: None }, ParameterOption { name: "PLN".into(), value: json!("PLN"), description: None }, ParameterOption { name: "RON".into(), value: json!("RON"), description: None }, ParameterOption { name: "SEK".into(), value: json!("SEK"), description: None }, ParameterOption { name: "SGD".into(), value: json!("SGD"), description: None }, ParameterOption { name: "THB".into(), value: json!("THB"), description: None }, ParameterOption { name: "TRY".into(), value: json!("TRY"), description: None }, ParameterOption { name: "USD".into(), value: json!("USD"), description: None }, ParameterOption { name: "ZAR".into(), value: json!("ZAR"), description: None }]), ..ParameterField::new("base", "Base", ParameterKind::Options).with_default(json!("EUR")) },
        ParameterField::new("symbols", "Symbols", ParameterKind::Json).expression(),
    ] };
    d
}

fn node_0003() -> NodeDescriptor {
    let mut d = NodeDescriptor::new(
        NodeKind::new("generated.frankfurter.getstart_date-.."),
        1,
        "Get rates for a time period",
    );
    d.hints.side_effect = SideEffect::Idempotent;
    d.description = Some("Returns historical rates for every day within a time period starting from the provided date until today.".into());
    d.params = ParameterSchema { fields: vec![
        ParameterField::new("start_date", "Start Date", ParameterKind::String).required().expression(),
        ParameterField { options: Some(vec![ParameterOption { name: "AUD".into(), value: json!("AUD"), description: None }, ParameterOption { name: "BGN".into(), value: json!("BGN"), description: None }, ParameterOption { name: "BRL".into(), value: json!("BRL"), description: None }, ParameterOption { name: "CAD".into(), value: json!("CAD"), description: None }, ParameterOption { name: "CHF".into(), value: json!("CHF"), description: None }, ParameterOption { name: "CNY".into(), value: json!("CNY"), description: None }, ParameterOption { name: "CZK".into(), value: json!("CZK"), description: None }, ParameterOption { name: "DKK".into(), value: json!("DKK"), description: None }, ParameterOption { name: "EUR".into(), value: json!("EUR"), description: None }, ParameterOption { name: "GBP".into(), value: json!("GBP"), description: None }, ParameterOption { name: "HKD".into(), value: json!("HKD"), description: None }, ParameterOption { name: "HUF".into(), value: json!("HUF"), description: None }, ParameterOption { name: "IDR".into(), value: json!("IDR"), description: None }, ParameterOption { name: "ILS".into(), value: json!("ILS"), description: None }, ParameterOption { name: "INR".into(), value: json!("INR"), description: None }, ParameterOption { name: "ISK".into(), value: json!("ISK"), description: None }, ParameterOption { name: "JPY".into(), value: json!("JPY"), description: None }, ParameterOption { name: "KRW".into(), value: json!("KRW"), description: None }, ParameterOption { name: "MXN".into(), value: json!("MXN"), description: None }, ParameterOption { name: "MYR".into(), value: json!("MYR"), description: None }, ParameterOption { name: "NOK".into(), value: json!("NOK"), description: None }, ParameterOption { name: "NZD".into(), value: json!("NZD"), description: None }, ParameterOption { name: "PHP".into(), value: json!("PHP"), description: None }, ParameterOption { name: "PLN".into(), value: json!("PLN"), description: None }, ParameterOption { name: "RON".into(), value: json!("RON"), description: None }, ParameterOption { name: "SEK".into(), value: json!("SEK"), description: None }, ParameterOption { name: "SGD".into(), value: json!("SGD"), description: None }, ParameterOption { name: "THB".into(), value: json!("THB"), description: None }, ParameterOption { name: "TRY".into(), value: json!("TRY"), description: None }, ParameterOption { name: "USD".into(), value: json!("USD"), description: None }, ParameterOption { name: "ZAR".into(), value: json!("ZAR"), description: None }]), ..ParameterField::new("base", "Base", ParameterKind::Options).with_default(json!("EUR")) },
        ParameterField::new("symbols", "Symbols", ParameterKind::Json).expression(),
    ] };
    d
}

fn node_0004() -> NodeDescriptor {
    let mut d = NodeDescriptor::new(
        NodeKind::new("generated.frankfurter.getstart_date-..-end_date"),
        1,
        "Get rates for a time period",
    );
    d.hints.side_effect = SideEffect::Idempotent;
    d.description = Some("Returns historical rates for every day within a time period. The end date defaults to today if not provided.".into());
    d.params = ParameterSchema { fields: vec![
        ParameterField::new("start_date", "Start Date", ParameterKind::String).required().expression(),
        ParameterField::new("end_date", "End Date", ParameterKind::String).required().expression(),
        ParameterField { options: Some(vec![ParameterOption { name: "AUD".into(), value: json!("AUD"), description: None }, ParameterOption { name: "BGN".into(), value: json!("BGN"), description: None }, ParameterOption { name: "BRL".into(), value: json!("BRL"), description: None }, ParameterOption { name: "CAD".into(), value: json!("CAD"), description: None }, ParameterOption { name: "CHF".into(), value: json!("CHF"), description: None }, ParameterOption { name: "CNY".into(), value: json!("CNY"), description: None }, ParameterOption { name: "CZK".into(), value: json!("CZK"), description: None }, ParameterOption { name: "DKK".into(), value: json!("DKK"), description: None }, ParameterOption { name: "EUR".into(), value: json!("EUR"), description: None }, ParameterOption { name: "GBP".into(), value: json!("GBP"), description: None }, ParameterOption { name: "HKD".into(), value: json!("HKD"), description: None }, ParameterOption { name: "HUF".into(), value: json!("HUF"), description: None }, ParameterOption { name: "IDR".into(), value: json!("IDR"), description: None }, ParameterOption { name: "ILS".into(), value: json!("ILS"), description: None }, ParameterOption { name: "INR".into(), value: json!("INR"), description: None }, ParameterOption { name: "ISK".into(), value: json!("ISK"), description: None }, ParameterOption { name: "JPY".into(), value: json!("JPY"), description: None }, ParameterOption { name: "KRW".into(), value: json!("KRW"), description: None }, ParameterOption { name: "MXN".into(), value: json!("MXN"), description: None }, ParameterOption { name: "MYR".into(), value: json!("MYR"), description: None }, ParameterOption { name: "NOK".into(), value: json!("NOK"), description: None }, ParameterOption { name: "NZD".into(), value: json!("NZD"), description: None }, ParameterOption { name: "PHP".into(), value: json!("PHP"), description: None }, ParameterOption { name: "PLN".into(), value: json!("PLN"), description: None }, ParameterOption { name: "RON".into(), value: json!("RON"), description: None }, ParameterOption { name: "SEK".into(), value: json!("SEK"), description: None }, ParameterOption { name: "SGD".into(), value: json!("SGD"), description: None }, ParameterOption { name: "THB".into(), value: json!("THB"), description: None }, ParameterOption { name: "TRY".into(), value: json!("TRY"), description: None }, ParameterOption { name: "USD".into(), value: json!("USD"), description: None }, ParameterOption { name: "ZAR".into(), value: json!("ZAR"), description: None }]), ..ParameterField::new("base", "Base", ParameterKind::Options).with_default(json!("EUR")) },
        ParameterField::new("symbols", "Symbols", ParameterKind::Json).expression(),
    ] };
    d
}
