import fs from 'node:fs';
import path from 'node:path';
import {magick} from '../raster.mjs';
const root=path.resolve('assets/characters'),actions=['hammering','quenching','forging','researching','salvaging'];
for(const sex of ['female','male']){
 const dir=path.join(root,sex);
 fs.rmSync(path.join(root,'source/animation/style-restoration',sex),{recursive:true,force:true});
 const keys=actions.flatMap(a=>Array.from({length:6},(_,k)=>path.join(dir,'frames',a+'-'+k+'.png')));
 for(const [suffix,bg,geometry]of [['','#eee5cd','256x320'],['-dark','#22252b','256x320'],['-native','#eee5cd','128x160']])magick(['montage',...keys,'-background',bg,'-alpha','remove','-filter','point','-geometry',geometry+'+2+2','-tile','6x5',path.join(dir,'previews/work-all-30'+suffix+'.png')]);
 magick(['montage',path.join(dir,'neutral.png'),...['hammering-0','hammering-3','researching-0','researching-2'].map(id=>path.join(dir,'frames',id+'.png')),'-background','#eee5cd','-alpha','remove','-filter','point','-geometry','512x640+4+4','-tile','5x1',path.join(dir,'previews/work-style-reference.png')]);
}
magick([path.join(root,'female/previews/work-style-reference.png'),path.join(root,'male/previews/work-style-reference.png'),'-append',path.join(root,'qa/face-style-restored.png')]);
magick([path.join(root,'female/previews/work-actions-review.png'),path.join(root,'male/previews/work-actions-review.png'),'+append',path.join(root,'qa/work-scenes-repaired.png')]);
