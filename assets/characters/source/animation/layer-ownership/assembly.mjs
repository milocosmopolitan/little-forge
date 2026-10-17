import {W,H,empty,select,grow,over,skin} from '../raster.mjs';
import {fabricMask} from '../style-restoration/neck-assembly.mjs';
import {faceRegion} from '../hip-pivot/coherent.mjs';

// These native pixels belong to the reference neck/collar, not the moving head.
export const neckRows={female:{63:[59,62],64:[59,65]},male:{62:[54,56],63:[54,60],64:[48,63]}};
export function headOnly(head,sex){
 return select(head,(x,y)=>{const range=neckRows[sex][y];return !range||x<range[0]||x>range[1];});
}
export function shirtMask(body){
 const candidates=fabricMask(body),out=empty(),seen=new Set();
 for(let k=0;k<W*H;k++){
  if(seen.has(k)||!candidates[k*4+3])continue;
  const group=[k];seen.add(k);
  for(let n=0;n<group.length;n++)for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
   const x=group[n]%W+dx,y=Math.floor(group[n]/W)+dy,j=y*W+x;
   if(x<0||x>=W||y<0||y>=H||seen.has(j)||!candidates[j*4+3])continue;
   seen.add(j);group.push(j);
  }
  // Single bright pixels in ears/hair are not shirt seeds.
  if(group.length>=4)for(const k of group)body.copy(out,k*4,k*4,k*4+4);
 }
 return out;
}
function inside(x,y,poly){
 let yes=false;for(let i=0,j=poly.length-1;i<poly.length;j=i++){
  const a=poly[i],b=poly[j];if((a[1]>y)!==(b[1]>y)&&x<(b[0]-a[0])*(y-a[1])/(b[1]-a[1])+a[0])yes=!yes;
 }return yes;
}
export function assembleNeck(body,upper,measurement,shift,rootCollar,sex,head,faceLimit=null,sourceSkinOnly=false){
 const fabric=shirtMask(body),edge=grow(fabric,1);
 let oldFace=grow(faceRegion(body).mask,2);
 // A thinking hand touches the face but remains an arm below the underlayer jaw.
 if(faceLimit)oldFace=select(oldFace,(x,y)=>y<faceLimit[3]);
 // Delete the old face contour even when it is connected to the shirt.
 const clean=select(upper,(x,y)=>!oldFace[(y*W+x)*4+3]||edge[(y*W+x)*4+3]);
 const garment=select(body,(x,y)=>edge[(y*W+x)*4+3]);
 const [cx,cy]=measurement.point,[rx,ry]=rootCollar;
 const top=sex==='female'?[[rx-5+shift[0],ry-7+shift[1]],[rx+6+shift[0],ry-5+shift[1]]]:[[rx-4+shift[0],ry-7+shift[1]],[rx+7+shift[0],ry-3+shift[1]]];
 const polygon=[top[0],top[1],[cx+5,cy+3],[cx-5,cy+3]];
 const samples=[];
 for(let y=0;y<H;y++)for(let x=0;x<W;x++){
  const i=(y*W+x)*4;if(body[i+3]&&inside(x+.5,y+.5,polygon)&&skin(body.subarray(i,i+4)))samples.push([x,y,i]);
 }
 if(!samples.length)throw Error('No source neck skin inside anatomical ownership polygon');
 const neck=empty();
 for(let y=0;y<H;y++)for(let x=0;x<W;x++){
  const i=(y*W+x)*4;if(!body[i+3]||edge[i+3]||!inside(x+.5,y+.5,polygon))continue;
  if(sourceSkinOnly){if(skin(body.subarray(i,i+4)))body.copy(neck,i,i,i+4);continue;}
  let nearest=samples[0],distance=Infinity;
  for(const s of samples){const d=(x-s[0])**2+(y-s[1])**2;if(d<distance){nearest=s;distance=d;}}
  body.copy(neck,i,nearest[2],nearest[2]+4);
 }
 if(sourceSkinOnly){
  const connected=over(garment,neck),seen=new Uint8Array(W*H),queue=[];
  for(let k=0;k<W*H;k++)if(garment[k*4+3]){seen[k]=1;queue.push(k);}
  for(let n=0;n<queue.length;n++)for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
   const x=queue[n]%W+dx,y=Math.floor(queue[n]/W)+dy,k=y*W+x;
   if(x<0||x>=W||y<0||y>=H||seen[k]||!connected[k*4+3])continue;
   seen[k]=1;queue.push(k);
  }
  // Source ear/face flecks above the garment are not part of the neck connector.
  for(let k=0;k<W*H;k++)if(!seen[k])neck.fill(0,k*4,k*4+4);
 }
 const silhouette=over(clean,garment,neck,head);
 let outline=null,score=Infinity;
 for(let i=0;i<head.length;i+=4)if(head[i+3]){const value=head[i]+head[i+1]+head[i+2];if(value<score){score=value;outline=head.subarray(i,i+4);}}
 // A contour is only needed at exposed neck/background boundaries, never across the collar.
 for(let y=0;y<H;y++)for(let x=1;x<W-1;x++){
  const i=(y*W+x)*4;if(!sourceSkinOnly&&neck[i+3]&&!head[i+3]&&(!silhouette[i-4+3]||!silhouette[i+4+3]))outline.copy(neck,i);
 }
 return {upper:over(clean,garment,neck),neck,garment,polygon,removed:select(upper,(x,y)=>oldFace[(y*W+x)*4+3]&&!fabric[(y*W+x)*4+3])};
}
