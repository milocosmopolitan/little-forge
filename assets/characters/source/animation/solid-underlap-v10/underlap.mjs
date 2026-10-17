import {W,H,empty,select,over,grow,skin} from '../raster.mjs';
import {shirtMask} from '../layer-ownership/assembly.mjs';

function inside(x,y,polygon){
 let yes=false;
 for(let i=0,j=polygon.length-1;i<polygon.length;j=i++){
  const a=polygon[i],b=polygon[j];
  if((a[1]>y)!==(b[1]>y)&&x<(b[0]-a[0])*(y-a[1])/(b[1]-a[1])+a[0])yes=!yes;
 }
 return yes;
}
export function coverageContract(body,identity){
 const [dx,dy]=identity.shift,[cx,cy]=identity.source_collar;
 // Native female neck attachment, including the pixels removed from the moving head.
 const polygon=[[55+dx,61+dy],[67+dx,65+dy],[cx+3,cy+2],[cx-3,cy+2]];
 const neck=empty();
 for(let y=0;y<H;y++)for(let x=0;x<W;x++)if(inside(x+.5,y+.5,polygon))neck[(y*W+x)*4+3]=255;
 const fabric=shirtMask(body),cloth=Buffer.from(fabric),seen=new Set();
 // Restore complete shirt clusters, including internal folds and their 1px contour.
 for(let k=0;k<W*H;k++){
  if(seen.has(k)||!fabric[k*4+3])continue;
  const q=[k];seen.add(k);
  for(let n=0;n<q.length;n++)for(let yy=-1;yy<=1;yy++)for(let xx=-1;xx<=1;xx++){
   const x=q[n]%W+xx,y=Math.floor(q[n]/W)+yy,j=y*W+x;
   if(x<0||x>=W||y<0||y>=H||seen.has(j)||!fabric[j*4+3])continue;
   seen.add(j);q.push(j);
  }
  const rows=new Map();
  for(const j of q){const y=Math.floor(j/W),x=j%W,r=rows.get(y)||[W,0];r[0]=Math.min(r[0],x);r[1]=Math.max(r[1],x);rows.set(y,r);}
  for(const [y,[lo,hi]]of rows)for(let x=lo;x<=hi;x++)cloth[(y*W+x)*4+3]=255;
 }
 const growCloth=grow(cloth,1);
 const garment=select(body,(x,y)=>growCloth[(y*W+x)*4+3]&&(!neck[(y*W+x)*4+3]||fabric[(y*W+x)*4+3]));
 return {neck,garment,fabric,polygon,required:over(neck,garment)};
}

export function completeUpper(body,upper,head,identity){
 const contract=coverageContract(body,identity),addedGarment=empty(),addedSkin=empty();
 const covered=over(upper,contract.garment),samples=[];
 for(let y=0;y<H;y++)for(let x=0;x<W;x++){
  const i=(y*W+x)*4;
  if(contract.neck[i+3]&&body[i+3]&&skin(body.subarray(i,i+4)))samples.push([x,y,i]);
  if(contract.garment[i+3]&&!upper[i+3])body.copy(addedGarment,i,i,i+4);
 }
 if(!samples.length)throw Error('No source neck pigment for the declared underlap');
 for(let y=0;y<H;y++)for(let x=0;x<W;x++){
  const i=(y*W+x)*4;if(!contract.neck[i+3]||covered[i+3])continue;
  let sample=samples[0],distance=Infinity;
  for(const candidate of samples){const d=(candidate[0]-x)**2+(candidate[1]-y)**2;if(d<distance){distance=d;sample=candidate;}}
  body.copy(addedSkin,i,sample[2],sample[2]+4);
 }
 const silhouette=over(upper,addedGarment,addedSkin,head);
 let contour=null,shade=Infinity;
 for(let i=0;i<head.length;i+=4)if(head[i+3]){const d=head[i]+head[i+1]+head[i+2];if(d<shade){shade=d;contour=head.subarray(i,i+4);}}
 for(let y=0;y<H;y++)for(let x=1;x<W-1;x++){
  const i=(y*W+x)*4;
  if(addedSkin[i+3]&&!head[i+3]&&(!silhouette[i-1]||!silhouette[i+7]))contour.copy(addedSkin,i);
 }
 const addedClosure=empty(),joined=over(upper,addedGarment,addedSkin,head),visited=new Uint8Array(W*H);
 const garmentSamples=[];
 for(let k=0;k<W*H;k++)if(contract.garment[k*4+3])garmentSamples.push([k%W,Math.floor(k/W),k*4]);
 for(let k=0;k<W*H;k++){
  if(visited[k]||joined[k*4+3])continue;
  const q=[k];visited[k]=1;let outside=false;
  for(let n=0;n<q.length;n++){
   const x=q[n]%W,y=Math.floor(q[n]/W);outside||=x===0||x===W-1||y===0||y===H-1;
   for(const [xx,yy]of [[x-1,y],[x+1,y],[x,y-1],[x,y+1]]){
    const j=yy*W+xx;if(xx<0||xx>=W||yy<0||yy>=H||visited[j]||joined[j*4+3])continue;
    visited[j]=1;q.push(j);
   }
  }
  const [dx,dy]=identity.shift;
  const hairCutouts=new Set(['44,57','44,58','45,59','45,60']);
  const intentionalHair=q.every(j=>hairCutouts.has((j%W-dx)+','+(Math.floor(j/W)-dy)));
  const seamHole=!outside&&!intentionalHair&&q.every(j=>body[j*4+3]&&j%W>=40+dx&&j%W<80+dx&&Math.floor(j/W)>=56+dy&&Math.floor(j/W)<=identity.source_collar[1]+8);
  if(!seamHole)continue;
  for(const j of q){
   const x=j%W,y=Math.floor(j/W);let nearest=garmentSamples[0],distance=Infinity;
   for(const sample of garmentSamples){const d=(x-sample[0])**2+(y-sample[1])**2;if(d<distance){distance=d;nearest=sample;}}
   body.copy(addedClosure,j*4,nearest[2],nearest[2]+4);contract.required[j*4+3]=255;
  }
 }
 const completed=over(upper,addedGarment,addedSkin,addedClosure);
 for(let i=0;i<upper.length;i+=4){
  if(upper[i+3]&&!upper.subarray(i,i+4).equals(completed.subarray(i,i+4)))throw Error('Underlap repainted an existing body pixel');
  if(contract.required[i+3]&&!completed[i+3])throw Error('Incomplete declared body coverage');
 }
 return {upper:completed,addedGarment,addedSkin,addedClosure,required:contract.required,polygon:contract.polygon};
}
