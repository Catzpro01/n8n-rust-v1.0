#!/usr/bin/env python3
"""L4 Hydration extraction — reference implementation (stdlib only).
Kontrak: AGENT7-W2-SCRAPE-L4-SPEC.md v0.2 §4 (pure function, tanpa IO).
Port ke Rust: modul pure `hydrate::extract(&[u8], keys, selector_path, max_bytes)`.
Error taxonomy: L4_PARSE, L4_HYDRATION_NOT_FOUND, L4_SELECTOR_MISSING, L4_SELECTOR_EMPTY, L4_OVERSIZE.
"""
import json
import re

class L4Err(Exception):
    def __init__(self, code, detail=""):
        self.code = code
        self.detail = detail
        super().__init__(f"{code}: {detail}")

L4_PARSE = "L4_PARSE"
L4_HYDRATION_NOT_FOUND = "L4_HYDRATION_NOT_FOUND"
L4_SELECTOR_MISSING = "L4_SELECTOR_MISSING"
L4_SELECTOR_EMPTY = "L4_SELECTOR_EMPTY"
L4_OVERSIZE = "L4_OVERSIZE"

_MAX_DEPTH = 24
_SCRIPT_RE = re.compile(
    r'<script\b[^>]*\bid=["\']([A-Za-z0-9_]+)["\'][^>]*type=["\']application/json[^"\']*["\'][^>]*>(.*?)</script>',
    re.IGNORECASE | re.DOTALL,
)
_SCRIPT_RE2 = re.compile(
    r'<script\b[^>]*type=["\']application/json[^"\']*["\'][^>]*\bid=["\']([A-Za-z0-9_]+)["\'][^>]*>(.*?)</script>',
    re.IGNORECASE | re.DOTALL,
)
_NUXT_ASSIGN_RE = re.compile(r'window\.__NUXT__\s*=\s*(\{.*?\});?\s*</script>', re.DOTALL)


def _depth_ok(obj, d=0):
    if d > _MAX_DEPTH:
        return False
    if isinstance(obj, dict):
        return all(_depth_ok(v, d + 1) for v in obj.values())
    if isinstance(obj, list):
        return all(_depth_ok(v, d + 1) for v in obj)
    return True


def _walk(obj, path):
    """Dot-notation selector: obj & array only; array index via [n]? minimal: keys + digits."""
    cur = obj
    for part in path.split("."):
        if not part:
            continue
        if isinstance(cur, dict) and part in cur:
            cur = cur[part]
        elif isinstance(cur, list) and part.isdigit() and int(part) < len(cur):
            cur = cur[int(part)]
        else:
            return None
    return cur


def _truncate_at_boundary(obj, max_bytes):
    """Potong deterministik di batas objek JSON; kembalikan (obj_baru, truncated)."""
    if isinstance(obj, dict):
        out, trunc = {}, False
        for k, v in obj.items():
            if len(json.dumps(out, separators=(",", ":"), default=str).encode()) >= max_bytes:
                return out, True
            vv, t = _truncate_at_boundary(v, max_bytes)
            out[k] = vv
            trunc = trunc or t
        return out, trunc
    if isinstance(obj, list):
        out, trunc = [], False
        for v in obj:
            if len(json.dumps(out, separators=(",", ":"), default=str).encode()) >= max_bytes:
                return out, True
            vv, t = _truncate_at_boundary(v, max_bytes)
            out.append(vv)
            trunc = trunc or t
        return out, trunc
    return obj, False


def extract(html, keys=("__NEXT_DATA__", "__NUXT__"), selector_path=None, max_bytes=512_000):
    """Jalur-A: parse script-tag JSON (tanpa eksekusi). Mengembalikan HydrationItem dict."""
    html = html if isinstance(html, bytes) else html.encode("utf-8", "replace")
    text = html.decode("utf-8", "replace")
    found = None
    for m in list(_SCRIPT_RE.finditer(text)) + list(_SCRIPT_RE2.finditer(text)):
        key, body = m.group(1), m.group(2)
        if key in keys:
            found = (key, body)
            break
    if found is None:
        # Nuxt gaya lama: window.__NUXT__ = {...} (objek JS). Best-effort: teks + parse bila valid.
        for key in keys:
            if key == "__NUXT__":
                for m in _NUXT_ASSIGN_RE.finditer(text):
                    body = m.group(1).strip().rstrip(";")
                    return {"source": "script_tag", "key": key,
                            "payload_json": None, "script_text": body,
                            "selector_path": selector_path, "truncated": False,
                            "byte_len": len(body), "note": "js_object_best_effort"}
        raise L4Err(L4_HYDRATION_NOT_FOUND, f"keys={list(keys)} tidak ditemukan di HTML")
    key, body = found
    body = body.strip()
    try:
        payload = json.loads(body)
    except json.JSONDecodeError as e:
        raise L4Err(L4_PARSE, f"JSON invalid pada key={key} posisi {e.pos}") from e
    if not _depth_ok(payload):
        raise L4Err(L4_PARSE, "kedalaman > 24 (bom rekursi dicegah)")
    raw_len = len(body)
    # MISSING vs EMPTY dibedakan atas payload UTUH (sebelum truncation):
    # MISSING = komponen path tak ada (struktur berubah) ; EMPTY = null legit.
    result_val = payload
    if selector_path:
        cur = payload
        missing = False
        for part in selector_path.split("."):
            if not part:
                continue
            if isinstance(cur, dict) and part in cur:
                cur = cur[part]
            elif isinstance(cur, list) and part.isdigit() and int(part) < len(cur):
                cur = cur[int(part)]
            else:
                missing = True
                break
        if missing:
            raise L4Err(L4_SELECTOR_MISSING,
                        f"path '{selector_path}' tidak ada di struktur payload (layout berubah)")
        if cur is None:
            raise L4Err(L4_SELECTOR_EMPTY, f"path '{selector_path}' ada namun null")
        result_val = cur
    # catatan: [] (array kosong) = data legit nol -> payload kosong SUCCESS, bukan error (kontrak §3.1)
    truncated = False
    if len(json.dumps(result_val, separators=(",", ":"), default=str).encode()) > max_bytes:
        result_val, truncated = _truncate_at_boundary(result_val, max_bytes)
    return {"source": "script_tag", "key": key,
            "payload_json": result_val,
            "script_text": None, "selector_path": selector_path,
            "truncated": truncated, "partial_reason": "truncated" if truncated else None,
            "byte_len": raw_len}


# --- PII/redaksi (kontrak §7.4; implementasi penuh saat integrasi agent10) ---
_PII_EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
_PII_PHONE = re.compile(r"(?<!\d)(\+?[0-9]{1,3}[-. ]?)?(\(?[0-9]{2,4}\)?[-. ]?)?[0-9]{3,4}[-. ][0-9]{3,4}(?!\d)")
_PII_TOKEN = re.compile(r"(?i)(bearer\s+[A-Za-z0-9\-._~+/]+=*|ghp_[A-Za-z0-9]{36}|sk-[A-Za-z0-9]{20,})")


def sanitize_pii(text):
    """Redaksi email/telepon/token sebelum masuk log (G-L4-6)."""
    if not isinstance(text, str):
        text = json.dumps(text)
    text = _PII_EMAIL.sub("[EMAIL]", text)
    text = _PII_TOKEN.sub("[TOKEN]", text)
    text = _PII_PHONE.sub("[PHONE]", text)
    return text


if __name__ == "__main__":
    import sys
    p = sys.argv[1] if len(sys.argv) > 1 else "fixtures/f1_next13.html"
    html = open(p).read()
    r = extract(html, selector_path=sys.argv[2] if len(sys.argv) > 2 else None)
    print(json.dumps(r, indent=1, default=str)[:800])
