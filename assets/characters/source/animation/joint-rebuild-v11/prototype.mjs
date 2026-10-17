import fs from 'node:fs';
import {read,save,over,move} from '../raster.mjs';
import {headOnly} from '../layer-ownership/assembly.mjs';
import {bodyWithoutHead} from './body.mjs';
import {faceRegion} from '../hip-pivot/coherent.mjs';
for(const sex of ['female','male']){
 const root='assets/characters/'+sex,a=JSON.parse(fs.readFileSync(root+'/animation.json'));
 for(const[id,p]of Object.entries(a.poses)){
  if(!p.head_identity?.root_pixels_unchanged)continue;
  for(const variant of ['base','equipped']){
   const eq=variant==='equipped',folder=root+'/poses/'+id;
   const raw=read(folder+'/source/'+(eq?'equipped':'body')+'-before-head-restoration.png');
   const prior=read(folder+'/'+(eq?'rendered/body/upper.png':'body/upper-base.png'));
   const identity=p[eq?'rendered_head_identity':'head_identity'];
   const underFace=faceRegion(read(folder+'/source/body-before-head-restoration.png')).box;
   const body=bodyWithoutHead(raw,prior,underFace,{point:identity.source_collar});
   const [cx,cy]=identity.source_collar,mount=sex==='female'?[62,66]:[57,64],shift=[cx-mount[0],cy-mount[1]];
   const head=move(over(headOnly(read(root+'/head-base.png'),sex),eq?read(root+'/equipment/'+(sex==='female'?'hair-tie':'headband')+'.png'):Buffer.alloc(128*160*4)),...shift);
   const final=over(read(folder+'/'+(eq?'rendered/body/lower.png':'body/lower-base.png')),body.upper,head,move(read(root+'/eyes/open.png'),...shift),move(read(root+'/mouth/rest.png'),...shift));
   const dir='/tmp/joint-rebuild-v11/'+sex+'/'+variant;save(dir+'/'+id+'.png',final);save(dir+'/'+id+'-body.png',body.upper);save(dir+'/'+id+'-head.png',head);
  }
 }
}
