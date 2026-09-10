const fs=require('fs'),vm=require('vm');
const d=JSON.parse(fs.readFileSync('expression-edge-cases.json','utf8'));

// bandingkan NILAI, bukan string: parse `expected` sebagai literal JS
const canon=v=>{try{return JSON.stringify(v)}catch(e){return String(v)}};
function ev(src,ctx){const c=vm.createContext(ctx||Object.create(null));return vm.runInContext(src,c,{timeout:200});}

// klasifikasi: butuh konteks n8n?
const n8nRe=/(^|[^A-Za-z0-9_$])\$(\b|\(|json|input|now|today|binary|node|items|execution|workflow|vars|env|secrets|itemIndex|runIndex|prevNode|fromAI|jmespath|parseJson|toDateTime|convertDateTime|isEmailValid|isValidJSON|base64|url|hash|roundTo|randomInt|uuid|randItem|shuffle|difference|intersection|merge|ifEmpty|ifNot|isEmpty|not)/;
// expected yang tidak bisa dibandingkan mesin
const nonComp=e=>/\bor\b|TZ_DEPENDENT|===|BLOCKED|object$|Generator|DEPENDS|N\/A|varies|error/i.test(e)&&!/^'|"|^-?\d|^\[|^\{|^true$|^false$|^NaN$|^undefined$|^null$|^Infinity$/.test(e);

const res=[];
for(const c of d.test_cases){
  const needsN8n=n8nRe.test(c.expr)||c.cat==='n8n_helpers'||c.cat==='n8n_scope';
  let actual,akind;
  try{actual=ev(c.expr);akind='ok';}
  catch(e){ if(e.code==='ERR_SCRIPT_EXECUTION_TIMEOUT'){actual='__TIMEOUT__';akind='timeout';}
            else {actual={__throws:e.constructor.name};akind='throw';} }
  let verdict,expVal=null;
  if(needsN8n) verdict='SKIP_NEEDS_N8N';
  else if(nonComp(c.expected)) verdict='SKIP_NON_COMPARABLE';
  else {
    try{expVal=ev('('+c.expected+')');}catch(e){expVal='__UNPARSEABLE__';}
    if(akind==='throw'&&expVal==='__UNPARSEABLE__') verdict='AMBIGUOUS';
    else if(akind==='throw') verdict=(expVal&&expVal.__throws)?'MATCH':'MISMATCH';
    else if(expVal==='__UNPARSEABLE__') verdict='SKIP_NON_COMPARABLE';
    else verdict=(canon(actual)===canon(expVal))?'MATCH':'MISMATCH';
  }
  res.push({...c,actual:canon(actual),expParsed:canon(expVal),verdict,akind,needsN8n});
}
const cnt=k=>res.filter(r=>r.verdict===k).length;
const tested=cnt('MATCH')+cnt('MISMATCH');
console.log('total                 :',res.length);
console.log('SKIP butuh konteks n8n:',cnt('SKIP_NEEDS_N8N'));
console.log('SKIP expected bukan literal (prosa/"or"):',cnt('SKIP_NON_COMPARABLE'));
console.log('BISA dibandingkan     :',tested);
console.log('  MATCH               :',cnt('MATCH'));
console.log('  MISMATCH            :',cnt('MISMATCH'));
console.log('  AKURASI agent3      :',(100*cnt('MATCH')/tested).toFixed(1)+'%');
console.log('\n=== MISMATCH NYATA (nilai expected agent3 salah menurut V8) ===');
for(const r of res.filter(r=>r.verdict==='MISMATCH'))
  console.log(`${r.id} [${r.cat}/${r.risk}]\n   expr        : ${r.expr}\n   agent3      : ${r.expected}   (parse: ${r.expParsed})\n   V8 SEBENARNYA: ${r.actual}`);
console.log('\n=== yang expected-nya prosa/tak bisa dibandingkan (perlu diperbaiki agent3) ===');
for(const r of res.filter(r=>r.verdict==='SKIP_NON_COMPARABLE'))
  console.log(`  ${r.id} [${r.cat}] ${r.expr.slice(0,60)}  -> expected: ${r.expected}`);
fs.writeFileSync('hasil-v8-final.json',JSON.stringify(res,null,1));
