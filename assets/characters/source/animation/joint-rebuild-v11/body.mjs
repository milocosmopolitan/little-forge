import {W,H,empty,select,over,grow,skin} from '../raster.mjs';
import {shirtMask} from '../layer-ownership/assembly.mjs';
import {faceRegion} from '../hip-pivot/coherent.mjs';

// Source-body ownership is independent of the approved head and its translation.
function hull(points){
 const sorted=[...new Map(points.map(p=>[p.join(','),p])).values()].sort((a,b)=>a[0]-b[0]||a[1]-b[1]);
 const cross=(a,b,c)=>(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]);
 const half=ps=>{const h=[];for(const p of ps){while(h.length>1&&cross(h.at(-2),h.at(-1),p)<=0)h.pop();h.push(p);}return h;};
 const a=half(sorted),b=half([...sorted].reverse());return [...a.slice(0,-1),...b.slice(0,-1)];
}
function inside(x,y,polygon){let yes=false;for(let i=0,j=polygon.length-1;i<polygon.length;j=i++){const a=polygon[i],b=polygon[j];if((a[1]>y)!==(b[1]>y)&&x<(b[0]-a[0])*(y-a[1])/(b[1]-a[1])+a[0])yes=!yes;}return yes;}
export function bodyWithoutHead(raw,partitionUpper,face,measurement,protectHand=false){
 const [cx,cy]=measurement.point;
 const fabric=shirtMask(raw),cloth=Buffer.from(fabric),seen=new Set();
 for(let k=0;k<W*H;k++){
  if(seen.has(k)||!fabric[k*4+3])continue;
  const q=[k];seen.add(k);
  for(let n=0;n<q.length;n++)for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
   const x=q[n]%W+dx,y=Math.floor(q[n]/W)+dy,j=y*W+x;
   if(x<0||x>=W||y<0||y>=H||seen.has(j)||!fabric[j*4+3])continue;seen.add(j);q.push(j);
  }
  // Grey source headbands can match the shirt palette, but do not reach the torso.
  if(q.every(j=>Math.floor(j/W)<cy-3)){for(const j of q){fabric.fill(0,j*4,j*4+4);cloth.fill(0,j*4,j*4+4);}continue;}
  const rows=new Map();for(const j of q){const x=j%W,y=Math.floor(j/W),r=rows.get(y)||[W,0];r[0]=Math.min(r[0],x);r[1]=Math.max(r[1],x);rows.set(y,r);}
  for(const[y,[lo,hi]]of rows)for(let x=lo;x<=hi;x++)cloth[(y*W+x)*4+3]=255;
 }
 const edge=grow(cloth,1),neck=empty(),samples=[];
 const neckArea=(x,y)=>x>=cx-4&&x<=cx+3&&y>=cy-3&&y<=cy+2;
 const garment=select(raw,(x,y,p)=>edge[(y*W+x)*4+3]&&(!neckArea(x,y)||fabric[(y*W+x)*4+3]||y>=cy));
 const sourceFace=faceRegion(raw);let removed=grow(sourceFace.mask,2);
 removed=select(removed,(x,y)=>y<(protectHand?cy:cy+3));
 if(sourceFace.box.some((v,i)=>v!==face[i]))removed=select(removed,(x,y)=>y<face[3]);
 const retained=select(partitionUpper,(x,y)=>(y>=cy||x<face[0]-4||x>=face[2]+4)&&!removed[(y*W+x)*4+3]);
 for(let y=cy-3;y<=cy+2;y++)for(let x=cx-4;x<=cx+3;x++){
  const i=(y*W+x)*4;if(raw[i+3]&&skin(raw.subarray(i,i+4)))samples.push([x,y,i]);
 }
 if(!samples.length)throw Error('Missing source neck pigment');
 for(let y=cy-3;y<=cy+2;y++)for(let x=cx-4;x<=cx+3;x++){
  const i=(y*W+x)*4;if(garment[i+3])continue;
  const nearest=samples.reduce((best,p)=>(p[0]-x)**2+(p[1]-y)**2<(best[0]-x)**2+(best[1]-y)**2?p:best);
  raw.copy(neck,i,nearest[2],nearest[2]+4);
 }
 const upper=over(retained,neck,garment),shoulder=empty(),points=[[cx-4,cy-3],[cx+4,cy-3],[cx-4,cy+3],[cx+4,cy+3]],swatches=[];
 for(let y=cy-8;y<=cy+3;y++)for(let x=cx-16;x<=cx+16;x++){
  const i=(y*W+x)*4;if(!fabric[i+3])continue;points.push([x,y],[x+1,y+1]);swatches.push([x,y,i]);
 }
 const polygon=hull(points);
 for(let y=cy-8;y<=cy+3;y++)for(let x=cx-16;x<=cx+16;x++){
  const i=(y*W+x)*4;if(upper[i+3]||neckArea(x,y)||!inside(x+.5,y+.5,polygon))continue;
  const nearest=swatches.reduce((best,p)=>(p[0]-x)**2+(p[1]-y)**2<(best[0]-x)**2+(best[1]-y)**2?p:best);
  raw.copy(shoulder,i,nearest[2],nearest[2]+4);
 }
 const complete=over(upper,shoulder);let outline=null,shade=Infinity;
 for(let i=0;i<raw.length;i+=4)if(raw[i+3]&&raw[i]+raw[i+1]+raw[i+2]<shade){shade=raw[i]+raw[i+1]+raw[i+2];outline=raw.subarray(i,i+4);}
 for(let y=cy-8;y<=cy+3;y++)for(let x=cx-16;x<=cx+16;x++){
  const i=(y*W+x)*4;if(shoulder[i+3]&&(!complete[i-W*4+3]||!complete[i-1]||!complete[i+7]))for(let c=0;c<4;c++)shoulder[i+c]=outline[c];
 }
 const final=over(upper,shoulder),required=empty();
 for(let y=cy-8;y<=cy+3;y++)for(let x=cx-16;x<=cx+16;x++){const i=(y*W+x)*4;if(inside(x+.5,y+.5,polygon)){required[i+3]=255;if(!final[i+3])throw Error('Incomplete source-owned shoulder plate');}}
 // Source head flecks can survive outside the shoulder envelope. They are not body.
 const visited=new Set();
 for(let k=0;k<W*H;k++){
  if(visited.has(k)||!final[k*4+3])continue;
  const q=[k];visited.add(k);
  for(let n=0;n<q.length;n++)for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
   const x=q[n]%W+dx,y=Math.floor(q[n]/W)+dy,j=y*W+x;
   if(x<0||x>=W||y<0||y>=H||visited.has(j)||!final[j*4+3])continue;visited.add(j);q.push(j);
  }
  if(q.length<=40&&q.every(j=>Math.floor(j/W)<cy+10&&j%W>=cx-24&&j%W<cx+28&&!required[j*4+3]))for(const j of q)final.fill(0,j*4,j*4+4);
 }
 return {upper:final,required,removed,neck,garment,retained,shoulder,polygon,neckPolygon:[[cx-4,cy-3],[cx+4,cy-3],[cx+4,cy+3],[cx-4,cy+3]],point:[cx,cy],policy:'Complete source shoulders and shirt; a body-owned 8px neck mount, independent of moving-head alpha'};
}
