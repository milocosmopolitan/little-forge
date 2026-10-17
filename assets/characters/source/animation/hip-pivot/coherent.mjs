import {W,H,empty,tile,register,remap,move,bounds,skin} from '../raster.mjs';

export function registeredSheet(file,cols,rows,colors,{}={}){
 const whole=tile(file,1,1,0),seen=new Uint8Array(whole.w*whole.h),groups=[];
 for(let i=0;i<seen.length;i++){
  if(seen[i]||!whole.b[i*4+3])continue;
  const q=[i];seen[i]=1;
  for(let n=0;n<q.length;n++)for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
   const x=q[n]%whole.w+dx,y=Math.floor(q[n]/whole.w)+dy,j=y*whole.w+x;
   if(x<0||x>=whole.w||y<0||y>=whole.h||seen[j]||!whole.b[j*4+3])continue;
   seen[j]=1;q.push(j);
  }
  if(q.length>500)groups.push(q);
 }
 groups.sort((a,b)=>b.length-a.length);const chosen=groups.slice(0,cols*rows).map(q=>{
  const xs=q.map(i=>i%whole.w),ys=q.map(i=>Math.floor(i/whole.w)),box=[Math.min(...xs),Math.min(...ys),Math.max(...xs)+1,Math.max(...ys)+1];
  const w=box[2]-box[0],h=box[3]-box[1],b=Buffer.alloc(w*h*4);
  for(const i of q){const x=i%whole.w-box[0],y=Math.floor(i/whole.w)-box[1];whole.b.copy(b,(y*w+x)*4,i*4,i*4+4);}
  return {w,h,b,source_box:box};
 });
 chosen.sort((a,b)=>Math.floor((a.source_box[3]-1)/ (whole.h/rows))-Math.floor((b.source_box[3]-1)/(whole.h/rows))||a.source_box[0]-b.source_box[0]);
 if(chosen.length!==cols*rows)throw Error('Incomplete connected sprite count '+file);
 const first=chosen[0],step=Math.max(register(first,{height:117}).registration.sample_step,...chosen.map(t=>{const m=register(t,{height:117}).registration,c=m.sample_origin[0]+64*m.sample_step;return Math.max(c/62,(t.w-c)/62);}));
 return Array.from({length:cols*rows},(_,k)=>{
  const t=chosen[k],m=register(t,{height:117}).registration,cx=m.sample_origin[0]+64*m.sample_step;
  const lock={sample_step:step,sample_origin:[cx-64*step,m.bbox[3]-141*step]},r=register(t,{locked:lock});
  r.registration.source_component_box=t.source_box;
  const native=remap(r.image,colors),dy=141-bounds(native).box[3];
  if(dy){r.registration.sample_origin[1]-=dy*step;r.registration.native_foot_adjustment=dy;}
  return {image:dy?move(native,0,dy):native,registration:r.registration};
 });
}
export function faceRegion(b){
 const seen=new Set(),components=[];
 for(let y=30;y<80;y++)for(let x=40;x<102;x++){
  const k=y*W+x,i=k*4;if(seen.has(k)||!b[i+3]||!skin(b.subarray(i,i+4)))continue;
  const q=[k];seen.add(k);
  for(let n=0;n<q.length;n++)for(const v of [q[n]-W,q[n]+W,q[n]-1,q[n]+1]){
   const xx=v%W,yy=Math.floor(v/W),j=v*4;
   if(xx<40||xx>=102||yy<30||yy>=80||seen.has(v)||!b[j+3]||!skin(b.subarray(j,j+4)))continue;
   seen.add(v);q.push(v);
  }
  if(q.length>40)components.push(q);
 }
 components.sort((a,b)=>{
  const score=q=>{const xs=q.map(v=>v%W),ys=q.map(v=>Math.floor(v/W)),cx=xs.reduce((a,b)=>a+b)/q.length,cy=ys.reduce((a,b)=>a+b)/q.length;return q.length+(cx>=55&&cy<71&&Math.max(...xs)-Math.min(...xs)>=13&&Math.max(...ys)-Math.min(...ys)>=13?10000:0);};
  return score(b)-score(a);
 });
 const points=components[0];if(!points)throw Error('No connected face skin');
 const mask=empty();for(const p of points)mask[p*4+3]=255;
 return {mask,box:bounds(mask).box};
}
