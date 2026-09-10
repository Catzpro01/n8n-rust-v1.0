const http = require('http');
const fs = require('fs');
const path = require('path');

const UI_PATH = path.join(__dirname, 'n8n-rust/crates/n8n-server/ui/app.html');
const PORT = process.env.PORT ? parseInt(process.env.PORT) : 3000;
const HOST = '0.0.0.0';

let hooks = new Map();
let runs = [];
let nextId = 1;

function json(res, code, data){
  res.writeHead(code, {
    'Content-Type':'application/json',
    'Access-Control-Allow-Origin':'*',
    'Access-Control-Allow-Methods':'GET,POST,PUT,PATCH,DELETE,OPTIONS',
    'Access-Control-Allow-Headers':'Content-Type,Authorization,X-Requested-With',
  });
  res.end(JSON.stringify(data));
}
function text(res, code, data, ctype='text/html'){
  res.writeHead(code, {
    'Content-Type': ctype,
    'Access-Control-Allow-Origin':'*',
  });
  res.end(data);
}

function mockTopo(wf){
  const enabled = (wf.nodes||[]).filter(n=>!n.disabled);
  const incoming = new Map(enabled.map(n=>[n.name,[]]));
  for(const n of enabled){
    const brs = (wf.connections?.[n.name]?.main)||[];
    for(const b of brs){
      if(!Array.isArray(b)) continue;
      for(const l of b){
        if(l && l.node && incoming.has(l.node)){
          incoming.get(l.node).push(n.name);
        }
      }
    }
  }
  const done=new Set(), order=[];
  let progress=true;
  while(progress && order.length<enabled.length){
    progress=false;
    for(const n of enabled){
      if(done.has(n.name)) continue;
      const deps=incoming.get(n.name)||[];
      if(deps.every(d=>done.has(d))){
        done.add(n.name); order.push(n.name); progress=true;
      }
    }
  }
  if(order.length!==enabled.length){
    const stuck=enabled.filter(n=>!done.has(n.name)).map(n=>n.name).join(", ");
    throw new Error("cycle detected, stuck at: "+stuck);
  }
  return order;
}

function mockValidate(wf){
  const diags=[];
  const NODE_TYPES = new Set([
    "n8n-nodes-base.manualTrigger","n8n-nodes-base.scheduleTrigger","n8n-nodes-base.webhook",
    "n8n-nodes-base.set","n8n-nodes-base.if","n8n-nodes-base.filter","n8n-nodes-base.switch",
    "n8n-nodes-base.merge","n8n-nodes-base.sort","n8n-nodes-base.limit","n8n-nodes-base.httpRequest",
    "n8n-nodes-base.code","n8n-nodes-base.function","n8n-nodes-base.dateTime","n8n-nodes-base.noOp",
    "n8n-nodes-base.wait","n8n-nodes-base.respondToWebhook","n8n-nodes-base.stopAndError","n8n-nodes-base.executeCommand"
  ]);
  const names=new Set((wf.nodes||[]).map(n=>n.name));
  for(const n of (wf.nodes||[])){
    if(!NODE_TYPES.has(n.type)) diags.push({level:"error",message:`node '${n.name}': tipe tak dikenal '${n.type}'`,node:n.name});
  }
  for(const from of Object.keys(wf.connections||{})){
    const m=wf.connections[from]?.main; if(!Array.isArray(m)) continue;
    for(const br of m){ if(!Array.isArray(br)) continue; for(const l of br){ if(l&&l.node&&!names.has(l.node)) diags.push({level:"error",message:`edge gantung: '${from}' -> '${l.node}'`,node:from}); } }
  }
  try{ mockTopo(wf); }catch(e){ diags.push({level:"error",message:e.message}); }
  // isolated warning
  const targets=new Set();
  for(const n of (wf.nodes||[])){ for(const s of (wf.connections?.[n.name]?.main||[]).flat().filter(Boolean).map(x=>x.node).filter(Boolean)) targets.add(s); }
  for(const n of (wf.nodes||[])){
    const hasOut = (wf.connections?.[n.name]?.main||[]).flat().length>0;
    if(!hasOut && !targets.has(n.name) && (wf.nodes||[]).length>1) diags.push({level:"warning",message:`node '${n.name}' terisolasi`,node:n.name});
  }
  return diags;
}

function mockRun(wf){
  const order=mockTopo(wf);
  const outputs={}; const durations={};
  for(const name of order){
    const node=(wf.nodes||[]).find(n=>n.name===name);
    if(!node) continue;
    const type=node.type;
    let preds=[];
    for(const k of Object.keys(wf.connections||{})){
      const m=wf.connections[k]?.main||[];
      for(let bi=0; bi<m.length; bi++){
        const br=m[bi]; if(!Array.isArray(br)) continue;
        if(br.some(l=>l&&l.node===name)){
          const out=outputs[k];
          if(out && out[bi]) preds.push(...out[bi]);
        }
      }
    }
    let items=[];
    if(preds.length===0){
      if(type.includes("manualTrigger")||type.includes("scheduleTrigger")) items=[{}];
      else if(type.includes("webhook")) items=[{mode:"manual",timestamp:new Date().toISOString()}];
      else items=[{}];
    } else items=preds;

    if(type==="n8n-nodes-base.set"){
      const assign=node.parameters?.assignments?.assignments||[];
      const values=node.parameters?.values||{};
      items=items.map(it=>{
        const base={...it};
        for(const a of assign){ if(a.name) base[a.name]=a.value; }
        for(const [k,v] of Object.entries(values)){ base[k]=v; }
        return base;
      });
    } else if(type==="n8n-nodes-base.limit"){
      const max=node.parameters?.maxItems||1;
      items=items.slice(0,max);
    } else if(type==="n8n-nodes-base.filter"){
      items=items.filter((_,i)=>i%2===0);
    } else if(type==="n8n-nodes-base.code"||type==="n8n-nodes-base.function"){
      items=items.map(it=>({...it, coded:true, at:Date.now()}));
    } else if(type==="n8n-nodes-base.executeCommand"){
      items=[{command: node.parameters?.command||"echo halo", stdout:"halo\n", stderr:"", exitCode:0}];
    } else if(type==="n8n-nodes-base.dateTime"){
      items=items.map(it=>({...it, currentDate:new Date().toISOString()}));
    }
    outputs[name]=[items];
    durations[name]=Math.floor(Math.random()*60)+8;
  }
  const total=Object.values(durations).reduce((a,b)=>a+b,0);
  return {outputs, order, durations_ms:durations, total_ms:total};
}

const server = http.createServer((req,res)=>{
  const url = new URL(req.url, `http://${req.headers.host}`);
  const pathname = url.pathname;

  if(req.method==='OPTIONS'){
    res.writeHead(204, {
      'Access-Control-Allow-Origin':'*',
      'Access-Control-Allow-Methods':'GET,POST,PUT,PATCH,DELETE,OPTIONS',
      'Access-Control-Allow-Headers':'Content-Type',
    });
    return res.end();
  }

  if(pathname==='/' && req.method==='GET'){
    try{
      const html = fs.readFileSync(UI_PATH,'utf8');
      return text(res,200,html);
    }catch(e){
      return text(res,500,"UI not found: "+e.message,'text/plain');
    }
  }

  if(pathname==='/health' && req.method==='GET'){
    return json(res,200,{status:"ok", uptime_secs: Math.floor(process.uptime()), version:"1.1.0", nodes:19, hooks:hooks.size, runs:runs.length});
  }
  if(pathname==='/api/nodes' && req.method==='GET'){
    return json(res,200,[
      "n8n-nodes-base.manualTrigger","n8n-nodes-base.scheduleTrigger","n8n-nodes-base.webhook",
      "n8n-nodes-base.set","n8n-nodes-base.if","n8n-nodes-base.filter","n8n-nodes-base.switch",
      "n8n-nodes-base.merge","n8n-nodes-base.sort","n8n-nodes-base.limit","n8n-nodes-base.httpRequest",
      "n8n-nodes-base.code","n8n-nodes-base.function","n8n-nodes-base.dateTime","n8n-nodes-base.noOp",
      "n8n-nodes-base.wait","n8n-nodes-base.respondToWebhook","n8n-nodes-base.stopAndError","n8n-nodes-base.executeCommand"
    ]);
  }
  if(pathname==='/api/metrics' && req.method==='GET'){
    return json(res,200,{uptime_secs:Math.floor(process.uptime()), total_runs:runs.length, hooks:hooks.size, nodes:19, version:"1.1.0"});
  }
  if(pathname==='/api/openapi.json' && req.method==='GET'){
    return json(res,200,{openapi:"3.0.0",info:{title:"n8n-rust API",version:"1.1.0"}});
  }
  if(pathname==='/api/runs' && req.method==='GET'){
    const limit = Math.min(100, parseInt(url.searchParams.get('limit')||'50'));
    return json(res,200,runs.slice(-limit).reverse());
  }
  if(pathname==='/api/hooks' && req.method==='GET'){
    return json(res,200,Array.from(hooks.keys()).sort());
  }

  // POST handlers need body
  if(req.method==='POST'){
    let body='';
    req.on('data',chunk=>body+=chunk);
    req.on('end',()=>{
      let data;
      try{ data=JSON.parse(body||'{}'); }catch{ data={}; }
      if(pathname==='/api/validate'){
        try{
          const diags=mockValidate(data);
          return json(res,200,diags);
        }catch(e){ return json(res,400,{message:e.message}); }
      }
      if(pathname==='/api/explain'){
        try{
          const order=mockTopo(data);
          return json(res,200,order);
        }catch(e){ return text(res,400,e.message,'text/plain'); }
      }
      if(pathname==='/api/run'){
        try{
          const report=mockRun(data);
          runs.push({id:nextId++, at_epoch:Math.floor(Date.now()/1000), workflow:data.name||"unnamed", ok:true, order:report.order, total_ms:report.total_ms});
          if(runs.length>100) runs.shift();
          return json(res,200,report);
        }catch(e){ return text(res,400,e.message,'text/plain'); }
      }
      if(pathname==='/api/hooks'){
        const p = (data.path||'').trim().replace(/^\/+|\/+$/g,'');
        if(!p || p.includes('/') || p.length>64) return text(res,400,'hook path harus satu segmen','text/plain');
        hooks.set(p, data.workflow||{name:p, nodes:[], connections:{}});
        return json(res,201,p);
      }
      return text(res,404,'not found','text/plain');
    });
    return;
  }

  if(pathname.startsWith('/api/hooks/') && req.method==='DELETE'){
    const p=pathname.replace('/api/hooks/','');
    const ok=hooks.delete(p);
    return json(res,200,ok);
  }

  if(pathname.startsWith('/hook/')){
    const p=pathname.replace('/hook/','').split('/')[0];
    const wf=hooks.get(p);
    if(!wf) return text(res,404,'hook tak dikenal: '+p,'text/plain');
    // simple fire
    try{
      const report=mockRun(wf);
      runs.push({id:nextId++, at_epoch:Math.floor(Date.now()/1000), workflow:wf.name||p, ok:true, order:report.order, total_ms:report.total_ms});
      if(runs.length>100) runs.shift();
      const mode=wf.nodes?.find(n=>n.type==='n8n-nodes-base.webhook')?.parameters?.responseMode||'onReceived';
      if(mode==='onReceived') return json(res,200,report);
      if(mode==='lastNode'){
        const last=report.order[report.order.length-1];
        const items=report.outputs[last]?.[0]||[];
        return json(res,200,items[0]||null);
      }
      return json(res,200,report);
    }catch(e){
      return text(res,400,e.message,'text/plain');
    }
  }

  return text(res,404,'not found','text/plain');
});

server.listen(PORT, HOST, ()=>{
  console.log(`n8n-rust premium preview running at http://${HOST}:${PORT}`);
  console.log(`UI: http://${HOST}:${PORT}/`);
});
