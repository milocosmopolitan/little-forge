import fs from 'node:fs';
import path from 'node:path';
import {W,empty,read,over,move,bounds,skin} from '../raster.mjs';
import {collarSocket} from './neck-assembly.mjs';
import {headOnly} from '../layer-ownership/assembly.mjs';

export function headJoint(b){
 const box=bounds(b).box,xs=[];
 for(let y=box[3]-2;y<box[3];y++)for(let x=40;x<104;x++){
  const i=(y*W+x)*4;if(b[i+3]&&skin(b.subarray(i,i+4)))xs.push(x);
 }
 return [xs.length?Math.round(xs.reduce((a,b)=>a+b,0)/xs.length):Math.round((box[0]+box[2])/2),box[3]-1];
}
export function approvedWorkHeads(dir,source,sex,colors){
 const root={head:read(path.join(dir,'head-base.png')),eyes:read(path.join(dir,'eyes/open.png')),mouth:read(path.join(dir,'mouth/rest.png'))};
 const approvedHead=headOnly(root.head,sex);
 const rootMount=sex==='female'?[62,66]:[57,64];
 const ornament=path.join(dir,'equipment',sex==='female'?'hair-tie.png':'headband.png');
 const gear=fs.existsSync(ornament)?read(ornament):empty();
 return (target,{facing,equipped=false,body,face,key})=>{
  if(facing==='back')return {head:target,eyes:empty(),mouth:empty(),identity:{source:'unchanged coherent rear pose',shift:[0,0],root_pixels_unchanged:false,head_facing:'back'}};
  const measurement=collarSocket(body,face,facing),a=rootMount,b=measurement.point,shift=[b[0]-a[0],b[1]-a[1]];
  return {head:move(equipped?over(approvedHead,gear):approvedHead,...shift),eyes:move(root.eyes,...shift),mouth:move(root.mouth,...shift),identity:{source:'approved root head and features',shift,root_pixels_unchanged:true,head_ownership:'approved head excluding documented neck/collar rows',anatomical_head_adjustment:[0,0],head_translation_adjustment:[0,0],attachment_policy:'body-owned neck mount; one root attachment for every visible work key',neck_attachment:b,head_facing:'right_3_4',anchor:'body-owned short neck mount, independent of moving-head silhouette',root_collar:a,source_collar:b},measurement};
 };
}
