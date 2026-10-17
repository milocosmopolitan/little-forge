import fs from 'node:fs';import path from 'node:path';import {W,H,read,save,bounds} from './raster.mjs';
export function pack(dir,animation){
 const pageW=1024,maxH=2048,page=Buffer.alloc(pageW*maxH*4),sprites=[],dedup=new Map();let x=2,y=2,rowH=0;
 for(const mode of ['equipped','base'])for(const id of Object.keys(animation.poses)){
  const source='frames/'+(mode==='base'?'base/':'')+id+'.png',b=read(path.join(dir,source)),bb=bounds(b).box,w=bb[2]-bb[0],h=bb[3]-bb[1],crop=Buffer.alloc(w*h*4);
  for(let yy=0;yy<h;yy++)b.copy(crop,yy*w*4,((yy+bb[1])*W+bb[0])*4,((yy+bb[1])*W+bb[2])*4);
  for(let i=0;i<crop.length;i+=4)if(!crop[i+3])crop.fill(0,i,i+4);
  const key=w+'x'+h+':'+crop.toString('base64');let rect=dedup.get(key);
  if(!rect){if(x+w+2>pageW){x=2;y+=rowH+4;rowH=0;}if(y+h+2>maxH)throw Error('Atlas overflow');rect=[x,y,w,h];
   // One edge extrusion pixel and one transparent gutter in all directions.
   for(let yy=-1;yy<=h;yy++)for(let xx=-1;xx<=w;xx++){const sx=Math.max(0,Math.min(w-1,xx)),sy=Math.max(0,Math.min(h-1,yy));crop.copy(page,((y+yy)*pageW+x+xx)*4,(sy*w+sx)*4,(sy*w+sx)*4+4);}
   dedup.set(key,rect);x+=w+4;rowH=Math.max(rowH,h);
  }
  sprites.push({id:(mode==='base'?'base/':'')+id,page:'player',rect,source_size:[W,H],trim_offset:bb.slice(0,2),pivot:[64,140],source,rotated:false});
 }
 const pageH=Math.ceil((y+rowH+2)/16)*16;save(path.join(dir,'atlas.png'),page.subarray(0,pageW*pageH*4),pageW,pageH);
 const clips={...animation.clips};for(const[id,c]of Object.entries(animation.clips))clips['base/'+id]={...c,frames:c.frames.map(fr=>({...fr,sprite:'base/'+fr.sprite}))};
 const m={version:1,sampler:'nearest',mipmaps:false,extrude_px:1,gutter_px:1,pages:[{id:'player',path:'atlas.png',size:[pageW,pageH]}],sprites,clips,runtime_integration:false};
 fs.writeFileSync(path.join(dir,'atlas-manifest.json'),JSON.stringify(m,null,2)+'\n');return {pages:1,size:[pageW,pageH],sprites:sprites.length,unique_payloads:dedup.size,rgba8_bytes:pageW*pageH*4};
}
