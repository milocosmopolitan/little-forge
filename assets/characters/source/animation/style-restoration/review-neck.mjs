import fs from 'node:fs';
import path from 'node:path';
import {read,over,save,magick} from '../raster.mjs';
// Read final registered layers; never approximate the production neck with an older algorithm.
const root=path.resolve('assets/characters'),files=[];
for(const sex of ['female','male']){
 const dir=path.join(root,sex),animation=JSON.parse(fs.readFileSync(path.join(dir,'animation.json')));
 for(const action of ['researching-0','forging-0','quenching-0','quenching-1','hammering-0']){
  const frame=over(...animation.poses[action].rendered_variant_layers.map(layer=>read(path.join(dir,layer))));
  const file=path.join(root,'source/animation/style-restoration/neck-prototype',sex+'-'+action+'.png');save(file,frame);files.push(file);
 }
}
magick(['montage',...files,'-background','#22252b','-alpha','remove','-filter','point','-geometry','512x640+4+4','-tile','5x2',path.join(root,'qa/neck-prototype.png')]);
