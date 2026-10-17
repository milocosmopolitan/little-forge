// A single registered source owns facial geometry; the blank edit supplies skin
// only beneath removable features, so unrelated imagegen drift is never adopted.
export function restoreFace({id,file,sample,remap,colors,old,upper}) {
 // Female source grid is offset by ~2 high-resolution pixels; one common
 // phase for the entire head preserves the lifted 1px smile corner.
 const phaseX=id==='female'?0.125:0;
 const transform=(x,y,w,h)=>[(x-8+phaseX)*w/80,y*h/68];
 const full=remap(sample(file('source/revision-3/generated-full.png'),transform),colors);
 const blank=remap(sample(file('source/revision-3/generated-blank.png'),transform),colors);
 const head=Buffer.from(old),eyes=Buffer.alloc(old.length),mouth=Buffer.alloc(old.length),mask=Buffer.alloc(old.length);
 const rect=(x,y,b)=>x>=b[0]&&x<b[2]&&y>=b[1]&&y<b[3];
 const skin=p=>p[0]>185&&p[1]>105&&p[2]>55&&p[0]>p[1]*1.22;
 const eyeBoxes=id==='female'?[[41,29,53,41],[55,27,63,41]]:[[39,27,51,41],[52,29,60,41]];
 const mouthBox=id==='female'?[49,41,57,46]:[47,40,56,45];
 for(let y=0;y<49;y++)for(let x=0;x<96;x++){
  const i=(y*96+x)*4;
  if(old[i+3]&&full[i+3])full.copy(head,i,i,i+4);
  // The jaw/neck junction may change alpha locally; hair/crown size stays fixed.
  if(id==='male'&&rect(x,y,[37,43,61,49]))full.copy(head,i,i,i+4);
  if(!head[i+3])continue;
  const p=full.subarray(i,i+4),q=blank.subarray(i,i+4);
  const dark=p[0]<220&&p[1]<150&&p[2]<125;
  const white=p[0]>145&&p[0]-p[1]<45&&p[1]-p[2]<45;
  const eye=eyeBoxes.some(b=>rect(x,y,b))&&skin(q)&&(dark||white);
  const lip=rect(x,y,mouthBox)&&skin(q)&&dark;
  if(eye||lip){full.copy(eye?eyes:mouth,i,i,i+4);blank.copy(head,i,i,i+4);}
 }
 // Only exposed neck skin changes in the approved body. Shirt and shoulder
 // pixels retain their authored values and alpha.
 if(id==='male')for(let y=49;y<56;y++)for(let x=39;x<57;x++){
  const i=(y*96+x)*4;if(skin(upper.subarray(i,i+4))&&skin(full.subarray(i,i+4)))full.copy(upper,i,i,i+4);
 }
 for(let i=0;i<old.length;i+=4)if(!head.subarray(i,i+4).equals(old.subarray(i,i+4))) {mask[i]=255;mask[i+3]=255;}
 return {head,eyes,mouth,mask,full,eyeBoxes,mouthBox,phaseX};
}
