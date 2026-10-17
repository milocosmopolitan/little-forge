import {W,H,empty,select,skin,grow,bounds} from '../raster.mjs';

export function fabricMask(body){
 return select(body,(x,y,p)=>y>=46&&y<112&&p[0]>170&&p[1]>150&&p[2]>125&&p[0]-p[1]<32&&p[1]-p[2]<38);
}
export function collarSocket(body,face,facing='right_3_4'){
 const fabric=fabricMask(body),near=grow(fabric,2),cx=(face[0]+face[2])/2,points=[];
 for(let y=Math.max(52,Math.floor(face[1]+(face[3]-face[1])*.5));y<Math.min(88,face[3]+12);y++)for(let x=Math.max(40,face[0]-3);x<Math.min(100,facing==='front'?cx+7:cx+1);x++){
  const i=(y*W+x)*4;
  if(body[i+3]&&near[i+3]&&skin(body.subarray(i,i+4)))points.push([x,y]);
 }
 if(points.length<3)throw Error('No measured collar skin '+JSON.stringify({face,facing,points}));
 const mean=n=>Math.round(points.reduce((s,p)=>s+p[n],0)/points.length);
 return {point:[mean(0),mean(1)],skin_points:points,fabric};
}
export function protectedConnection(body,face,facing,measurement){
 const garment=grow(measurement.fabric,2),[cx,cy]=measurement.point;
 const stem=select(body,(x,y,p)=>skin(p)&&x>=cx-6&&x<=cx+6&&y>=cy-7&&y<=cy+4);
 const neck=grow(stem,1);
 return select(body,(x,y)=>!!garment[(y*W+x)*4+3]||!!neck[(y*W+x)*4+3]);
}
