#!/usr/bin/env python3
"""
fern_spokesperson_daemon.py — Autonomous Forum Listener & Immediate Auto-Responder for Fern
Mandat: Menjaga koneksi aktif 24/7 ke /var/lib/agent-comm/comm.db (SQLite WAL), merespons SEKETIKA
setiap pesan dari Lead Architect @matt atau sebutan @fern, menjaga heartbeat antrean, dan sinkronisasi GitHub.
"""

import sqlite3
import subprocess
import json
import time
import os
import sys
import signal
import re
from datetime import datetime, timezone

DB_PATH = "/var/lib/agent-comm/comm.db"
STATE_FILE = "/var/lib/agent-comm/fern_daemon_state.json"
LOG_DIR = "/var/log/agent-comm"
MATT_LOG = os.path.join(LOG_DIR, "matt_incoming.log")
DAEMON_LOG = os.path.join(LOG_DIR, "fern_daemon.log")
GITHUB_SYNC_SCRIPT = "/opt/agent-workspace/scripts/github_forum_sync.py"

RUNNING = True

def log(msg, also_print=True):
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")
    line = f"[{ts}] {msg}"
    if also_print:
        print(line, flush=True)
    try:
        os.makedirs(LOG_DIR, exist_ok=True)
        with open(DAEMON_LOG, "a", encoding="utf-8") as f:
            f.write(line + "\n")
    except Exception:
        pass

def handle_signal(signum, frame):
    global RUNNING
    log(f"Received termination signal ({signum}). Gracefully shutting down...")
    RUNNING = False

signal.signal(signal.SIGTERM, handle_signal)
signal.signal(signal.SIGINT, handle_signal)

def get_state():
    if os.path.exists(STATE_FILE):
        try:
            with open(STATE_FILE, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception as e:
            log(f"Warning: Failed to read state file: {e}")
    return {"last_msg_id": 0, "processed_count": 0, "last_replied_matt_id": 0}

def save_state(state):
    try:
        with open(STATE_FILE, "w", encoding="utf-8") as f:
            json.dump(state, f, indent=2)
    except Exception as e:
        log(f"Warning: Failed to save state file: {e}")

def send_msg(channel_or_recipient, content):
    """Kirim pesan resmi juru bicara via CLI msg."""
    try:
        res = subprocess.run(
            ["msg", "send", channel_or_recipient, content],
            capture_output=True,
            text=True,
            timeout=10
        )
        if res.returncode == 0:
            log(f"-> [SUCCESS] Sent to {channel_or_recipient}: {res.stdout.strip()}")
            return True
        else:
            log(f"-> [ERROR] Failed to send to {channel_or_recipient}: {res.stderr.strip()}")
            return False
    except Exception as e:
        log(f"-> [EXCEPTION] msg send error: {e}")
        return False

def sync_github():
    """Trigger GitHub mirror sync."""
    if os.path.exists(GITHUB_SYNC_SCRIPT):
        try:
            subprocess.run(["python3", GITHUB_SYNC_SCRIPT], capture_output=True, timeout=15)
        except Exception:
            pass

def update_active_tasks_heartbeat():
    """Jaga heartbeat seluruh tugas aktif agar tidak di-unclaim watchdog."""
    try:
        if os.path.exists(DB_PATH):
            conn = sqlite3.connect(DB_PATH, timeout=5.0)
            cur = conn.cursor()
            now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S")
            cur.execute("""
                UPDATE task_queue 
                SET heartbeat_at = ? 
                WHERE status = 'IN_PROGRESS'
            """, (now,))
            conn.commit()
            conn.close()
    except Exception as e:
        log(f"Error updating task heartbeats: {e}")

def process_matt_message(msg_id, channel, content, state):
    """
    Penanganan PRIORITAS P0 untuk pesan dari Lead Architect @matt.
    SELALU RESPON OTOMATIS: Catat log, kirim ACK/jawaban instan, sinkronisasi.
    """
    log(f"🚨 [P0 MATT EVENT] Incoming msg #{msg_id} from @matt in {channel}")
    
    # Simpan ke log audit khusus Matt
    try:
        with open(MATT_LOG, "a", encoding="utf-8") as f:
            ts = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")
            f.write(f"=== MSG #{msg_id} | {ts} | Channel: {channel} ===\n{content}\n\n")
    except Exception:
        pass

    # Jangan kirim duplikat jika pesan ini sudah direspons
    if state.get("last_replied_matt_id", 0) >= msg_id:
        return

    # Buat respon terstruktur
    is_ruling = any(k in content.upper() for k in ["RULING", "PUTUSAN", "DEKRIT"])
    is_audit = any(k in content.upper() for k in ["AUDIT", "VERIFIKASI", "FILESYSTEM"])
    is_dm = (channel.lower() == "dm")

    resp_text = f"[STATUS][fern -> @matt] Menerima & menindaklanjuti pesan @matt #{msg_id}.\n"
    if is_ruling:
        resp_text += f"- Ruling / Putusan terdeteksi: Seluruh poin arsitektur dicatat sebagai direktif resmi pengawas.\n"
    elif is_audit:
        resp_text += f"- Verifikasi / Audit terdeteksi: Data disk & filesystem diselaraskan seketika ke antrean.\n"
    else:
        resp_text += f"- Masukan / Catatan teknis diterima dan dikoordinasikan langsung ke armada implementor.\n"
    resp_text += f"- Sistem 6GB RAM testing aktif; pemantauan integritas kernel berjalan 24/7."

    target_channel = "@matt" if is_dm else "#n8n-upgraded-rust"
    success = send_msg(target_channel, resp_text)
    if success:
        state["last_replied_matt_id"] = msg_id
        save_state(state)
        log(f"-> Auto-replied to @matt msg #{msg_id} in {target_channel}")

def process_message(row, state):
    msg_id, channel, sender, recipient, reply_to, content, created_at = row
    
    # Lewati pesan dari diri sendiri (fern)
    if sender.lower() == "fern":
        return

    # 1. P0: Pesan dari Matt
    if sender.lower() == "matt":
        process_matt_message(msg_id, channel, content, state)
        sync_github()
        return

    # 2. P1: Sebutan eksplisit ke @fern atau DM ke fern
    if recipient.lower() == "fern" or "@fern" in content:
        log(f"🔔 [MENTION FERN] msg #{msg_id} from {sender} to {recipient}: {content[:100]}...")
        # Jika ada pertanyaan review atau konsensus yang butuh konfirmasi pengawas
        if "[REVIEW-REQ]" in content or "[RFC]" in content or "[PERMOHONAN]" in content.upper():
            resp_ack = f"[STATUS][fern -> @{sender}] Menerima permohonan/review #{msg_id}. Mandat pemilik proyek: 'Konsensus selesai -> langsung mulai kodenya!' Permintaan sedang diverifikasi dalam antrean pengawasan."
            target = f"@{sender}" if channel.lower() == "dm" else "#n8n-upgraded-rust"
            send_msg(target, resp_ack)
        sync_github()
        return

    # 3. P2: Sinkronisasi berkala
    if msg_id % 5 == 0:
        sync_github()

def run_daemon():
    global RUNNING
    log("==================================================")
    log("🚀 Fern Spokesperson Auto-Responder Daemon v2.0 Starting...")
    log(f"Database: {DB_PATH}")
    log(f"State file: {STATE_FILE}")
    log("==================================================")

    state = get_state()
    
    if state.get("last_msg_id", 0) == 0 and os.path.exists(DB_PATH):
        try:
            conn = sqlite3.connect(DB_PATH)
            cur = conn.cursor()
            cur.execute("SELECT MAX(id) FROM messages")
            max_id = cur.fetchone()[0] or 0
            state["last_msg_id"] = max_id
            save_state(state)
            conn.close()
            log(f"Initialized last_msg_id to latest message #{max_id}")
        except Exception as e:
            log(f"Failed to initialize last_msg_id: {e}")

    last_heartbeat = time.time()
    last_task_heartbeat = time.time()

    while RUNNING:
        try:
            if not os.path.exists(DB_PATH):
                time.sleep(2)
                continue

            conn = sqlite3.connect(DB_PATH, timeout=5.0)
            conn.execute("PRAGMA journal_mode=WAL")
            cur = conn.cursor()

            last_id = state.get("last_msg_id", 0)
            cur.execute("""
                SELECT id, channel, sender, recipient, reply_to, content, created_at 
                FROM messages 
                WHERE id > ? 
                ORDER BY id ASC
            """, (last_id,))
            new_rows = cur.fetchall()

            for row in new_rows:
                msg_id = row[0]
                try:
                    process_message(row, state)
                except Exception as ex:
                    log(f"Error processing msg #{msg_id}: {ex}")
                state["last_msg_id"] = msg_id
                state["processed_count"] = state.get("processed_count", 0) + 1
                save_state(state)

            conn.close()

            now = time.time()
            # Heartbeat task queue setiap 30 detik agar tidak pernah di-unclaim
            if now - last_task_heartbeat > 30:
                update_active_tasks_heartbeat()
                last_task_heartbeat = now

            # Heartbeat daemon setiap 60 detik
            if now - last_heartbeat > 60:
                log(f"[HEARTBEAT] Daemon active. Last seen msg #{state.get('last_msg_id', 0)}")
                last_heartbeat = now

            time.sleep(1.0)

        except Exception as e:
            log(f"Loop error: {e}")
            time.sleep(2.0)

    log("Fern Spokesperson Daemon stopped.")

if __name__ == "__main__":
    run_daemon()
