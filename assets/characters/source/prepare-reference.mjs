import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const run=(args,input)=>execFileSync('magick',args,{input,maxBuffer:40*1024*1024});
const configs={female:{top:179,height:1150,body_center_x:560},male:{top:316,height:1043,body_center_x:571}};
for(const[id,c]of Object.entries(configs)){
 const dir=path.join(root,id,'source'),src=path.join(dir,'reference.png');
 const raw=run([src,'-depth','8','rgba:-']);
 const step=c.height/117,x0=c.body_center_x-48*step,y0=c.top-8*step;
 const out=Buffer.alloc(96*128*4);
 for(let y=0;y<128;y++)for(let x=0;x<96;x++){
  const sx=Math.floor(x0+(x+.5)*step),sy=Math.floor(y0+(y+.5)*step);
  if(sx<0||sx>=1086||sy<0||sy>=1448)continue;
  const i=(sy*1086+sx)*4,j=(y*96+x)*4;
  if(raw[i+3]>=128){raw.copy(out,j,i,i+3);out[j+3]=255;}
 }
 run(['-size','96x128','-depth','8','rgba:-','-strip','PNG32:'+path.join(dir,'reference-native.png')],out);
 run([path.join(dir,'reference-native.png'),'-filter','point','-resize','800%','-strip',path.join(dir,'reference-edit-target.png')]);
 fs.writeFileSync(path.join(dir,'reference-registration.json'),JSON.stringify({source_size:[1086,1448],canvas:[96,128],sample_step:step,sample_origin:[x0,y0],visible_height:117,foot_anchor:[48,124]},null,2)+'\n');
}
