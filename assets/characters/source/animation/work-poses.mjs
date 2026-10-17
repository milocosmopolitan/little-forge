import path from 'node:path';
import {W,H,empty,tile,register,select,over,remap,skin,cloth,grow,inside,move} from './raster.mjs';

// A shared sampling interval and foot anchor register the coherent new sources.
// The old bench builder mixed three independently generated heads and bodies.
export function benchPoses(source,sex,colors){
 const baseFile=path.join(source,'work-repair',sex+'-bench-front-back.png');
 const gearFile=path.join(source,'work-repair',sex+'-bench-equipped.png');
 function sheet(file){
  const step=register(tile(file,4,2,0),{height:117}).registration.sample_step;
  return Array.from({length:8},(_,k)=>{
   const t=tile(file,4,2,k),r=register(t,{height:117}).registration;
   const cx=r.sample_origin[0]+64*r.sample_step;
   return remap(register(t,{locked:{sample_step:step,sample_origin:[cx-64*step,r.bbox[3]-141*step]}}).image,colors);
  });
 }
 const bases=sheet(baseFile),equipped=sheet(gearFile);
 const bt=tile(path.join(source,'work-repair/front-heads-blank.png'),2,1,sex==='female'?0:1),blank=empty();
 for(let y=20;y<72;y++)for(let x=32;x<96;x++){
  const sx=Math.floor((x-32+.5)*bt.w/64),sy=Math.floor((y-20+.5)*bt.h/52),i=(sy*bt.w+sx)*4;
  if(bt.b[i+3])bt.b.copy(blank,(y*W+x)*4,i,i+4);
 }
 const skinBlank=remap(blank,colors),faces={};
 for(const direction of ['front','back']){
  const full=select(bases[direction==='front'?0:4],(x,y)=>y<65),head=Buffer.from(full),eyes=empty(),mouth=empty();
  if(direction==='front')for(let y=42;y<62;y++)for(let x=52;x<77;x++){
   const i=(y*W+x)*4,p=full.subarray(i,i+4),q=skinBlank.subarray(i,i+4);
   const eye=inside(x,y,[53,43,62,58])||inside(x,y,[67,43,75,58]),lip=inside(x,y,[60,57,69,61]);
   const pigment=p[0]<185&&p[1]<145&&p[2]<125||p[0]>145&&Math.abs(p[0]-p[1])<40&&Math.abs(p[1]-p[2])<40;
   if((eye||lip)&&p[3]&&skin(q)&&pigment){p.copy(eye?eyes:mouth,i);q.copy(head,i);}
  }
  faces[direction]={head,eyes,mouth};
 }
 const blinkTile=tile(path.join(source,'work-repair/front-heads-blink.png'),2,1,sex==='female'?0:1),blinkRaw=empty();
 for(let y=20;y<72;y++)for(let x=32;x<96;x++){
  const sx=Math.floor((x-32+.5)*blinkTile.w/64),sy=Math.floor((y-20+.5)*blinkTile.h/52),i=(sy*blinkTile.w+sx)*4;
  if(blinkTile.b[i+3])blinkTile.b.copy(blinkRaw,(y*W+x)*4,i,i+4);
 }
 const closed=over(select(faces.front.eyes,(x,y)=>y<48),select(remap(blinkRaw,colors),(x,y,p)=>y>=48&&y<58&&(x>=53&&x<62||x>=67&&x<75)&&skin(faces.front.head.subarray((y*W+x)*4,(y*W+x)*4+4))&&p[0]<145&&p[1]<115&&p[2]<95));
 return bases.map((raw,k)=>{
  const front=k<4,direction=front?'front':'back',ref=bases[front?0:4];
  // Hands are the only pixels redrawn over a desk. Face skin is never an arm seed.
  const think=front&&k===3,thinkRect=sex==='female'?[69,60,80,81]:[55,60,66,81];
  const armSkin=select(raw,(x,y,p)=>skin(p)&&(y>=75&&y<93||think&&inside(x,y,thinkRect)));
  const armMask=grow(armSkin,1),arms=select(raw,(x,y)=>!!armMask[(y*W+x)*4+3]&&(y>=65||think&&inside(x,y,thinkRect)));
  const upper=over(select(raw,(x,y)=>y>=65&&y<90),select(ref,(x,y)=>inside(x,y,[58,62,70,71])));
  const lower=select(ref,(x,y)=>y>=90);
  // Only equipment material at measured apron/strap sites is taken from the edit.
  const eq=equipped[k],gear=select(eq,(x,y,p)=>{
   if(front){
    const apron=inside(x,y,sex==='female'?[47,70,83,115]:[43,70,87,117]);
    const straps=inside(x,y,[51,64,57,77])||inside(x,y,[71,64,77,77]);
    const towel=sex==='male'&&inside(x,y,[74,89,86,113])&&cloth(p);
    return (apron||straps)&&!skin(p)&&!cloth(p)&&(p[0]>p[1]*1.15||p[0]<65||p[1]>110&&p[0]>p[1]*1.05)||towel;
   }
   const back=inside(x,y,[46,65,85,114]);
   return back&&!skin(p)&&(!cloth(p)&&(p[0]>p[1]*1.15||p[0]<65)||sex==='male'&&x<55&&y>90&&cloth(p));
  });
  const headgear=sex==='male'?select(eq,(x,y,p)=>y>=35&&y<55&&(!front||x<55||x>74)&&p[0]<110&&Math.abs(p[0]-p[1])<24&&Math.abs(p[1]-p[2])<24):empty();
  const parts={...faces[direction],upper,lower};
  return {parts,closed,front:arms,gear:{'equipment/work-apron.png':gear,'equipment/gloves.png':empty(),...(sex==='male'?{'equipment/headband.png':headgear}:{})},facing:direction,source:'source/animation/work-repair/'+sex+'-bench-front-back.png'};
 });
}
