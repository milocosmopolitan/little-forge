import fs from 'node:fs';
import path from 'node:path';
import {magick} from '../raster.mjs';
for(const sex of ['female','male']){
 const root=path.resolve('assets/characters',sex),dir=path.join(root,'previews/hip-pivot-hammering');
 fs.rmSync(dir,{recursive:true,force:true});
 fs.mkdirSync(dir,{recursive:true});
 for(const [mode,name] of [['','equipped-motion.gif'],['base/','motion.gif']]){
  const args=[];
  for(let k=0;k<6;k++)args.push('-delay',[14,18,8,8,12,16][k],'(',path.join(root,'frames',mode,'hammering-'+k+'.png'),'-background','#eee5cd','-alpha','remove','-filter','point','-resize','400%',')');
  magick([...args,'-loop','0',path.join(dir,name)]);
 }
 const keys=Array.from({length:6},(_,k)=>path.join(root,'frames','hammering-'+k+'.png'));
 magick(['montage',...keys,'-background','#eee5cd','-alpha','remove','-filter','point','-geometry','512x640+4+4','-tile','6x1',path.join(dir,'equipped-keys.png')]);
 fs.writeFileSync(path.join(dir,'registration.json'),JSON.stringify({status:'review_candidate',source:'../../animation.json',method:'Current coherent pose frames; head and torso bend together around planted hips.'},null,2)+'\n');
}
