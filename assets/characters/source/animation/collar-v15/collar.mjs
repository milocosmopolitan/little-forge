import {W,H,empty,read,over,move,select,grow,skin} from '../raster.mjs';
import {shirtMask,headOnly} from '../layer-ownership/assembly.mjs';
import path from 'node:path';

// Native source neck contours, traced at the jaw exit and actual shirt opening.
// Head translation seats the approved jaw at that exit, independent of face bounds.
const contours={
 female:{'walk-0':[66,73],'walk-1':[66,73],'walk-2':[63,70],'walk-3':[63,70],'walk-4':[63,70],'walk-5':[66,73],'give-half':[65,71],'give-full':[67,71],'give-release':[67,72]},
 male:{'walk-0':[63,72],'walk-1':[66,75],'walk-2':[66,76],'walk-3':[65,75],'walk-4':[67,76],'walk-5':[63,72],'give-half':[63,72],'give-full':[63,70],'give-release':[63,72]}
};
export function sourceCollar(raw,sex,key){
 const range=contours[sex][key];if(!range)throw Error('No authored collar '+sex+'/'+key);
 const [top,bottom]=range,cloth=shirtMask(raw),seen=new Set();
 // Only shirt components reaching the chest may seed garment coverage. Source
 // ears and headbands can share cream pigments but never reach the chest.
 for(let k=0;k<W*H;k++){
  if(seen.has(k)||!cloth[k*4+3])continue;const q=[k];seen.add(k);
  for(let n=0;n<q.length;n++)for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
   const x=q[n]%W+dx,y=Math.floor(q[n]/W)+dy,j=y*W+x;if(x<0||x>=W||y<0||y>=H||seen.has(j)||!cloth[j*4+3])continue;seen.add(j);q.push(j);
  }
  if(q.every(j=>Math.floor(j/W)<bottom))for(const j of q)cloth.fill(0,j*4,j*4+4);
 }
 const near=grow(cloth,1),neck=empty(),required=empty(),rows={};
 for(let y=top;y<=bottom;y++){
  const xs=[];for(let x=50;x<83;x++){const i=(y*W+x)*4;if(raw[i+3]&&skin(raw.subarray(i,i+4)))xs.push(x);}
  if(!xs.length)throw Error('Missing authored neck row '+sex+'/'+key+'/'+y);
  rows[y]=[Math.min(...xs)-1,Math.max(...xs)+2];
 }
 const first=rows[top],rootExit=sex==='female'?[60.5,63]:[55,62];
 const shift=[Math.round((first[0]+1+first[1]-2)/2-rootExit[0]),top-rootExit[1]],point=[(sex==='female'?62:57)+shift[0],(sex==='female'?66:64)+shift[1]];
 const upper=select(raw,(x,y,p)=>{
  if(y>bottom)return true;
  const span=rows[y],n=!!span&&x>=span[0]&&x<=span[1],fabric=!!near[(y*W+x)*4+3];
  if(n&&p[3])raw.copy(neck,(y*W+x)*4,(y*W+x)*4,(y*W+x)*4+4);
  if((n||fabric)&&p[3])required[(y*W+x)*4+3]=255;
  return n||fabric;
 });
 return {upper,neck,required,shift,point,joint:{jaw_exit:top,collar_end:bottom,neck_rows:rows,root_exit:rootExit}};
}
export function collarHead(dir,sex,shift,equipped=false){
 const head=headOnly(read(path.join(dir,'head-base.png')),sex),gear=equipped?read(path.join(dir,'equipment',sex==='female'?'hair-tie.png':'headband.png')):empty();
 return {head:move(over(head,gear),...shift),eyes:move(read(path.join(dir,'eyes/open.png')),...shift),mouth:move(read(path.join(dir,'mouth/rest.png')),...shift)};
}
