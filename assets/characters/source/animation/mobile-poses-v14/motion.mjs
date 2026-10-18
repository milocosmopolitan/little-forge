import fs from 'node:fs';import path from 'node:path';
import {W,H,empty,read,save,over,move,select,grow,bounds,skin,cloth} from '../raster.mjs';
import {faceRegion} from '../hip-pivot/coherent.mjs';
import {sourceCollar,collarHead} from '../collar-v15/collar.mjs';

function waist(body){
 for(let y=78;y<107;y++){
  const xs=[];for(let x=24;x<105;x++){const i=(y*W+x)*4,p=body.subarray(i,i+4);if(p[3]&&p[0]<145&&p[1]>55&&p[1]>p[2]*1.18&&p[0]<p[1]*1.28)xs.push(x);}
  if(xs.length>=7)return [Math.round(xs.reduce((s,x)=>s+x,0)/xs.length),y];
 }
 throw Error('No trouser waist');
}
function shirtAxis(body,point){
 const xs=[];for(let y=point[1]+4;y<=point[1]+8;y++)for(let x=35;x<95;x++){const i=(y*W+x)*4;if(body[i+3]&&cloth(body.subarray(i,i+4)))xs.push(x);}
 return xs.length?Math.round((Math.min(...xs)+Math.max(...xs))/2):point[0];
}
function retarget(gear,fromNeck,fromWaist,toNeck,toWaist){
 const out=empty();
 // Inverse native row mapping keeps the complete strap/bib silhouette closed.
 for(let y=0;y<H;y++){
  const t=Math.max(0,Math.min(1,(y-toNeck[1])/(toWaist[1]-toNeck[1])));
  const sy=y>=toWaist[1]?y-toWaist[1]+fromWaist[1]:Math.round(fromNeck[1]+(y-toNeck[1])*(fromWaist[1]-fromNeck[1])/(toWaist[1]-toNeck[1]));
  if(sy<0||sy>=H)continue;
  const dx=Math.round((toNeck[0]-fromNeck[0])*(1-t)+(toWaist[0]-fromWaist[0])*t);
  for(let x=0;x<W;x++){const sx=x-dx;if(sx<0||sx>=W)continue;const i=(y*W+x)*4,j=(sy*W+sx)*4;if(gear[j+3])gear.copy(out,i,j,j+4);}
 }
 return out;
}
function armsAndGloves(body,point,gloveSource,isGive,armStart){
 const exposed=select(body,(x,y,p)=>y>=armStart&&y<117&&skin(p));
 const seen=new Set(),groups=[];
 for(let k=0;k<W*H;k++){
  if(seen.has(k)||!exposed[k*4+3])continue;const q=[k];seen.add(k);
  for(let n=0;n<q.length;n++)for(const[dx,dy]of [[1,0],[-1,0],[0,1],[0,-1],[-1,-1],[1,-1],[-1,1],[1,1]]){
   const x=q[n]%W+dx,y=Math.floor(q[n]/W)+dy,j=y*W+x;if(x<0||x>=W||y<0||y>=H||seen.has(j)||!exposed[j*4+3])continue;seen.add(j);q.push(j);
  }
  groups.push(q);
 }
 groups.sort((a,b)=>b.length-a.length);const chosen=groups.slice(0,2),mask=empty(),gloves=empty(),hands=[];
 for(const extra of groups.slice(2)){
  let best=null,distance=Infinity;for(const arm of chosen)for(const p of extra)for(const q of arm){const d=(p%W-q%W)**2+(Math.floor(p/W)-Math.floor(q/W))**2;if(d<distance){distance=d;best=arm;}}
  if(distance<=9)best.push(...extra);
 }
 const swatches=new Map();for(let i=0;i<gloveSource.length;i+=4)if(gloveSource[i+3]){const p=gloveSource.subarray(i,i+4),key=p.toString('hex');swatches.set(key,{p,n:(swatches.get(key)?.n||0)+1});}
 const pigments=[...swatches.values()].filter(v=>v.p[0]>45&&v.p[0]>v.p[1]*1.2).sort((a,b)=>b.n-a.n).slice(0,8).map(v=>v.p).sort((a,b)=>a[0]+a[1]+a[2]-b[0]-b[1]-b[2]);
 const tones=[pigments[0],pigments[Math.floor(pigments.length/2)],pigments.at(-1)];
 for(const q of chosen){
  for(const j of q)mask[j*4+3]=255;
  const xs=q.map(j=>j%W),ys=q.map(j=>Math.floor(j/W)),loX=Math.min(...xs),hiX=Math.max(...xs),loY=Math.min(...ys),hiY=Math.max(...ys),horizontal=hiX-loX>hiY-loY;
  const cuff=q.filter(j=>horizontal?j%W<=loX+1:Math.floor(j/W)<=loY+1),cx=cuff.reduce((s,j)=>s+j%W,0)/cuff.length,cy=cuff.reduce((s,j)=>s+Math.floor(j/W),0)/cuff.length;
  const tips=q.filter(j=>horizontal?j%W===hiX:Math.floor(j/W)===hiY),tip=tips[Math.floor(tips.length/2)],tx=tip%W,ty=Math.floor(tip/W);
  const reach=Math.min(16,Math.max(7,Math.hypot(tx-cx,ty-cy)*.47));
  const hand=q.filter(j=>Math.hypot(j%W-tx,Math.floor(j/W)-ty)<=reach);
  for(const j of hand){const i=j*4,p=body.subarray(i,i+4),tone=p[1]<155?tones[0]:p[1]<185?tones[1]:tones[2];tone.copy(gloves,i);}
  hands.push({side:cx<point[0]?'anatomical_right':'anatomical_left',horizontal,point:[tx,ty],cuff:[Math.round(cx),Math.round(cy)],skin_pixels:q.length,glove_pixels:hand.length,hand_pixels:hand.map(j=>[j%W,Math.floor(j/W)])});
 }
 if(hands.length===2){const near=isGive?(hands[0].horizontal?0:1):(hands[0].cuff[0]<=hands[1].cuff[0]?0:1);hands.forEach((h,i)=>h.side=i===near?'anatomical_right':'anatomical_left');}
 const frontArea=grow(mask,1),front=select(body,(x,y)=>!!frontArea[(y*W+x)*4+3]);
 return {front,gloves,hands};
}
export function motionFactory({dir,sex,source,colors}){
 const root={upper:read(path.join(dir,'body/upper-base.png')),lower:read(path.join(dir,'body/lower-base.png'))},recipe=JSON.parse(fs.readFileSync(path.join(dir,'character.json')));
 const gear=Object.fromEntries(recipe.equipped.filter(n=>n.startsWith('equipment/')).map(n=>[n,read(path.join(dir,n))]));
 const neck=sex==='female'?[62,66]:[57,64],rootWaist=waist(over(root.lower,root.upper));
 return (input,key,sourcePath)=>{
  const isGive=key.startsWith('give-')||key.startsWith('gestures-'),sourceWaist=waist(input),delta=[rootWaist[0]-sourceWaist[0],isGive?rootWaist[1]-sourceWaist[1]:0];
  let raw=move(input,...delta);
  if(isGive){
   const armArea=grow(select(raw,(x,y,p)=>y>=75&&y<112&&skin(p)),1);
   raw=over(root.lower,select(raw,(x,y)=>y<rootWaist[1]||!!armArea[(y*W+x)*4+3]));
  }
  const face=faceRegion(raw).box,collar=sourceCollar(raw,sex,key),point=collar.point;
  const restore=equipped=>({...collarHead(dir,sex,collar.shift,equipped),identity:{source:'approved root head and features',shift:collar.shift,root_pixels_unchanged:true,attachment_policy:'Source jaw exit traced above the original shirt opening; no fabricated neck pixels',anatomical_head_adjustment:[0,0],head_translation_adjustment:[0,0],root_collar:neck,source_collar:point}});
  const identity=restore(false),armStart=collar.joint.collar_end+2;
  const armArea=grow(select(raw,(x,y,p)=>y>=armStart&&y<117&&skin(p)),1);
  const lower=select(collar.upper,(x,y)=>y>=94&&!armArea[(y*W+x)*4+3]);
  const body={upper:select(collar.upper,(x,y)=>y<94||!!armArea[(y*W+x)*4+3]),required:collar.required,neck:collar.neck};
  const bodyImage=over(lower,body.upper),targetWaist=isGive?rootWaist:waist(bodyImage),gearNeck=[shirtAxis(raw,point),point[1]],rootGearNeck=[shirtAxis(over(root.lower,root.upper),neck),neck[1]];
  const movedGear={};for(const[n,b]of Object.entries(gear))if(!/gloves|hair-tie|headband/.test(n)){
   const adapted=retarget(b,rootGearNeck,rootWaist,gearNeck,targetWaist);
   movedGear[n]=n.endsWith('/apron.png')?select(adapted,(x,y)=>(y>=targetWaist[1]||!!bodyImage[(y*W+x)*4+3])&&(y>point[1]+8||cloth(body.upper.subarray((y*W+x)*4,(y*W+x)*4+4)))):adapted;
  }
  const {front,gloves,hands}=armsAndGloves(bodyImage,point,gear['equipment/gloves.png'],isGive,armStart);
  const equippedIdentity=restore(true);
  const equippedBody=over(bodyImage,...Object.values(movedGear),front,gloves),base=over(bodyImage,identity.head,identity.eyes,identity.mouth),composed=over(equippedBody,equippedIdentity.head,equippedIdentity.eyes,equippedIdentity.mouth);
  const layers={'body/lower-base.png':lower,'body/upper-base.png':body.upper,'head-base.png':identity.head,'eyes/open.png':identity.eyes,'mouth/rest.png':identity.mouth,...movedGear,'body/arms-front.png':front,'equipment/gloves.png':gloves,'props/held.png':empty(),'rendered/body/lower.png':lower,'rendered/body/upper.png':select(equippedBody,(x,y)=>!lower[(y*W+x)*4+3]||y<117),'rendered/head-base.png':equippedIdentity.head,'rendered/eyes/open.png':equippedIdentity.eyes,'rendered/mouth/rest.png':equippedIdentity.mouth,'source/complete-pose.png':raw,'body/required-coverage.png':body.required,'body/neck-skin.png':body.neck};
  // Arms remain in the upper layer even when a hand reaches below the waist.
  const upperMask=(x,y)=>y<94||!!armArea[(y*W+x)*4+3]||!!gloves[(y*W+x)*4+3];
  layers['rendered/body/upper.png']=select(equippedBody,upperMask);layers['rendered/body/lower.png']=select(equippedBody,(x,y)=>!upperMask(x,y));
  layers['foundation.png']=over(lower,body.upper,identity.head);layers['rendered/foundation.png']=over(equippedBody,equippedIdentity.head);layers['base.png']=base;layers['composed.png']=composed;
  const p='poses/'+key,baseOrder=['body/lower-base.png','body/upper-base.png','head-base.png','eyes/open.png','mouth/rest.png'],eqOrder=['rendered/body/lower.png','rendered/body/upper.png','rendered/head-base.png','rendered/eyes/open.png','rendered/mouth/rest.png'];
  return {layers,base,composed,pose:{canvas:[W,H],pivot:[64,140],facing:'right_3_4',source:sourcePath,layer_finish:'collar-v15',layers:baseOrder.map(n=>p+'/'+n),rendered_variant_layers:eqOrder.map(n=>p+'/'+n),base:'frames/base/'+key+'.png',composed:'frames/'+key+'.png',sockets:{neck:point,waist:targetWaist,hands,foot:[64,140]},motion_head_identity:identity.identity,motion_equipped_head_identity:equippedIdentity.identity,motion_assembly:{revision:'collar-v15',source_collar:collar.joint,source_face:face,source_registration_delta:delta,neck:point,waist:targetWaist,root_neck:neck,root_waist:rootWaist,hand_contours:'Source skin silhouettes recolored into existing leather palette; no fitted glove image',gear_registration:'Native inverse row mapping from neck and waist anchors',body_owned_neck:true},source_partition_roundtrip:false,source_contract:'Approved head replaces generated head; give retains approved neutral legs',rendered_equipment_baked:true,pose_layers_interchangeable:false}};
 };
}
export function writeMotion(dir,key,result){
 const p=path.join(dir,'poses',key);fs.rmSync(p,{recursive:true,force:true});for(const[n,b]of Object.entries(result.layers))save(path.join(p,n),b);
 save(path.join(dir,'frames/base',key+'.png'),result.base);save(path.join(dir,'frames',key+'.png'),result.composed);return result.pose;
}
