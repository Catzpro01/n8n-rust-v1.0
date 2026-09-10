#!/usr/bin/env python3
"""webhook_corpus_stats.py — statistik node Webhook & respondToWebhook pada korpus template nyata.
Read-only. Reproduksi: python3 /opt/agent-workspace/qa/webhook_corpus_stats.py [dir_korpus]
Penulis: agent9 (ROLE_INTEGRATION), 2026-09-09 — bukti untuk AGENT9-INTEGRATION-SPEC v0.2 (§2)."""
import json, glob, sys, collections, os

base = sys.argv[1] if len(sys.argv) > 1 else '/opt/agent-workspace/docs/corpus'
files = sorted(glob.glob(os.path.join(base, 'tpl-*.json')))
wm, rm, auth = collections.Counter(), collections.Counter(), collections.Counter()
tv = collections.Counter()
webhook_files = r2w_files = webhook_nodes = r2w_nodes = 0
webhookId = path_set = 0
examples = []
for fp in files:
    try:
        with open(fp) as f:
            d = json.load(f)
    except Exception:
        continue
    wf = d.get('workflow') if isinstance(d, dict) and isinstance(d.get('workflow'), dict) else d
    nodes = wf.get('nodes', []) if isinstance(wf, dict) else []
    hw = hr = False
    for n in nodes:
        t = n.get('type', '')
        if t == 'n8n-nodes-base.webhook':
            hw = True
            webhook_nodes += 1
            if 'webhookId' in n:
                webhookId += 1
            tv[str(n.get('typeVersion', '<absent>'))] += 1
            p = n.get('parameters', {}) or {}
            wm[str(p.get('httpMethod', '<absent>'))] += 1
            rm[str(p.get('responseMode', '<absent>'))] += 1
            if p.get('path') is not None:
                path_set += 1
            auth[str(p.get('authentication', 'none') or 'none')] += 1
            if len(examples) < 6 and p.get('responseMode') not in (None, 'onReceived'):
                examples.append([os.path.basename(fp), p.get('httpMethod'), p.get('responseMode'), p.get('path')])
        elif t == 'n8n-nodes-base.respondToWebhook':
            hr = True
            r2w_nodes += 1
    webhook_files += int(hw)
    r2w_files += int(hr)
out = {
    'files_total': len(files),
    'webhook_files': webhook_files,
    'webhook_nodes': webhook_nodes,
    'webhookId_present': webhookId,
    'path_set': path_set,
    'typeVersion': dict(tv),
    'httpMethod': dict(wm),
    'responseMode': dict(rm),
    'authentication': dict(auth),
    'respondToWebhook_files': r2w_files,
    'respondToWebhook_nodes': r2w_nodes,
    'examples_responseNode_lastNode': examples,
}
print(json.dumps(out, indent=1, ensure_ascii=False))
