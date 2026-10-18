import fs from 'node:fs';import path from 'node:path';
import {tile,register,remap,save} from '../raster.mjs';import {motionFactory} from './motion.mjs';
const src=path.resolve('assets/characters/source/animation'),jobs=JSON.parse(fs.readFileSync(path.join(src,'prompts.json')));
for(const sex of ['female','male']){
 const dir=path.resolve('assets/characters',sex),colors=JSON.parse(fs.readFileSync(path.join(dir,'source/palette.json'))).map(h=>[...Buffer.from(h.slice(1),'hex')]),locks={};
 function raw(job,k){const j=jobs.find(j=>j.id===sex+'-'+job),p=path.join(src,j.id+'.png');locks[job]??=register(tile(p,j.cols,j.rows,0),{height:117}).registration;const t=tile(p,j.cols,j.rows,k),m=register(t,{height:117}).registration,l=locks[job],locked={...l,sample_origin:[m.sample_origin[0]+64*m.sample_step-64*l.sample_step,l.sample_origin[1]]};return remap(register(t,{height:117,locked}).image,colors);}
 const make=motionFactory({dir,sex,source:src,colors}),out=path.join(src,'mobile-poses-v14/candidates',sex);
 const selected=[['walk-opposite',0,'walk-0'],['walk-opposite',1,'walk-1'],['walk',2,'walk-2'],['walk',0,'walk-3'],['walk',1,'walk-4'],['walk-opposite',2,'walk-5'],['gestures',3,'give-half'],['gestures',4,'give-full'],['gestures',5,'give-release']];
 for(const[job,k,key]of selected){
  const b=raw(job,k),r=make(b,key,'source/animation/'+sex+'-'+job+'.png');
  for(const[n,im]of Object.entries({raw:b,base:r.base,composed:r.composed,upper:r.layers['body/upper-base.png'],front:r.layers['body/arms-front.png'],gloves:r.layers['equipment/gloves.png']}))save(path.join(out,key+'-'+n+'.png'),im);
  fs.writeFileSync(path.join(out,key+'.json'),JSON.stringify(r.pose,null,2)+'\n');
 }
}
