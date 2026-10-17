import {restoreFace} from './restore-face.mjs';
import {cleanHead} from './clean-head.mjs';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
const ROOT=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const W=96,H=128,N=W*H*4;
const CANVAS=[128,160],OFFSET=[16,16],FOOT=[64,140];
const run=(args,input)=>execFileSync('magick',args.map(String),{input,maxBuffer:48*1024*1024});
const blank=()=>Buffer.alloc(N);
function raster(p){const[w,h]=run(['identify','-format','%w %h',p]).toString().split(' ').map(Number);return{w,h,b:run([p,'-depth','8','rgba:-'])};}
function sample(p,transform=(x,y,w,h)=>[x*w/W,y*h/H]){
 const{w,h,b}=raster(p),out=blank();
 for(let y=0;y<H;y++)for(let x=0;x<W;x++){
  const[u,v]=transform(x+.5,y+.5,w,h),sx=Math.floor(u),sy=Math.floor(v),j=(y*W+x)*4;
  if(sx<0||sx>=w||sy<0||sy>=h)continue;
  const i=(sy*w+sx)*4;if(b[i+3]>=128){b.copy(out,j,i,i+3);out[j+3]=255;}
 }return out;
}
function select(b,fn){const out=blank();for(let y=0;y<H;y++)for(let x=0;x<W;x++){const i=(y*W+x)*4;if(b[i+3]&&fn(x,y,b.subarray(i,i+4)))b.copy(out,i,i,i+4);}return out;}
function over(...bs){const out=blank();for(const b of bs)for(let i=0;i<N;i+=4)if(b[i+3])b.copy(out,i,i,i+4);return out;}
function move(b,dx,dy){const out=blank();for(let y=0;y<H;y++)for(let x=0;x<W;x++){const i=(y*W+x)*4,xx=x+dx,yy=y+dy;if(b[i+3]){if(xx<0||xx>=W||yy<0||yy>=H)throw Error('Layer overflow');b.copy(out,(yy*W+xx)*4,i,i+4);}}return out;}
function poly(x,y,ps){let ok=false;for(let i=0,j=ps.length-1;i<ps.length;j=i++){const[a,b]=ps[i],[c,d]=ps[j];if((b>y)!==(d>y)&&x<(c-a)*(y-b)/(d-b)+a)ok=!ok;}return ok;}
const box=(x,y,b)=>x>=b[0]&&y>=b[1]&&x<b[2]&&y<b[3];
const skin=p=>p[0]>185&&p[1]>105&&p[2]>55&&p[0]>p[1]*1.22;
const feature=p=>p[0]<205&&p[1]<160&&p[2]<140||p[0]>205&&p[1]>185&&p[2]>155;
function bounds(b){let x0=W,y0=H,x1=0,y1=0,n=0;for(let y=0;y<H;y++)for(let x=0;x<W;x++)if(b[(y*W+x)*4+3]){x0=Math.min(x0,x);y0=Math.min(y0,y);x1=Math.max(x1,x+1);y1=Math.max(y1,y+1);n++;}return{bbox:[x0,y0,x1,y1],pixels:n};}
function grow(b){const out=blank();for(let y=0;y<H;y++)for(let x=0;x<W;x++)if(b[(y*W+x)*4+3])for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++)if(x+dx>=0&&x+dx<W&&y+dy>=0&&y+dy<H)out[((y+dy)*W+x+dx)*4+3]=255;return out;}
function registerGlove(src,hand){
 const s=bounds(src).bbox,t=bounds(hand).bbox,allowed=grow(grow(hand));
 let best=null;
 for(let left=0;left<=3;left++)for(let right=0;right<=3;right++)for(let top=0;top<=3;top++)for(let bottom=0;bottom<=3;bottom++){
  const rect=[t[0]-left,t[1]-top,t[2]+right,t[3]+bottom],out=blank();
  for(let y=rect[1];y<rect[3];y++)for(let x=rect[0];x<rect[2];x++){
   const sx=Math.floor(s[0]+(x-rect[0]+.5)*(s[2]-s[0])/(rect[2]-rect[0]));
   const sy=Math.floor(s[1]+(y-rect[1]+.5)*(s[3]-s[1])/(rect[3]-rect[1]));
   src.copy(out,(y*W+x)*4,(sy*W+sx)*4,(sy*W+sx)*4+4);
  }
  let missing=0,spill=0,extra=0;
  for(let i=3;i<N;i+=4){if(hand[i]&&!out[i])missing++;if(out[i]&&!allowed[i])spill++;if(out[i]&&!hand[i])extra++;}
  const score=missing*100000+spill*1000+extra;
  if(!best||score<best.score)best={out,source:s,target:rect,missing,spill,extra,score};
 }
 if(best.missing)throw Error('Glove does not cover its hand');
 return best;
}
function paletteOf(bs,count,fn=()=>true){const rgb=[];for(const b of bs)for(let y=0;y<H;y++)for(let x=0;x<W;x++){const i=(y*W+x)*4;if(b[i+3]&&fn(x,y))rgb.push(...b.subarray(i,i+3));}const p=run(['-size',`${rgb.length/3}x1`,'-depth','8','rgb:-','-dither','None','-colors',String(count),'-unique-colors','-depth','8','rgb:-'],Buffer.from(rgb));return Array.from({length:p.length/3},(_,k)=>[...p.subarray(k*3,k*3+3)]);}
function remap(b,colors){const out=Buffer.from(b),cache=new Map();for(let i=0;i<N;i+=4)if(b[i+3]){const key=b.readUIntBE(i,3);let c=cache.get(key);if(!c){let score=Infinity;for(const q of colors){const d=(q[0]-b[i])**2+(q[1]-b[i+1])**2+(q[2]-b[i+2])**2;if(d<score){score=d;c=q;}}cache.set(key,c);}for(let k=0;k<3;k++)out[i+k]=c[k];}return out;}
const configs={
 female:{head_cut:49,eyes:[[42,32,51,41],[55,29,61,40]],mouth:[51,43,55,45],face:[41,25,62,48],skin_sample:[52,37],
  straps:[[[38,50],[42,51],[45,58],[46,64],[42,64],[40,58]],[[57,53],[61,54],[62,61],[58,63]]],
  belt:[[36,72],[61,72],[62,79],[59,81],[36,79]],tools:[[34,77],[40,77],[40,86],[39,102],[34,102],[33,89]],
  glove_near:[[24,68],[31,67],[36,71],[40,77],[36,82],[28,79],[23,74]],glove_far:[[61,75],[66,74],[71,79],[73,85],[75,89],[70,92],[65,87],[61,81]],
  glove_shift:[[4,0],[1,3]],apron_target:[34,60,65,103],prop:[[73,88],[78,88],[86,92],[86,102],[76,103],[71,98]],
  neck:[48,49],waist:[48,74],hands:[[37,77],[71,89]],hair_accessory:'hair-tie'},
 male:{head_cut:49,eyes:[[40,29,49,40],[53,31,58,40]],mouth:[49,42,54,44],face:[38,27,59,46],skin_sample:[50,37],
  straps:[[[33,48],[37,49],[41,57],[42,63],[38,62],[36,56]],[[56,51],[61,53],[62,61],[57,62]]],
  belt:[[33,71],[60,71],[62,78],[58,81],[33,79]],tools:[[31,77],[37,77],[39,84],[37,102],[31,102],[30,91]],
  glove_near:[[22,69],[29,68],[36,72],[39,79],[34,84],[26,81],[21,75]],glove_far:[[61,76],[66,75],[72,79],[74,83],[76,87],[73,92],[67,89],[63,84]],
  glove_shift:[[5,0],[0,3]],apron_target:[32,59,65,103],prop:[[73,86],[79,87],[87,92],[86,103],[75,103],[70,98]],
  neck:[46,48],waist:[47,74],hands:[[35,78],[70,89]],hair_accessory:'headband'}
};
for(const[id,c]of Object.entries(configs)){
 const dir=path.join(ROOT,id),file=n=>path.join(dir,n);
 const write=(n,b)=>{
  fs.mkdirSync(path.dirname(file(n)),{recursive:true});const pub=Buffer.alloc(128*160*4);
  for(let y=0;y<H;y++)b.copy(pub,((y+16)*128+16)*4,y*W*4,(y+1)*W*4);
  run(['-size','128x160','-depth','8','rgba:-','-strip','PNG32:'+file(n)],pub);
 };
 const reference=sample(file('source/reference-native.png'));
 const source=raster(file('source/generated-underlayer.png'));
 const alpha=run([file('source/generated-underlayer.png'),'-alpha','extract','-threshold','50%','-format','%@','info:']).toString();
 const[,bw,bh,bx,by]=alpha.match(/(\d+)x(\d+)\+(\d+)\+(\d+)/).map(Number);
 const step=bh/117;
 let footMin=source.w,footMax=0;
 for(let y=Math.floor(by+bh*.93);y<by+bh;y++)for(let x=0;x<source.w;x++)if(source.b[(y*source.w+x)*4+3]>=128){footMin=Math.min(footMin,x);footMax=Math.max(footMax,x+1);}
 const center=(footMin+footMax)/2,under=sample(file('source/generated-underlayer.png'),(x,y)=>[center+(x-48)*step,by+(y-8)*step]);
 const faceBlank=sample(file('source/generated-face-base.png'),(x,y,w,h)=>[(x-14)*w/64,(y-4)*h/52]);
 const e=select(reference,(x,y,p)=>c.eyes.some(b=>box(x,y,b))&&skin(faceBlank.subarray((y*W+x)*4,(y*W+x)*4+4))&&feature(p));
 const m=select(reference,(x,y,p)=>box(x,y,c.mouth)&&p[0]<230&&p[1]<165&&p[2]<125);
 const head=select(reference,(x,y)=>y<c.head_cut);
 const skinIndex=(c.skin_sample[1]*W+c.skin_sample[0])*4,skinPixel=faceBlank.subarray(skinIndex,skinIndex+4);
 if(!skin(skinPixel))throw Error(`${id}: invalid reconstructed face skin`);
 for(let i=0;i<N;i+=4){if(e[i+3])faceBlank.copy(head,i,i,i+4);if(m[i+3])skinPixel.copy(head,i);}
 const accessoryColor=select(reference,(x,y,p)=>y<46&&(id==='female'?x<40&&y<30&&p[0]>110&&p[1]<p[0]*.35&&p[2]<p[0]*.4:p[2]>p[0]*1.07&&p[2]>p[1]*1.02));
 const contourArea=grow(accessoryColor);
 const accessory=select(reference,(x,y,p)=>accessoryColor[(y*W+x)*4+3]||contourArea[(y*W+x)*4+3]&&p[0]<60&&p[1]<55&&p[2]<60);
 const accessoryArea=id==='male'?grow(grow(grow(accessoryColor))):accessory;
 for(let i=0;i<N;i+=4)if(accessoryArea[i+3])under.copy(head,i,i,i+4);
 const bodyPath=path.join(ROOT,`source/revision-1/${id}-body-lines.png`),bodyRaster=raster(bodyPath);
 const bodyAlpha=run([bodyPath,'-alpha','extract','-threshold','50%','-format','%@','info:']).toString().match(/(\d+)x(\d+)\+(\d+)\+(\d+)/).map(Number);
 const bodyStep=bodyAlpha[2]/117,bodyTop=bodyAlpha[4];
 let soleMin=bodyRaster.w,soleMax=0;
 for(let y=Math.floor(bodyTop+bodyAlpha[2]*.93);y<bodyTop+bodyAlpha[2];y++)for(let x=0;x<bodyRaster.w;x++)if(bodyRaster.b[(y*bodyRaster.w+x)*4+3]>=128){soleMin=Math.min(soleMin,x);soleMax=Math.max(soleMax,x+1);}
 const bodyCenter=(soleMin+soleMax)/2;
 const refined=sample(bodyPath,(x,y)=>[bodyCenter+(x-48)*bodyStep,bodyTop+(y-8)*bodyStep]);
 const shirtEdge=grow(select(refined,(x,y,p)=>y>=45&&y<52&&p[0]>190&&p[1]>170&&p[2]>130&&p[0]<p[1]*1.3));
 const body=select(refined,(x,y)=>y>=c.head_cut||y>=45&&shirtEdge[(y*W+x)*4+3]);
 const arms=select(body,(x,y,p)=>y>=63&&y<98&&skin(p));
 const armArea=grow(grow(arms));
 const upper=select(body,(x,y)=>y<74||armArea[(y*W+x)*4+3]>0);
 const lower=select(body,(x,y)=>y>=74&&!armArea[(y*W+x)*4+3]);
 const frontArea=grow(arms);
 const front=select(body,(x,y,p)=>{
  if(!frontArea[(y*W+x)*4+3])return false;
  const skinShade=p[0]>140&&p[1]>75&&p[2]>35&&p[0]>p[1]*1.35;
  const contour=p[0]<110&&p[1]<85&&p[2]<70;
  const cuff=p[0]>190&&p[1]>170&&p[2]>130&&(x<32&&y<73||x>=60&&y<79);
  return skinShade||contour||cuff;
 });
 const leather=p=>!(p[0]>215&&p[1]>130&&p[2]>70&&p[0]>p[1]*1.2)&&!(p[0]>190&&p[1]>175&&p[2]>140);
 const straps=select(reference,(x,y,p)=>c.straps.some(ps=>poly(x+.5,y+.5,ps))&&leather(p));
 const target=c.apron_target;
 const apronBody=sample(path.join(ROOT,'source/generated-apron.png'),(x,y,w,h)=>[(31+(x-target[0])*39/(target[2]-target[0]))*w/W,(64+(y-target[1])*48/(target[3]-target[1]))*h/H]);
 const apron=over(select(apronBody,(x,y)=>box(x,y,target)),straps);
 const belt=select(reference,(x,y)=>poly(x+.5,y+.5,c.belt)||poly(x+.5,y+.5,c.tools));
 const isolatedGloves=sample(path.join(ROOT,'source/generated-gloves.png'));
 const handRegions=id==='female'?[[29,70,42,81],[62,81,73,94]]:[[23,73,38,85],[59,82,73,97]];
 const handTargets=handRegions.map(region=>select(body,(x,y,p)=>box(x,y,region)&&skin(p)));
 const fitted=handTargets.map((hand,k)=>registerGlove(select(isolatedGloves,(x)=>k===0?x<50:x>=50),hand));
 const gloveNear=fitted[0].out,gloveFar=fitted[1].out;
 const gloves=over(gloveNear,gloveFar);
 const farBox=bounds(gloveFar).bbox,hammerOffset=[farBox[2]-74,farBox[3]-92];
 const hammer=move(select(reference,(x,y)=>poly(x+.5,y+.5,c.prop)),...hammerOffset);
 const towel=id==='male'?select(reference,(x,y,p)=>poly(x+.5,y+.5,[[55,78],[60,78],[61,88],[62,99],[55,99],[54,92]])&&(p[0]>170&&p[1]>145&&p[2]>105||p[0]>140&&p[0]>p[1]*1.5)):blank();
 // The towel owns its upper hem too; toggling it off must not leave a white
 // fragment baked into the adjacent belt layer.
 for(let i=0;i<N;i+=4)if(towel[i+3])belt.fill(0,i,i+4);
 const layers={'head-base.png':head,'body/upper-base.png':upper,'body/lower-base.png':lower,'body/arms-front.png':front,'eyes/open.png':e,'mouth/rest.png':m,'equipment/apron.png':apron,'equipment/belt-tools.png':belt,'equipment/gloves.png':gloves,[`equipment/${c.hair_accessory}.png`]:accessory,'props/hammer.png':hammer};
 if(id==='male')layers['equipment/towel.png']=towel;
 const lockedPalette=JSON.parse(fs.readFileSync(file('source/revision-0/palette.json'))).map(s=>[...Buffer.from(s.slice(1),'hex')]);
 const materialColors=paletteOf([upper,lower,apron,gloves,belt],80-lockedPalette.length);
 const colors=[...new Map([...lockedPalette,...materialColors].map(p=>[p.join(','),p])).values()];
 for(const n of Object.keys(layers))layers[n]=remap(layers[n],colors);
 for(const n of ['head-base.png','eyes/open.png','mouth/rest.png',`equipment/${c.hair_accessory}.png`])layers[n]=sample(file('source/revision-0/'+n));
 layers['head-base.png']=cleanHead({id,file,sample,remap,colors,old:layers['head-base.png']});
 const faceRevision=restoreFace({id,file,sample,remap,colors,old:layers['head-base.png'],upper:layers['body/upper-base.png']});
 layers['head-base.png']=faceRevision.head;layers['eyes/open.png']=faceRevision.eyes;layers['mouth/rest.png']=faceRevision.mouth;
 c.eyes=faceRevision.eyeBoxes;c.mouth=faceRevision.mouthBox;
 const usedColors=new Set();
 for(const b of Object.values(layers))for(let i=0;i<N;i+=4)if(b[i+3])usedColors.add(b.readUIntBE(i,3));
 const foundation=over(layers['body/lower-base.png'],layers['body/upper-base.png'],layers['head-base.png']);
 const base=over(foundation,layers['eyes/open.png'],layers['mouth/rest.png']);
 const order=['equipment/apron.png','equipment/belt-tools.png',...(id==='male'?['equipment/towel.png']:[]),'body/arms-front.png','equipment/gloves.png',`equipment/${c.hair_accessory}.png`];
 const neutral=over(base,...order.map(n=>layers[n]));
 const full=over(neutral,layers['props/hammer.png']);
 for(const[n,b]of Object.entries(layers))write(n,b);
 write('source/native-reference.png',remap(reference,lockedPalette));write('source/registered-underlayer.png',remap(refined,colors));
 for(let k=0;k<2;k++)write(`source/hand-target-${k}.png`,handTargets[k]);
 fs.writeFileSync(file('source/glove-registration.json'),JSON.stringify({coordinate_system:'96x128 authoring; add [16,16] for published coordinates',hand_regions:handRegions,hands:fitted.map(({source,target,missing,spill,extra})=>({source,target,uncovered_skin_pixels:missing,outside_two_pixel_hand_margin:spill,extra_pixels:extra})),hammer_offset:hammerOffset},null,2)+'\n');
 write('source/revision-3/registered-full.png',faceRevision.full);write('source/revision-3/head-change-mask.png',faceRevision.mask);
 write('source/foundation.png',foundation);write('source/face-change-mask.png',over(faceRevision.eyes,faceRevision.mouth));write('source/accessory-reconstruction-mask.png',accessoryArea);
 write('base-neutral.png',base);write('neutral.png',neutral);write('neutral-with-hammer.png',full);
 const coverage=Object.fromEntries(Object.entries({base,neutral,full}).map(([n,b])=>[n,Array.from({length:N/4},(_,j)=>foundation[j*4+3]&&!b[j*4+3]?1:0).reduce((a,b)=>a+b,0)]));
 if(Object.values(coverage).some(Boolean))throw Error('Foundation loss');
 fs.mkdirSync(file('qa'),{recursive:true});
 const report={canvas:[W,H],foot_anchor:[48,124],palette_colors:colors.length,foundation_lost_pixels:coverage,layer_bounds:Object.fromEntries(Object.entries(layers).map(([n,b])=>[n,bounds(b)])),source_registration:{body:{sample_step:step,sample_origin:[center-48*step,by-8*step]},face_crop:{source_cells:[64,52],offset:[14,4]},glove_source:'../source/generated-gloves.png',glove_translation:{near:id==='male'?[-2,1]:[0,0],far:[0,0]},reference_hammer_translation:c.glove_shift[1],apron_body_target:target},source_eye_feature_pixels:bounds(faceRevision.eyes).pixels,source_mouth_feature_pixels:bounds(faceRevision.mouth).pixels,runtime_integration:'not_integrated'};
 report.canvas=CANVAS;report.foot_anchor=FOOT;report.base_offset_px=OFFSET;
 for(const value of Object.values(report.layer_bounds))value.bbox=value.bbox.map(v=>v+16);
 report.source_registration.face_crop={source_cells:[80,68],offset:[8,0],phase_x:faceRevision.phaseX,full_source:'source/revision-3/generated-full.png',blank_source:'source/revision-3/generated-blank.png',policy:'One shared transform for head, eyes, mouth and neck; blank adopted only below features'};
 report.source_registration.neck_repair=id==='male'?{head_roi:[37,43,61,49],exposed_skin_roi:[39,49,57,56],coordinate_system:'authoring'}:null;
 report.source_registration.authoring_canvas=[96,128];report.source_registration.published_offset=OFFSET;
 report.source_registration.body={source:`../source/revision-1/${id}-body-lines.png`,sample_step:bodyStep,sample_origin:[bodyCenter-48*bodyStep,bodyTop-8*bodyStep]};
 report.source_registration.gloves=fitted.map(({source,target,missing,spill})=>({source,target,uncovered_hand_pixels:missing,outside_two_pixel_hand_margin:spill}));
 delete report.source_registration.glove_translation;
 report.source_registration.reference_hammer_translation=hammerOffset;
 fs.writeFileSync(file('qa/coverage-report.json'),JSON.stringify(report,null,2)+'\n');
 fs.writeFileSync(file('source/palette.json'),JSON.stringify(colors.map(p=>'#'+p.map(v=>v.toString(16).padStart(2,'0')).join('')),null,2)+'\n');
 fs.writeFileSync(file('source/ownership.json'),JSON.stringify(c,null,2)+'\n');
 const recipe={canvas:[W,H],pivot:[48,124],sampler:'nearest',mipmaps:false,base:['body/lower-base.png','body/upper-base.png','head-base.png','eyes/open.png','mouth/rest.png'],equipped:order,optional_prop:'props/hammer.png',states:{base:'base-neutral.png',neutral:'neutral.png',with_hammer:'neutral-with-hammer.png'},animation:{frames:['neutral'],loop:false},runtime_integration:false};
 recipe.canvas=CANVAS;recipe.pivot=FOOT;recipe.authoring_canvas=[96,128];recipe.base_offset_px=OFFSET;
 fs.writeFileSync(file('character.json'),JSON.stringify(recipe,null,2)+'\n');
 const span=(b,y,x0,x1)=>{const xs=[];for(let x=x0;x<x1;x++)if(b[(y*W+x)*4+3])xs.push(x);return xs.length?Math.max(...xs)-Math.min(...xs)+1:0;};
 const size=b=>{const[a,c,d,e]=bounds(b).bbox;return[d-a,e-c];};
 const nearEye=bounds(select(layers['eyes/open.png'],(x,y)=>box(x,y,c.eyes[0]))).bbox;
 const farEye=bounds(select(layers['eyes/open.png'],(x,y)=>box(x,y,c.eyes[1]))).bbox;
 const spec={version:1,id:`player-${id}`,grid:'B',grid_unit_px:[16,16],canvas_cells:[6,8],native_canvas_px:[96,128],preview_scale:4,origin_px:[0,0],foot_anchor_px:[48,124],
  reference_roles:{identity:'source/reference.png',style:'source/reference.png',expressions:'source/reference.png'},
  reconstructed_occluded_regions:['Full shirt chest and waist under apron/straps/belt','Trouser hips and thighs under apron','Bare hands beneath gloves',id==='male'?'Full brown hair beneath removable headband':'Brown hair beneath removable hair tie','Apron leather under belt/tools/hand'],
  baseline:{complete_underlayer:'source/foundation.png',foundation:'source/foundation.png',facial_change_mask:'source/face-change-mask.png',builder:'../source/build-characters.mjs',coverage_report:'qa/coverage-report.json'},
  palette:{max_opaque_colors:64,actual_opaque_colors:usedColors.size,palette_entries:colors.length,rgba_hex:'source/palette.json',alpha_values:[0,255],reason:'Preserve reference face/blush, copper hair, leather, steel and brass; reserve separate accessory pigments'},
  identity:{head_size_without_hat_px:size(layers['head-base.png']),head_measurement:'Head-base alpha bounds including short neck attachment; native layer measurement, not a source-scale equality claim',face_bbox_px:c.face,face_bbox_measurement:'Reference interior face ROI for pigment preservation, excluding ear',eye_size_px:{near:[nearEye[2]-nearEye[0],nearEye[3]-nearEye[1]],far:[farEye[2]-farEye[0],farEye[3]-farEye[1]]},eye_spacing_px:farEye[0]-nearEye[2],eye_white_iris_design:'Coherent generated asymmetric brown eyes at shared head registration, following the original reference gaze',shoulder_width_px:span(upper,58,15,70),chest_width_px:span(upper,66,32,67),waist_width_px:span(foundation,75,30,66),upper_arm_width_px:span(upper,68,20,35),thigh_width_px:span(lower,91,25,67),leg_length_px:51,body_measurement:'Row spans: shoulder y58 x15..69; torso y66 x32..66; waist y75 x30..65; near arm y68 x20..34; combined thighs y91 x25..66; legs from waist74 to sole124 inclusive',body_build:id==='female'?'Compact reference female with full hips and sturdy work trousers':'Compact reference male with broad shirt and sturdy hips/legs',hair_color:'Copper/chestnut brown, no grey hair pigment',beard_and_moustache_mask:'none',preserved_expression_character:'Original small smile and rightward three-quarter gaze; blush fixed on head base',outline_rule:'Native dark contour, separate material shadows; no blanket erosion or lightening'},
  limb_identity:{side_convention:'character_anatomical',near_side:'right',far_side:'left',identifying_marks:'Near hand at hip; far hand at side'},
  sockets_px:{neck:c.neck,waist:c.waist,eyes:[Math.round((nearEye[0]+farEye[2])/2),Math.round((nearEye[1]+nearEye[3])/2)],mouth:bounds(layers['mouth/rest.png']).bbox.slice(0,2),right_hand:c.hands[0],left_hand:c.hands[1]},
  layers:{head_base:'head-base.png',upper_base:'body/upper-base.png',lower_base:'body/lower-base.png',arms_front:'body/arms-front.png',eyes:{open:'eyes/open.png'},mouth:{rest:'mouth/rest.png'},equipment:Object.keys(layers).filter(n=>n.startsWith('equipment/')),props:['props/hammer.png']},eyebrow_owner:'eyes',expression_recipes:{neutral:{eyes:'open',mouth:'rest'}},composition_order:[...recipe.base,...order],pose_overrides:{},production_plan:{status:'neutral_base_pose_generated',current_scope:'One right-facing neutral pose per sex; bare, equipped and optional hammer variants'},animation:{frame_order:['neutral'],looping:false},runtime_export:{status:'not_integrated',mode:'layered_authoring_assets',manifest:'character.json',sampler:'nearest',mipmaps:false},generation:{tool:'built-in image_gen',final_prompts:'../source/prompts.json',registration:'qa/coverage-report.json',ownership:'source/ownership.json'}};
 spec.canvas_cells=[8,10];spec.native_canvas_px=CANVAS;spec.foot_anchor_px=FOOT;
 spec.authoring_canvas_px=[96,128];spec.base_offset_px=OFFSET;
 spec.palette.max_opaque_colors=80;
 spec.palette.reason='Keep the approved face palette locked; add material colors for NPC-quality cloth folds, leather/steel boot detail and brass';
 spec.identity.face_bbox_px=spec.identity.face_bbox_px.map(v=>v+16);
 spec.identity.body_measurement+='; these row/ROI measurements use the 96x128 source coordinates, add [16,16] on published assets';
 const centerOf=b=>{const [x,y,a,z]=bounds(b).bbox;return[Math.round((x+a-1)/2),Math.round((y+z-1)/2)];};
 spec.sockets_px.right_hand=centerOf(handTargets[0]);spec.sockets_px.left_hand=[farBox[2]-1,farBox[3]-2];
 spec.sockets_px.mouth=centerOf(layers['mouth/rest.png']);
 for(const key of Object.keys(spec.sockets_px))spec.sockets_px[key]=spec.sockets_px[key].map(v=>v+16);
 spec.generation.quality_revision='NPC body line/material references; independently registered gloves; coherent revision-3 facial details and original closed smile; local male jaw/neck repair; clothes, hands and equipment preserved';
 spec.generation.head_revision_prompts='../source/revision-3/prompts.json';
 spec.generation.glove_registration='source/glove-registration.json';
 spec.generation.revision_prompts='../source/revision-1/prompts.json';
 fs.writeFileSync(file('character-spec.yaml'),JSON.stringify(spec,null,2)+'\n');
 fs.mkdirSync(file('previews'),{recursive:true});
 for(const n of ['base-neutral','neutral','neutral-with-hammer'])run([file(n+'.png'),'-filter','point','-resize','400%','-strip',file(`previews/${n}-4x.png`)]);
 for(const[bg,name]of [['#eee5cd','layers-light'],['#24232a','layers-dark']])run(['montage',...Object.keys(layers).map(file),'-filter','point','-background',bg,'-alpha','remove','-geometry','512x640+6+6','-tile','4x','-strip',file(`previews/${name}.png`)]);
 run(['montage',...['source/native-reference.png','base-neutral.png','neutral.png','neutral-with-hammer.png'].map(file),'-filter','point','-background','#eee5cd','-alpha','remove','-geometry','512x640+6+6','-tile','4x1','-strip',file('previews/reference-base-equipped.png')]);
 run(['montage',...['source/native-reference.png','base-neutral.png','neutral.png'].map(p=>['(',file(p),'-crop','58x48+33+21','+repage',')']).flat(),'-filter','point','-background','#eee5cd','-alpha','remove','-geometry','464x384+6+6','-tile','3x1','-strip',file('previews/face-comparison.png')]);
 const handArgs=['montage'];
 for(const rect of ['29x30+35+76','25x29+72+88'])for(const name of ['base-neutral.png','equipment/gloves.png','neutral.png'])handArgs.push('(',file(name),'-crop',rect,'+repage','-filter','point','-resize','800%',')');
 run([...handArgs,'-background','#eee5cd','-alpha','remove','-geometry','+6+6','-tile','3x2','-strip',file('previews/hand-registration.png')]);
 console.log(JSON.stringify({id,colors:colors.length,bounds:bounds(neutral).bbox,coverage}));
}
for(const[state,out]of [['neutral','players-equipped'],['base-neutral','players-base'],['neutral-with-hammer','players-with-hammer']])run(['montage',...['female','male'].map(id=>path.join(ROOT,id,state+'.png')),'-filter','point','-background','#eee5cd','-alpha','remove','-geometry','512x640+12+12','-tile','2x1','-strip',path.join(ROOT,'previews',out+'.png')]);
run(['montage',...['base-neutral','neutral'].flatMap(state=>['female','male'].map(id=>path.join(ROOT,id,state+'.png'))),'-filter','point','-background','#24232a','-alpha','remove','-geometry','512x640+12+12','-tile','2x2','-strip',path.join(ROOT,'previews/players-dark.png')]);
const comparison=['montage','-font','Arial','-pointsize','14','-fill','#30251c'];
for(const npc of ['farmer','leatherworker','chef','carpenter'])comparison.push('-label',npc,path.join(ROOT,'../npc',npc,'frames/neutral.png'));
for(const id of ['female','male'])comparison.push('-label',`player ${id}`,path.join(ROOT,id,'neutral.png'));
run([...comparison,'-background','#eee5cd','-alpha','remove','-filter','point','-geometry','256x320+6+6','-tile','6x1','-strip',path.join(ROOT,'previews/npc-player-comparison-2x.png')]);
const beforeAfter=['montage'];
for(const id of ['female','male'])for(const before of [true,false]){
 beforeAfter.push('(',path.join(ROOT,id,before?'source/revision-0/neutral.png':'neutral.png'));
 if(before)beforeAfter.push('-background','none','-gravity','center','-extent','128x160');
 beforeAfter.push('-filter','point','-resize','400%',')');
}
run([...beforeAfter,'-background','#eee5cd','-alpha','remove','-geometry','+6+6','-tile','2x2','-strip',path.join(ROOT,'previews/quality-before-after.png')]);

// Face-only current view keeps historical candidates out of the primary review.
const currentHeads=['montage'];
for(const id of ['female','male'])currentHeads.push('(',path.join(ROOT,id,'base-neutral.png'),'-crop','80x56+24+16','+repage','-filter','point','-resize','800%',')');
run([...currentHeads,'-background','#eee5cd','-alpha','remove','-geometry','+12+12','-tile','2x1','-strip',path.join(ROOT,'previews/heads-current.png')]);
const headComparison=['montage','-font','Arial','-pointsize','18','-fill','#30251c'];
for(const id of ['female','male'])for(const before of [true,false])headComparison.push('-label',id+' '+(before?'BEFORE':'AFTER'),'(',path.join(ROOT,id,before?'source/revision-3/before/base-neutral.png':'base-neutral.png'),'-crop','80x56+24+16','+repage','-filter','point','-resize','800%',')');
run([...headComparison,'-background','#eee5cd','-alpha','remove','-geometry','+12+12','-tile','2x2','-strip',path.join(ROOT,'previews/heads-before-after.png')]);
