import fs from 'node:fs';
import path from 'node:path';
import {W,H,empty,select,over,save,read,bounds,skin,cloth,grow} from './raster.mjs';
import {approvedWorkHeads,headJoint} from './style-restoration/heads.mjs';
import {bodyWithoutHead} from './joint-rebuild-v11/body.mjs';
import {finishResearchBody,researchFront} from './research-finish-v12/layers.mjs';
import {restoreResearchMaterials} from './research-material-v13/materials.mjs';
import {registeredSheet,faceRegion} from './hip-pivot/coherent.mjs';

const maps={hammering:[0,1,2,3,4,0],quenching:[0,1,2,2,3,0],forging:[4,5,6,6,7,4],researching:[0,1,2,1,3,0],salvaging:[4,5,6,5,7,4]};
function features(original,blank,back){
 if(back)return {eyes:empty(),mouth:empty(),face:null,body:blank};
 const face=faceRegion(blank),[x0,y0,x1,y1]=face.box,fh=y1-y0;
 const part=select(original,(x,y,p)=>{
  const i=(y*W+x)*4,q=blank.subarray(i,i+4);
  if(!face.mask[i+3]||!skin(q)||x<=x0||x>=x1-1||y<=y0+2||y>=y1-2)return false;
  return (y<y0+fh*.65||y>=y0+fh*.78)&&(p[0]<190&&p[1]<150&&p[2]<125||p[0]>145&&Math.abs(p[0]-p[1])<35&&Math.abs(p[1]-p[2])<35);
 });
 const body=blank;
 return {eyes:select(part,(x,y)=>y<y0+fh*.65),mouth:select(part,(x,y)=>y>=y0+fh*.78),face:face.box,body};
}
function partition(body,facing,face,key){
 let neck=face?.[3];
 if(!neck){
  for(let y=52;y<85;y++){let n=0;for(let x=54;x<76;x++){const i=(y*W+x)*4;if(body[i+3]&&cloth(body.subarray(i,i+4)))n++;}if(n>=8){neck=y;break;}}
 }
 neck??=70;
 const armSkin=select(body,(x,y,p)=>y>=75&&y<118&&!(face&&x>=face[0]&&x<face[2]&&y>=face[1]&&y<face[3])&&skin(p)),armArea=grow(armSkin,1);
 const head=select(body,(x,y)=>y<neck&&((face&&x>=face[0]&&x<face[2]&&y>=face[1]&&y<face[3])||!armArea[(y*W+x)*4+3]&&!(key==='hammering-1'&&x<(face?.[0]??56)-1&&y<61)));
 const upper=select(body,(x,y)=>!head[(y*W+x)*4+3]&&(y<91||armArea[(y*W+x)*4+3]));
 const lower=select(body,(x,y)=>!head[(y*W+x)*4+3]&&!upper[(y*W+x)*4+3]);
 if(!over(lower,upper,head).equals(body))throw Error(key+' changed coherent source while partitioning');
 return {head,upper,lower,neck};
}

// Workwear follows the drawn torso; approved facial layers follow each moving neck.
export function coherentWork({dir,sex,source,colors}){
 const src=path.join(source,'hip-pivot');
 if(!fs.existsSync(path.join(src,'ready.json')))throw Error('Coherent work source registration is required');
 const palette=[...colors,...['ff5a12','ffad22','ffe679','ad3020','636570','969a9c'].map(h=>[...Buffer.from(h,'hex')])];
 const groups={};
 for(const group of ['hammering','heat','bench']){
  const first=group==='hammering'?(sex==='female'?'female-hammering-base-candidate.png':'male-hammering-base.png'):sex+'-'+group+'-base.png';
  const cols=group==='hammering'?6:4,rows=group==='hammering'?1:2;
  groups[group]={original:registeredSheet(path.join(src,first),cols,rows,palette),blank:registeredSheet(path.join(src,sex+'-'+group+'-underlayer.png'),cols,rows,palette),equipped:registeredSheet(path.join(src,sex+'-'+group+'-equipment-blank.png'),cols,rows,palette)};
 }
 const restoreHead=approvedWorkHeads(dir,source,sex,palette);
 const approvedRootHead=read(path.join(dir,'head-base.png'));
 const poses={};
 for(const [action,indices]of Object.entries(maps))for(let k=0;k<6;k++){
  const id=action+'-'+k,group=groups[action==='hammering'?'hammering':['quenching','forging'].includes(action)?'heat':'bench'],index=indices[k];
  const original=group.original[index].image,blankSource=group.blank[index].image,eqSource=group.equipped[index].image;
  const facing=action==='researching'?'front':action==='salvaging'?'back':'right_3_4';
  const f=features(original,blankSource,facing==='back'),ef=features(original,eqSource,facing==='back');
  // A thinking hand can join facial skin in the clothed source; it is not a longer neck.
  const equippedFace=action==='researching'&&index===3?f.face:ef.face;
  const parts=partition(f.body,facing,f.face,id),ep=partition(ef.body,facing,equippedFace,id);
  const identity=restoreHead(parts.head,{facing,body:f.body,face:f.face,key:id}),equippedIdentity=restoreHead(ep.head,{facing,equipped:true,body:ef.body,face:equippedFace,key:id});
  const sourceSkinOnly=false;
  let baseAssembly=facing==='back'?null:bodyWithoutHead(f.body,parts.upper,f.face,identity.measurement,action==='researching'&&index===3);
  let equippedAssembly=facing==='back'?null:bodyWithoutHead(ef.body,ep.upper,equippedFace,equippedIdentity.measurement,action==='researching'&&index===3);
  if(action==='researching'){
   baseAssembly=finishResearchBody(baseAssembly,f.body,group.blank[0].image,sex,index,parts.lower,false);
   equippedAssembly=finishResearchBody(equippedAssembly,ef.body,group.equipped[0].image,sex,index,ep.lower,true);
   if(sex==='female')equippedAssembly=restoreResearchMaterials(equippedAssembly,ef.body,index,true,approvedRootHead);
   parts.lower=baseAssembly.lower;ep.lower=equippedAssembly.lower;
  }
  const garment=baseAssembly?.garment||empty(),renderedGarment=equippedAssembly?.garment||empty();
  const neckSkin=baseAssembly?.neck||empty(),renderedNeck=equippedAssembly?.neck||empty();
  if(baseAssembly)parts.upper=baseAssembly.upper;
  if(equippedAssembly)ep.upper=equippedAssembly.upper;
  parts.head=identity.head;ep.head=equippedIdentity.head;f.eyes=identity.eyes;f.mouth=identity.mouth;ef.eyes=equippedIdentity.eyes;ef.mouth=equippedIdentity.mouth;
  parts.neck=headJoint(parts.head)[1]+1;ep.neck=headJoint(ep.head)[1]+1;
  const blank=over(parts.lower,parts.upper,parts.head),eq=over(ep.lower,ep.upper,ep.head);
  const base=over(blank,f.eyes,f.mouth),full=over(eq,ef.eyes,ef.mouth),p='poses/'+id;
  const put=(name,b)=>save(path.join(dir,name),b);
  fs.rmSync(path.join(dir,p),{recursive:true,force:true});
  const layers={'head-base.png':parts.head,'eyes/open.png':f.eyes,'mouth/rest.png':f.mouth,'body/upper-base.png':parts.upper,'body/lower-base.png':parts.lower};
  const render={'rendered/head-base.png':ep.head,'rendered/eyes/open.png':ef.eyes,'rendered/mouth/rest.png':ef.mouth,'rendered/body/upper.png':ep.upper,'rendered/body/lower.png':ep.lower};
  for(const [prefix,assembly]of [['body/',baseAssembly],['rendered/body/',equippedAssembly]])if(assembly){
   layers[prefix+'shoulder-plate.png']=assembly.shoulder;
   layers[prefix+'required-coverage.png']=assembly.required;
   if(assembly.removedProp)layers[prefix+'removed-research-prop.png']=assembly.removedProp;
   if(assembly.restored){layers[prefix+'restored-straps.png']=assembly.restored;layers[prefix+'restored-collar.png']=assembly.collar;layers[prefix+'removed-seam.png']=assembly.removedSeam;}
  }
  const armArea=grow(select(parts.upper,(x,y,q)=>y>=70&&y<103&&!(y<84&&x>56&&x<73)&&skin(q)),2);
  const armOwner=(x,y)=>!!armArea[(y*W+x)*4+3]||(y>=parts.neck-2&&y<100&&(x<53||x>=76));
  const baseFront=action==='researching'?researchFront(parts.upper,parts.lower,baseAssembly.point):select(parts.upper,(x,y)=>armOwner(x,y)&&!parts.head[(y*W+x)*4+3]);
  const eqFront=action==='researching'?researchFront(ep.upper,ep.lower,equippedAssembly.point):select(ep.upper,(x,y)=>armOwner(x,y)&&!ep.head[(y*W+x)*4+3]);
  // This is an external-tool clip guide, not a replacement for the complete tool art.
  const toolClip=select(eq,(x,y)=>x>=85&&y>=65&&!blank[(y*W+x)*4+3]);
  for(const[name,b]of Object.entries({...layers,...render,'body/arms-front.png':eqFront,'body/arms-front-base.png':baseFront,'equipment/gloves.png':empty(),'props/held.png':toolClip,'foundation.png':blank,'base.png':base,'composed.png':full,'rendered/foundation.png':eq,'source/body-with-tools.png':original,'body/neck-skin.png':neckSkin,'body/source-connection.png':garment,'rendered/body/neck-skin.png':renderedNeck,'rendered/body/source-connection.png':renderedGarment,'body/removed-old-face.png':baseAssembly?.removed||empty(),'rendered/body/removed-old-face.png':equippedAssembly?.removed||empty(),'source/body-before-head-restoration.png':f.body,'source/equipped-before-head-restoration.png':ef.body}))put(p+'/'+name,b);
  put('frames/base/'+id+'.png',base);put('frames/'+id+'.png',full);
  const baseOrder=['body/lower-base.png','body/upper-base.png','head-base.png','eyes/open.png','mouth/rest.png'];
  const eqOrder=['rendered/body/lower.png','rendered/body/upper.png','rendered/head-base.png','rendered/eyes/open.png','rendered/mouth/rest.png'];
  poses[id]={...(action==='researching'?{layer_finish:sex==='female'?'research-material-v13':'research-finish-v12'}:{}),canvas:[W,H],pivot:[64,140],facing,source:'source/animation/hip-pivot/'+sex+'-'+(action==='hammering'?'hammering':action==='quenching'||action==='forging'?'heat':'bench')+'-underlayer.png',layers:baseOrder.map(n=>p+'/'+n),rendered_variant_layers:eqOrder.map(n=>p+'/'+n),pose_layers_interchangeable:false,rendered_equipment_baked:true,rendered_tools_baked:true,source_partition_roundtrip:true,neck_assembly:{body_owned_mount:baseAssembly?{revision:'joint-rebuild-v11',policy:baseAssembly.policy,base_point:baseAssembly.point,equipped_point:equippedAssembly.point,base_shoulder_polygon:baseAssembly.polygon,equipped_shoulder_polygon:equippedAssembly.polygon}:null,source_skin_only:sourceSkinOnly,method:'Source-owned complete garment and short neck; no body cut from moving-head alpha; no per-key head offsets',base_neck_polygon:baseAssembly?.neckPolygon||[],equipped_neck_polygon:equippedAssembly?.neckPolygon||[],base_skin_points:identity.measurement?.skin_points||[],equipped_skin_points:equippedIdentity.measurement?.skin_points||[]},head_identity:identity.identity,rendered_head_identity:equippedIdentity.identity,source_registrations:{base:group.blank[index].registration,equipped:group.equipped[index].registration},sockets:{neck:headJoint(parts.head),head:[Math.round((bounds(parts.head).box[0]+bounds(parts.head).box[2])/2),Math.round((bounds(parts.head).box[1]+bounds(parts.head).box[3])/2)],foot:[64,140]},front_layers:{base:[p+'/body/arms-front-base.png'],equipped:[p+'/body/arms-front.png']},tool_clip_guide:p+'/props/held.png',base:'frames/base/'+id+'.png',composed:'frames/'+id+'.png'};
 }
 return poses;
}
