import {W,empty,over,skin,grow} from '../raster.mjs';
import {faceRegion} from '../hip-pivot/coherent.mjs';

// Native source strap columns, including their dark seam, leather and buckle pixels.
// These are garment ownership guides, not a brightness classifier or head mask.
export const straps={
 0:[[55,59,66,70,78],[69,73,66,70,78]],
 1:[[55,59,66,71,79],[70,74,66,71,79]],
 2:[[55,59,71,75,82],[71,75,71,75,82]],
 3:[[55,59,64,69,78]],
};
export function restoreResearchMaterials(assembly,raw,index,equipped,rootHead){
 const upper=Buffer.from(assembly.upper),restored=empty(),collar=empty(),removedSeam=empty();
 const required=Buffer.from(assembly.required);
 const neck=Buffer.from(assembly.neck),garment=Buffer.from(assembly.garment);
 const [cx,cy]=assembly.point;
 if(equipped){
  const oldFaceEdge=grow(faceRegion(raw).mask,1);
  const pigment=(x,y)=>rootHead.subarray((y*W+x)*4,(y*W+x)*4+4);
  for(let y=cy-3;y<=cy+2;y++)for(let x=cx-4;x<=cx+3;x++){
   const i=(y*W+x)*4;
   if(!skin(upper.subarray(i,i+4))||index===3&&x>=cx)continue;
   // Match the approved neck's skin and under-jaw shade, without importing source-face bands.
   const p=pigment(x<cx-2?63:64,63);p.copy(neck,i);p.copy(upper,i);garment.fill(0,i,i+4);
  }
  const collarY=[73,75,78,73][index],white=raw.subarray((collarY*W+60)*4,(collarY*W+60)*4+4);
  if(!(white[0]>165&&white[1]>145&&white[2]>110))throw Error('Missing measured collar pigment');
  const left=59,right=index===2?71: index===1?70:69,raise=[2,3,2,3][index];
  // Reposition the source collar around the short neck, rather than lengthening skin
  // down to the old, differently shaped source face. The torso and head stay fixed.
  for(let y=collarY-3-raise;y<=collarY;y++)for(let x=left;x<right;x++){
   if(index===3&&x>=cx)continue;
   const i=(y*W+x)*4;white.copy(collar,i);white.copy(upper,i);garment.fill(0,i,i+4);neck.fill(0,i,i+4);
  }
  for(let sy=collarY-3;sy<=collarY;sy++)for(let x=left;x<right;x++){
   if(index===3&&x>=cx)continue;
   const i=((sy-raise)*W+x)*4,j=(sy*W+x)*4,p=raw.subarray(j,j+4);
   const q=skin(p)?pigment(x<cx-2?63:64,63):oldFaceEdge[j+3]&&p[0]+p[1]+p[2]<350?white:p;
   q.copy(collar,i);q.copy(upper,i);if(skin(q))q.copy(neck,i);
  }
  if(index===3)for(const[y,lo,hi]of [[66,51,55],[67,52,54]])for(let x=lo;x<hi;x++){
   const i=(y*W+x)*4;upper.copy(removedSeam,i,i,i+4);upper.fill(0,i,i+4);garment.fill(0,i,i+4);required.fill(0,i,i+4);
  }
 }
 if(equipped)for(const[x0,x1,top,sourceY,bottom]of straps[index]){
  for(let y=top;y<bottom;y++)for(let x=x0;x<x1;x++){
   const i=(y*W+x)*4,j=(Math.max(y,sourceY)*W+x)*4;
   if(!raw[j+3])throw Error('Missing source leather strap');
   raw.copy(restored,i,j,j+4);
  }
 }
 for(let i=0;i<restored.length;i+=4)if(restored[i+3]){garment.fill(0,i,i+4);neck.fill(0,i,i+4);}
 return {...assembly,upper:over(upper,restored),neck,garment,required,restored,collar,removedSeam};
}
