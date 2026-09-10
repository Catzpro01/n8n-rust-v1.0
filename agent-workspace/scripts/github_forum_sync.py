#!/usr/bin/env python3
import sqlite3, json, os, urllib.request, time

TOKEN = "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"
REPO = "Catzpro01/n8n-rust-forum"
STATE_FILE = "/var/lib/agent-comm/github_synced_ids.json"
DB_PATH = "/var/lib/agent-comm/comm.db"

CH_CORE_ENGINE   = 1
CH_SECURITY      = 2
CH_QA_TESTING    = 3
CH_ANNOUNCEMENTS = 4
CH_INNOVATION    = 5
CH_STORAGE       = 6
CH_PARSER        = 7

def determine_target_issue(channel, content, sender):
    c_lower = content.lower()
    
    if any(k in content for k in ["MANDAT PEMILIK PROYEK", "SUPERVISOR", "ATURAN WAJIB", "PENGUMUMAN"]) or channel in ["aturan"]:
        return CH_ANNOUNCEMENTS

    if any(k in c_lower for k in ["sayembara", "proposal", "usulan", "timeline execution", "item lineage", "preflight dry-run", "ebc", "bytecode cache", "rosetta", "determinism contract", "wasm", "envelope"]):
        return CH_INNOVATION

    if any(k in c_lower for k in ["spill", "sqlite wal", "casd", "data-plane", "blake3", "postcard", "deduplication", "pitr"]) or channel in ["database"]:
        return CH_STORAGE

    if any(k in c_lower for k in ["parser", "schema compiler", "stickynote", "opaque node", "alias deprecation", "to_cron", "functionitem"]):
        return CH_PARSER

    if any(k in c_lower for k in ["sec-", "quickjs sandbox", "audit keamanan", "umask", "nopasswd", "cve", "injection"]) or channel in ["security", "audit-keamanan"]:
        return CH_SECURITY

    if any(k in c_lower for k in ["korpus", "differential test", "normalizer", "v8 vs", "exec-diff", "verify_corpus", "edge-case", "harness"]) or channel in ["corpus-qa"]:
        return CH_QA_TESTING

    if channel in ["architecture", "n8n-upgraded-rust"]:
        return CH_CORE_ENGINE
    elif channel in ["general"]:
        return CH_ANNOUNCEMENTS

    return CH_CORE_ENGINE

def post_comment(issue_num, body):
    url = f"https://api.github.com/repos/{REPO}/issues/{issue_num}/comments"
    req = urllib.request.Request(
        url,
        data=json.dumps({"body": body}).encode("utf-8"),
        headers={
            "Authorization": f"Bearer {TOKEN}",
            "Accept": "application/vnd.github.v3+json",
            "Content-Type": "application/json",
            "User-Agent": "fern-agent-sync"
        }
    )
    with urllib.request.urlopen(req, timeout=15) as resp:
        return resp.status == 201

def main():
    synced_ids = set()
    if os.path.exists(STATE_FILE):
        try:
            with open(STATE_FILE, "r") as f:
                synced_ids = set(json.load(f))
        except Exception:
            pass

    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cur = conn.cursor()

    cur.execute("SELECT id, channel, sender, recipient, content, created_at FROM messages ORDER BY id ASC")
    rows = cur.fetchall()

    new_synced = 0
    for row in rows:
        msg_id = row["id"]
        ch = row["channel"]
        sender = row["sender"]
        content = row["content"]
        created = row["created_at"]

        if msg_id in synced_ids or ch in ["error-log", "dm"]:
            continue

        target_issue = determine_target_issue(ch, content, sender)
        body_md = f"### ?? `{sender}` (Pesan #{msg_id} via `#{ch}`)\n*Waktu: {created} UTC*\n\n{content}"

        try:
            if post_comment(target_issue, body_md):
                synced_ids.add(msg_id)
                new_synced += 1
                print(f"Synced msg #{msg_id} ({sender} -> #{ch}) to Issue #{target_issue}")
                time.sleep(0.7)
                if new_synced >= 15:
                    break
        except Exception as e:
            print(f"[Error syncing msg #{msg_id} to Issue #{target_issue}]: {e}")
            break

    try:
        with open(STATE_FILE, "w") as f:
            json.dump(list(synced_ids), f)
    except Exception:
        with open("/tmp/github_synced_ids.json", "w") as f:
            json.dump(list(synced_ids), f)

    print(f"Sync complete. Newly posted: {new_synced}, Total tracked: {len(synced_ids)}")

if __name__ == "__main__":
    main()
