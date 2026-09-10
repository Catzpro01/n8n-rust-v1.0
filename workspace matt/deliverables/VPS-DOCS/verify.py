import json, re, sys
t = json.load(open('tree.json'))['tree']
assert not t or True
d = json.load(open('tree.json'))
if d.get('truncated'): sys.exit('FATAL: tree terpotong, hitungan tidak lengkap')
blobs = [e['path'] for e in d['tree'] if e['type'] == 'blob']

def istest(p): return re.search(r'\.(test|spec)\.ts$', p) is not None

NB = 'packages/nodes-base/'
LC = 'packages/@n8n/nodes-langchain/'

paket = [p for p in blobs if p.endswith('/package.json')]
node  = [p for p in blobs if p.endswith('.node.ts') and not istest(p)
         and (p.startswith(NB + 'nodes/') or p.startswith(LC + 'nodes/'))]
cred  = [p for p in blobs if p.endswith('.ts') and not istest(p)
         and re.match(re.escape(NB) + r'credentials/[^/]+\.ts$', p)
         or (p.endswith('.ts') and not istest(p)
             and re.match(re.escape(LC) + r'credentials/[^/]+\.ts$', p))]
ent   = [p for p in blobs if p.startswith('packages/@n8n/db/src/entities/')
         and p.endswith('.ts') and not istest(p) and not p.endswith('/index.ts')]
nbdir = sorted(set(p.split('/')[3] for p in node if p.startswith(NB)))

print('truncated            :', d.get('truncated'))
print('paket (package.json) :', len(paket))
print('node  (*.node.ts)    :', len(node), '= nodes-base',
      sum(1 for p in node if p.startswith(NB)), '+ langchain',
      sum(1 for p in node if p.startswith(LC)))
print('direktori node NB    :', len(nbdir))
print('credential type      :', len(cred))
print('entity DB            :', len(ent), '(enterprise .ee:',
      sum(1 for p in ent if '.ee.' in p), ')')
