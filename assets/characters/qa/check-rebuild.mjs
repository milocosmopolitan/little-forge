import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
const root=path.resolve('assets/characters');
function hashes(){
 const out={};
 function walk(dir){for(const e of fs.readdirSync(dir,{withFileTypes:true})){
  const p=path.join(dir,e.name);
  if(e.isDirectory())walk(p);
  else if(/\.(png|json)$/.test(p)&&!p.includes('/qa/')||p.includes('/qa/')&&p.endsWith('.png'))out[p]=crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
 }}
 for(const sex of ['female','male'])walk(path.join(root,sex));
 return out;
}
const before=hashes();
execFileSync(process.execPath,[path.join(root,'source/animation/build.mjs'),'--no-previews']);
const after=hashes(),changed=Object.keys(before).filter(p=>before[p]!==after[p]);
if(changed.length)throw Error('Rebuild changed authored output: '+JSON.stringify(changed));
const report={status:'pass',native_png_and_json_files:Object.keys(before).length,byte_identical:true,command:'node assets/characters/source/animation/build.mjs --no-previews',excluded:'QA JSON reports, which are independent check outputs'};
fs.writeFileSync(path.join(root,'qa/animation-rebuild.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report));
