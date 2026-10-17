import {W,empty,select,over,grow,skin,cloth} from '../raster.mjs';

// Reuse the complete generated resting hand, without resizing it or redrawing fingers.
export function finishResearchBody(assembly,raw,donor,sex,index,sourceLower,equipped){
 const upper=Buffer.from(assembly.upper),patch=Buffer.from(assembly.shoulder);
 const lower=Buffer.from(sourceLower),garment=Buffer.from(assembly.garment),removedProp=empty();
 if(sex==='male'&&equipped){
  // User requested removal of the misplaced left hanging cloth in every research key.
  const tail=grow(select(raw,(x,y,p)=>x>=38&&x<54&&y>=93&&y<110&&cloth(p)),1);
  for(let y=93;y<110;y++)for(let x=38;x<54;x++){
   const i=(y*W+x)*4;if(!tail[i+3]||!raw[i+3])continue;
   raw.copy(removedProp,i,i,i+4);upper.fill(0,i,i+4);lower.fill(0,i,i+4);garment.fill(0,i,i+4);
  }
  const combined=over(lower,upper),seen=new Set();
  for(let k=0;k<W*160;k++){
   if(seen.has(k)||!combined[k*4+3])continue;
   const q=[k];seen.add(k);
   for(let n=0;n<q.length;n++)for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
    const x=q[n]%W+dx,y=Math.floor(q[n]/W)+dy,j=y*W+x;
    if(x<0||x>=W||y<0||y>=160||seen.has(j)||!combined[j*4+3])continue;seen.add(j);q.push(j);
   }
   // Removing the cloth must also remove its detached dark outline remnants.
   if(q.length<=12&&q.every(j=>j%W>=38&&j%W<54&&Math.floor(j/W)>=93&&Math.floor(j/W)<110))for(const j of q){
    raw.copy(removedProp,j*4,j*4,j*4+4);upper.fill(0,j*4,j*4+4);lower.fill(0,j*4,j*4+4);garment.fill(0,j*4,j*4+4);
   }
  }
 }
 if(sex==='female'){
  const samples=[];
  for(let y=58;y<82;y++)for(let x=38;x<90;x++){
   const i=(y*W+x)*4,p=raw.subarray(i,i+4);
   if(p[3]&&cloth(p)&&p[0]>=220&&p[1]>=200)samples.push([x,y,i]);
  }
  // The old inferred plate added black horizontal ledges and unrelated dark pigment.
  // Extend the adjacent shirt pigment only in that synthesized plate.
  for(let y=0;y<160;y++)for(let x=0;x<W;x++){
   const i=(y*W+x)*4;if(!patch[i+3])continue;
   const nearest=samples.reduce((a,b)=>(b[0]-x)**2+(b[1]-y)**2<(a[0]-x)**2+(a[1]-y)**2?b:a);
   raw.copy(patch,i,nearest[2],nearest[2]+4);patch.copy(upper,i,i,i+4);
  }
  if(index===3){
   const hand=b=>grow(select(b,(x,y,p)=>x>=40&&x<64&&y>=81&&y<97&&skin(p)),1);
   const old=hand(raw),replacement=hand(donor);
   for(let y=81;y<98;y++)for(let x=39;x<65;x++){
    const i=(y*W+x)*4;
    if(assembly.garment[i+3]&&(y<85||cloth(raw.subarray(i,i+4))))continue;
    if(old[i+3]&&!replacement[i+3]){upper.fill(0,i,i+4);garment.fill(0,i,i+4);}
    if(replacement[i+3]&&donor[i+3]){donor.copy(upper,i,i,i+4);garment.fill(0,i,i+4);}
   }
  }
 }
 return {...assembly,upper,lower,garment,removedProp,shoulder:patch};
}

export function researchFront(upper,lower,mount){
 const body=over(lower,upper);
 // Only skin-connected forearms/hands and their sleeves can cover the desktop.
 // Each equipment variant gets its own mask. Shirt tails and towels stay behind it.
 const hands=select(body,(x,y,p)=>y>=79&&y<99&&skin(p));
 const seen=new Set();
 for(let k=0;k<W*160;k++){
  if(seen.has(k)||!hands[k*4+3])continue;
  const q=[k];seen.add(k);
  for(let n=0;n<q.length;n++)for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
   const x=q[n]%W+dx,y=Math.floor(q[n]/W)+dy,j=y*W+x;
   if(x<0||x>=W||y<0||y>=160||seen.has(j)||!hands[j*4+3])continue;seen.add(j);q.push(j);
  }
  // Copper fittings can share skin pigment; they are not fingers.
  if(q.length<12)for(const j of q)hands.fill(0,j*4,j*4+4);
 }
 const handArea=grow(hands,1);
 return select(body,(x,y)=>!!handArea[(y*W+x)*4+3]||
  (y>=mount[1]+3&&y<83&&(x<53||x>=76)));
}
