import fs from 'node:fs';import path from 'node:path';import {magick} from './raster.mjs';
const R=path.resolve('assets/characters'),catalog=JSON.parse(fs.readFileSync(path.join(R,'stations/catalog.json')));
for(const sex of ['female','male']){const d=path.join(R,sex),a=JSON.parse(fs.readFileSync(path.join(d,'animation.json')));
 for(const action of ['hammering','quenching','forging','researching','salvaging']){
  const c=a.clips[action],w=a.workshop.stations[action],s=catalog.stations[w.station_id],files=[];
  for(let k=0;k<c.frames.length;k++){
   const id=c.frames[k].sprite,actor=path.join(d,'frames',(action==='quenching'?'scene-':'')+id+'.png'),out=path.join(d,'previews/scenes',action+'-'+k+'.png');fs.mkdirSync(path.dirname(out),{recursive:true});const args=['-size','256x192','xc:#eee5cd'];
   const add=(file,xy)=>args.push('(',file,')','-geometry',`+${xy[0]}+${xy[1]}`,'-composite');
   if(action==='researching'){add(actor,[32,16]);add(path.join(R,s.path),w.origin);add(path.join(d,'poses',id,'body/arms-front.png'),[32,16]);add(path.join(d,'poses',id,'equipment/gloves.png'),[32,16]);}
   else if(action==='salvaging'||action==='forging'){add(path.join(R,s.path),w.origin);add(actor,[32,16]);}
   else{add(path.join(R,s.back),w.origin);add(path.join(R,s.fire),w.origin);add(path.join(R,s.water),w.origin);add(actor,[32,16]);add(path.join(R,s.front),w.origin);}
   magick([...args,'-strip','PNG32:'+out]);files.push(out);
  }
  const gif=[];for(let k=0;k<files.length;k++)gif.push('-delay',Math.round(c.frames[k].duration_ms/10),'(',files[k],'-filter','point','-resize','300%',')');
  magick([...gif,'-loop',c.loop?'0':'1','-strip',path.join(d,'previews',action+'-scene.gif')]);
 }
 const sheet=[];for(const action of ['hammering','quenching','forging','researching','salvaging'])for(const k of [0,3,5])sheet.push('-label',action+' '+k,path.join(d,'previews/scenes',action+'-'+k+'.png'));
 magick(['montage',...sheet,'-font','Arial','-pointsize','14','-background','#eee5cd','-filter','point','-geometry','512x384+4+4','-tile','3x5','-strip',path.join(d,'previews/work-actions-review.png')]);
}
