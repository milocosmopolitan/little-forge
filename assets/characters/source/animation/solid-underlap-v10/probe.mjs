import fs from 'node:fs';
import path from 'node:path';
import {read,save,over} from '../raster.mjs';
import {completeUpper} from './underlap.mjs';
const root=path.resolve('assets/characters/female'),out='/tmp/solid-underlap-v10';
const animation=JSON.parse(fs.readFileSync('assets/characters/source/animation/solid-underlap-v10/before-animation.json')),report=[];
for(const [id,pose]of Object.entries(animation.poses)){
 if(!pose.head_identity?.root_pixels_unchanged)continue;
 for(const equipped of [false,true]){
  const p=path.join(root,'poses',id),prefix=equipped?'rendered/':'';
  const upper=read(path.join('assets/characters/source/animation/solid-underlap-v10/before-upper',equipped?'equipped':'base',id+'.png'));
  const raw=read(path.join(p,'source/'+(equipped?'equipped':'body')+'-before-head-restoration.png'));
  const head=read(path.join(p,prefix+'head-base.png'));
  const identity=pose[equipped?'rendered_head_identity':'head_identity'];
  const repaired=completeUpper(raw,upper,head,identity);
  save(path.join('assets/characters/source/animation/solid-underlap-v10/contracts',equipped?'equipped':'base',id+'.png'),repaired.required);
  const lower=read(path.join(p,equipped?'rendered/body/lower.png':'body/lower-base.png'));
  const frame=over(lower,repaired.upper,head,read(path.join(p,prefix+'eyes/open.png')),read(path.join(p,prefix+'mouth/rest.png')));
  const dest=path.join(out,equipped?'equipped':'base');save(path.join(dest,id+'.png'),frame);
  save(path.join(out,equipped?'equipped-upper':'base-upper',id+'.png'),repaired.upper);
  let missingBody=0,missingVisible=0,added=0;
  const oldFrame=read(path.join('assets/characters/source/animation/solid-underlap-v10/before',equipped?'equipped':'base',id+'.png'));
  for(let i=0;i<upper.length;i+=4){
   if(repaired.required[i+3]&&!upper[i+3])missingBody++;
   if(repaired.required[i+3]&&!oldFrame[i+3])missingVisible++;
   if(repaired.upper[i+3]&&!upper[i+3])added++;
  }
  report.push({id,equipped,missing_required_body_before:missingBody,missing_visible_before:missingVisible,body_pixels_added:added});
 }
}
fs.mkdirSync(out,{recursive:true});fs.writeFileSync(path.join(out,'report.json'),JSON.stringify(report,null,2));
console.log(JSON.stringify({variants:report.length,visible_missing_before:report.reduce((s,r)=>s+r.missing_visible_before,0),affected_variants:report.filter(r=>r.missing_visible_before>0).length}));
