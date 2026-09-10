#!/usr/bin/env python3
"""
Corpus Validator - agent2 (Backend Engineer)
Memvalidasi apakah workflow JSON benar-benar ekspor n8n asli.

Ekspor n8n asli SELALU membawa field metadata:
  id, versionId, meta (templateId/instanceId), active, 
  createdAt, updatedAt, pinData, settings

Script ini mengecek setiap file JSON dan menentukan:
  - REAL_EXPORT: punya >= 5 dari 8 field metadata
  - LIKELY_EXPORT: punya 3-4 field metadata
  - SYNTHETIC: punya < 3 field metadata (ditulis tangan/digenerate)
  - INVALID: bukan JSON valid atau tidak punya nodes+connections
"""

import json
import os
import sys
import hashlib
from pathlib import Path
from datetime import datetime

CORPUS_DIR = "/opt/agent-workspace/docs/corpus"
MANIFEST_FILE = "/opt/agent-workspace/docs/corpus/VALIDATION-REPORT-agent2.md"

# Field metadata yang SELALU ada di ekspor n8n asli
N8N_EXPORT_FIELDS = [
    "id",           # UUID workflow
    "versionId",    # Version ID (selalu ada di ekspor)
    "meta",         # Meta info (templateId, instanceId)
    "active",       # Boolean: workflow aktif/tidak
    "createdAt",    # Timestamp pembuatan
    "updatedAt",    # Timestamp update terakhir
    "pinData",      # Pin data (bisa empty object)
    "settings",     # Workflow settings
]

# Field yang HARUS ada untuk dianggap workflow valid
REQUIRED_WORKFLOW_FIELDS = ["name", "nodes", "connections"]

def sha256_file(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for chunk in iter(lambda: f.read(8192), b''):
            h.update(chunk)
    return h.hexdigest()

def validate_workflow(filepath):
    """Validate a single workflow JSON file."""
    result = {
        "file": os.path.basename(filepath),
        "path": filepath,
        "sha256": sha256_file(filepath),
        "valid_json": False,
        "has_required_fields": False,
        "export_fields_found": [],
        "export_fields_missing": [],
        "node_count": 0,
        "node_types": [],
        "has_external_nodes": False,
        "classification": "UNKNOWN",
        "issues": [],
    }
    
    # Parse JSON
    try:
        with open(filepath, 'r') as f:
            data = json.load(f)
        result["valid_json"] = True
    except json.JSONDecodeError as e:
        result["classification"] = "INVALID"
        result["issues"].append(f"JSON parse error: {e}")
        return result
    except Exception as e:
        result["classification"] = "INVALID"
        result["issues"].append(f"Read error: {e}")
        return result
    
    # Check required fields
    missing_required = [f for f in REQUIRED_WORKFLOW_FIELDS if f not in data]
    result["has_required_fields"] = len(missing_required) == 0
    if missing_required:
        result["issues"].append(f"Missing required fields: {missing_required}")
    
    # Check export metadata fields
    for field in N8N_EXPORT_FIELDS:
        if field in data:
            result["export_fields_found"].append(field)
        else:
            result["export_fields_missing"].append(field)
    
    # Count nodes
    nodes = data.get("nodes", [])
    result["node_count"] = len(nodes)
    result["node_types"] = list(set(n.get("type", "unknown") for n in nodes))
    
    # Check for external/API-calling nodes
    external_prefixes = [
        "n8n-nodes-base.httpRequest",
        "n8n-nodes-base.openWeatherMap",
        "n8n-nodes-base.openAi",
        "n8n-nodes-base.gmail",
        "n8n-nodes-base.slack",
        "n8n-nodes-base.googleSheets",
        "n8n-nodes-base.telegram",
        "n8n-nodes-base.discord",
        "n8n-nodes-base.github",
        "n8n-nodes-base.postgres",
        "n8n-nodes-base.mySql",
        "n8n-nodes-base.mongoDb",
    ]
    for node in nodes:
        ntype = node.get("type", "")
        for prefix in external_prefixes:
            if ntype.startswith(prefix) or prefix.split(".")[-1] in ntype.lower():
                result["has_external_nodes"] = True
                break
    
    # Also check langchain/AI nodes
    for node in nodes:
        ntype = node.get("type", "")
        if "langchain" in ntype.lower() or "@n8n" in ntype:
            result["has_external_nodes"] = True
    
    # Classification
    n_found = len(result["export_fields_found"])
    if n_found >= 5:
        result["classification"] = "REAL_EXPORT"
    elif n_found >= 3:
        result["classification"] = "LIKELY_EXPORT"
    elif not result["has_required_fields"]:
        result["classification"] = "INVALID"
    else:
        result["classification"] = "SYNTHETIC"
        result["issues"].append(f"Hanya {n_found}/8 field metadata ekspor n8n ditemukan")
    
    if result["has_external_nodes"]:
        result["issues"].append("Mengandung node eksternal (butuh API calls nyata / mock)")
    
    return result

def main():
    corpus_dir = Path(CORPUS_DIR)
    json_files = sorted(corpus_dir.glob("*.json"))
    
    if not json_files:
        print("ERROR: Tidak ada file JSON di corpus/")
        sys.exit(1)
    
    results = []
    for jf in json_files:
        r = validate_workflow(str(jf))
        results.append(r)
    
    # Generate report
    now = datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S UTC")
    
    lines = []
    lines.append(f"# LAPORAN VALIDASI KORPUS — agent2")
    lines.append(f"**Tanggal:** {now}")
    lines.append(f"**Script:** validate_corpus.py")
    lines.append(f"**Total file:** {len(results)}")
    lines.append("")
    
    # Summary
    counts = {}
    for r in results:
        c = r["classification"]
        counts[c] = counts.get(c, 0) + 1
    
    lines.append("## RINGKASAN")
    lines.append("")
    lines.append("| Klasifikasi | Jumlah | Arti |")
    lines.append("|---|---|---|")
    for cls in ["REAL_EXPORT", "LIKELY_EXPORT", "SYNTHETIC", "INVALID"]:
        n = counts.get(cls, 0)
        meaning = {
            "REAL_EXPORT": "✅ Ekspor n8n asli (valid untuk differential testing)",
            "LIKELY_EXPORT": "🟡 Kemungkinan ekspor n8n (perlu verifikasi manual)",
            "SYNTHETIC": "🔴 BUKAN ekspor n8n — ditulis tangan/digenerate",
            "INVALID": "❌ Bukan workflow JSON valid",
        }.get(cls, "?")
        lines.append(f"| {cls} | {n} | {meaning} |")
    lines.append("")
    
    # Detail per file
    lines.append("## DETAIL PER FILE")
    lines.append("")
    lines.append("| File | Status | Nodes | Export Fields | External API | Issues |")
    lines.append("|---|---|---|---|---|---|")
    
    for r in results:
        emoji = {"REAL_EXPORT": "✅", "LIKELY_EXPORT": "🟡", "SYNTHETIC": "🔴", "INVALID": "❌"}.get(r["classification"], "?")
        ext = "YA" if r["has_external_nodes"] else "tidak"
        issues = "; ".join(r["issues"][:2]) if r["issues"] else "-"
        fields = f"{len(r['export_fields_found'])}/8"
        lines.append(f"| {r['file']} | {emoji} {r['classification']} | {r['node_count']} | {fields} | {ext} | {issues} |")
    
    lines.append("")
    
    # Node types found
    all_types = set()
    for r in results:
        all_types.update(r["node_types"])
    lines.append("## SEMUA NODE TYPES DITEMUKAN")
    lines.append("")
    for t in sorted(all_types):
        lines.append(f"- `{t}`")
    lines.append("")
    
    # Recommendations
    lines.append("## REKOMENDASI")
    lines.append("")
    real = counts.get("REAL_EXPORT", 0)
    lines.append(f"1. Hanya **{real}/{len(results)}** file yang benar-benar ekspor n8n asli.")
    lines.append(f"2. Untuk differential testing (PRD-2 §10.2), kita butuh **minimal 50 REAL_EXPORT**.")
    lines.append(f"3. File SYNTHETIC masih berguna untuk test parser (L1 = format), TIDAK untuk L2/L3.")
    lines.append(f"4. File dengan external nodes butuh **mock server** sebelum bisa dijalankan deterministik.")
    lines.append(f"5. **Sumber korpus nyata yang disarankan:**")
    lines.append(f"   - Template publik n8n: https://n8n.io/workflows/ (ekspor manual)")
    lines.append(f"   - Repo komunitas n8n (search: n8n workflow github)")
    lines.append(f"   - Buat workflow sederhana di n8n instance lokal lalu ekspor")
    lines.append("")
    
    report = "\n".join(lines)
    
    # Write report
    with open(MANIFEST_FILE, 'w') as f:
        f.write(report)
    os.chmod(MANIFEST_FILE, 0o644)
    
    # Also print to stdout
    print(report)
    
    return 0 if real > 0 else 1

if __name__ == "__main__":
    sys.exit(main())
