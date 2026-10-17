// Derive modular expression parts and authored pose frames from reviewed sources.
import {execFileSync} from 'node:child_process';
import {mkdirSync,readFileSync,writeFileSync,rmSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const W=96,H=128,blank=()=>Buffer.alloc(W*H*4);
const file=name=>path.join(root,name);
const run=args=>execFileSync('magick',args.map(String),{maxBuffer:32*1024*1024});
const read=name=>run([file(name),'-depth','8','rgba:-']);
// stdin is needed for native buffers; keep image conversion and drawing separate.
function write(name,data){mkdirSync(path.dirname(file(name)),{recursive:true});execFileSync('magick',['-size','96x128','-depth','8','rgba:-','-define','png:exclude-chunks=date,time','PNG32:'+file(name)],{input:data});}
function over(...layers){const out=blank();for(const layer of layers)for(let i=0;i<out.length;i+=4)if(layer[i+3])layer.copy(out,i,i,i+4);return out;}
function move(source,dx,dy){const out=blank();for(let y=0;y<H;y++)for(let x=0;x<W;x++){const nx=x+dx,ny=y+dy;if(nx>=0&&ny>=0&&nx<W&&ny<H)source.copy(out,(ny*W+nx)*4,(y*W+x)*4,(y*W+x)*4+4);}return out;}
function eyeCenter(source,left,right){const selected=blank();for(let y=33;y<44;y++)for(let x=left;x<right;x++){const i=(y*W+x)*4;source.copy(selected,i,i,i+4);}const b=bounds(selected);return b[2]>b[0]?[Math.round((b[0]+b[2]-1)/2),Math.round((b[1]+b[3]-1)/2)]:null;}
function mainComponent(source){
 const visited=new Set(),components=[];
 for(let n=0;n<W*H;n++)if(source[n*4+3]&&!visited.has(n)){
  const queue=[n];visited.add(n);
  for(let at=0;at<queue.length;at++){const p=queue[at],x=p%W,y=Math.floor(p/W);for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){const a=x+dx,b=y+dy,m=b*W+a;if(a>=0&&a<W&&b>=0&&b<H&&source[m*4+3]&&!visited.has(m)){visited.add(m);queue.push(m);}}}
  components.push(queue);
 }
 components.sort((a,b)=>b.length-a.length);
 if(components.slice(1).some(c=>c.length>12))throw Error('Unexpected detached body component; inspect source');
 const out=blank();for(const n of components[0]||[])source.copy(out,n*4,n*4,n*4+4);return out;
}
const palette=JSON.parse(readFileSync(file('source/palette.json'))).map(c=>[1,3,5].map(i=>parseInt(c.slice(i,i+2),16)));
function color(rgb){let best=palette[0],score=Infinity;for(const p of palette){const d=p.reduce((s,v,k)=>s+(v-rgb[k])**2,0);if(d<score){score=d;best=p;}}return best;}
function candidate(name,kind='head'){
 const [sw,sh]=run(['identify','-format','%w %h',file(name)]).toString().split(' ').map(Number),raw=read(name),out=blank();
 for(let y=0;y<H;y++)for(let x=0;x<W;x++){
  let sx,sy;
  if(kind==='head'){if(x<12||x>=76||y>=64)continue;sx=Math.floor((x-12+.5)*sw/64);sy=Math.floor((y+.5)*sh/64);}
  else if(kind==='lower'){if(y<64)continue;sx=Math.floor((x+.5)*sw/96);sy=Math.floor((y-64+.5)*sh/64);}
  else {sx=Math.floor((x+11)*10*sw/1122);sy=Math.floor((y+10)*10*sh/1402);}
  if(sx<0||sy<0||sx>=sw||sy>=sh)continue;
  const j=(sy*sw+sx)*4,i=(y*W+x)*4;if(raw[j+3]<128)continue;
  const c=color(raw.subarray(j,j+3));for(let k=0;k<3;k++)out[i+k]=c[k];out[i+3]=255;
 }
 return out;
}
const head=read('head-base.png'),upper=read('body/upper-base.png'),lower=read('body/lower-base.png');
const hat=read('equipment/hat.png'),apron=read('equipment/apron.png'),belt=read('equipment/belt-tools.png'),gloves=read('equipment/gloves.png');
const eyes={open:read('eyes/open.png')},mouth={rest:read('mouth/rest.png')};
const skin=p=>p[0]>150&&p[0]-p[1]>55;
const eyeMasks=blank(),mouthMasks=blank();
for(const id of ['half','closed','happy-closed','wide']){
 const src=candidate(`source/expressions/eye-${id}.png`),part=blank();
 for(let y=25;y<42;y++)for(let x=42;x<63;x++){
  const i=(y*W+x)*4,p=src.subarray(i,i+4),under=head.subarray(i,i+4);
  const feature=(Math.max(...p.subarray(0,3))<135&&p[0]>p[1]*1.08&&p[0]>p[2]*1.08)||Math.min(...p.subarray(0,3))>205;
  if(p[3]&&feature&&(skin(under)||eyes.open[i+3]))src.copy(part,i,i,i+4);
 }
 eyes[id]=part;write(`eyes/${id}.png`,part);
}
for(const id of ['small-open','wide-open','round']){
 const src=candidate(`source/expressions/mouth-${id}.png`),part=blank();
 for(let y=44;y<49;y++)for(let x=49;x<59;x++){
  const i=(y*W+x)*4,p=src.subarray(i,i+4),under=head.subarray(i,i+4);
  const feature=Math.max(...p.subarray(0,3))<140||Math.min(...p.subarray(0,3))>205||(p[0]>130&&p[1]<100);
  if(p[3]&&feature&&(skin(under)||mouth.rest[i+3]))src.copy(part,i,i,i+4);
 }
 mouth[id]=part;write(`mouth/${id}.png`,part);
}
for(const [mask,parts] of [[eyeMasks,Object.values(eyes)],[mouthMasks,Object.values(mouth)]])for(const part of parts)for(let i=0;i<part.length;i+=4)if(part[i+3])mask.fill(255,i,i+4);
write('source/expression-eye-mask.png',eyeMasks);write('source/expression-mouth-mask.png',mouthMasks);
const recipes={neutral:['open','rest'],'talk-small':['open','small-open'],'talk-wide':['open','wide-open'],'talk-round':['open','round'],blink:['closed','rest'],happy:['happy-closed','rest'],surprised:['wide','round'],relaxed:['half','rest']};
const base=over(lower,upper,head),equipped=over(apron,belt,gloves,hat),frames={};
for(const [id,[eye,lip]] of Object.entries(recipes)){
 const composed=over(base,eyes[eye],mouth[lip],equipped);frames[id]=composed;
 write(`expressions/${id}.png`,composed);
 write(`expressions/base/${id}.png`,over(base,eyes[eye],mouth[lip]));
}
for(const eye of Object.keys(eyes))for(const lip of Object.keys(mouth))write(`previews/combinations/${eye}--${lip}.png`,over(base,eyes[eye],mouth[lip],equipped));
function sheet(name,paths,columns,scale=4){run(['montage',...paths.map(file),'-filter','point','-background','#efe5cd','-geometry',`${W*scale}x${H*scale}+8+8`,'-tile',`${columns}x`, '-define','png:exclude-chunks=date,time',file('previews/'+name)]);}
sheet('expressions-8.png',Object.keys(recipes).map(id=>`expressions/${id}.png`),4);
sheet('expressions-20.png',Object.keys(eyes).flatMap(e=>Object.keys(mouth).map(m=>`previews/combinations/${e}--${m}.png`)),4,3);
writeFileSync(file('expressions/recipes.json'),JSON.stringify({canvas:[W,H],eyes:Object.fromEntries(Object.keys(eyes).map(id=>[id,`eyes/${id}.png`])),mouth:Object.fromEntries(Object.keys(mouth).map(id=>[id,`mouth/${id}.png`])),recipes:Object.fromEntries(Object.entries(recipes).map(([id,[eye,lip]])=>[id,{eyes:eye,mouth:lip}]))},null,2)+'\n');
const poseFrames={},poseBases={},poseRecipes={};
for(const [id,hatOffset] of [['nod-down',[0,2]],['turn-toward',[-1,0]],['turn-away',[1,0]]]){
 const blankSource=id==='turn-away'?'turn-away-blank-fixed':`${id}-blank`;
 const raw=candidate(`source/poses/${id}.png`),clean=candidate(`source/poses/${blankSource}.png`);
 const h=blank(),eye=blank(),lip=blank();
 for(let y=0;y<57;y++)for(let x=0;x<W;x++){
  const i=(y*W+x)*4,p=raw.subarray(i,i+4),under=clean.subarray(i,i+4);
  const headPixel=y<50||(y<57&&x>=38&&x<=65);
  if(headPixel)clean.copy(h,i,i,i+4);
  const feature=(Math.max(...p.subarray(0,3))<140&&p[0]>p[1]*1.05)||Math.min(...p.subarray(0,3))>205;
  if(!headPixel||!p[3]||!skin(under)||!feature)continue;
  if(x>=40&&x<66&&y>=25&&y<43)raw.copy(eye,i,i,i+4);
  if(x>=48&&x<65&&y>=43&&y<51)raw.copy(lip,i,i,i+4);
 }
 const cap=move(hat,...hatOffset),body=over(lower,upper,h),underlayer=over(body,eye,lip),full=over(underlayer,apron,belt,gloves,cap);
 for(const [name,data] of Object.entries({'head-base.png':h,'eyes/open.png':eye,'mouth/rest.png':lip,'equipment/hat.png':cap,'base.png':underlayer,'composed.png':full}))write(`poses/${id}/${name}`,data);
 poseBases[id]=underlayer;poseFrames[id]=full;
 const mb=bounds(lip);
 poseRecipes[id]={layers:['body/lower-base.png','body/upper-base.png',`poses/${id}/head-base.png`,`poses/${id}/eyes/open.png`,`poses/${id}/mouth/rest.png`,'equipment/apron.png','equipment/belt-tools.png','equipment/gloves.png',`poses/${id}/equipment/hat.png`],sockets_px:{neck:[46,55],waist:[48,76],eyes:[eyeCenter(eye,40,55),eyeCenter(eye,55,66)],mouth:[Math.round((mb[0]+mb[2]-1)/2),Math.round((mb[1]+mb[3]-1)/2)],left_hand:[70,85],right_hand:[30,79]},eye_states:['open'],mouth_states:['rest'],head_source:`source/poses/${blankSource}.png`,hat_offset:hatOffset};
}
sheet('head-poses.png',['neutral.png',...Object.keys(poseFrames).map(id=>`poses/${id}/composed.png`)],4);
sheet('head-poses-no-hat.png',['base-neutral.png',...Object.keys(poseFrames).map(id=>`poses/${id}/base.png`)],4);
run([file('poses/turn-away/head-base.png'),'-crop','64x64+12+0','+repage','-filter','point','-resize','800%','-background','#efe5cd','-alpha','remove','-define','png:exclude-chunks=date,time',file('previews/turn-away-base-fixed-8x.png')]);
function region(x,y,boxes){return boxes.some(([a,b,c,d])=>x>=a&&x<c&&y>=b&&y<d);}
function grow(mask,radius){const out=blank();for(let y=0;y<H;y++)for(let x=0;x<W;x++)if(mask[(y*W+x)*4+3])for(let dy=-radius;dy<=radius;dy++)for(let dx=-radius;dx<=radius;dx++){const nx=x+dx,ny=y+dy;if(nx>=0&&nx<W&&ny>=0&&ny<H)out[(ny*W+nx)*4+3]=255;}return out;}
const armSources={
 'give-mid':{source:'give-mid',hands:[[44,69,66,84],[64,67,82,84]],sockets:[[57,77],[75,75]]},
 'give-extended':{source:'give-extended',hands:[[43,62,67,78],[70,59,92,78]],sockets:[[59,71],[85,67]]},
 'walk-upper-a':{source:'walk-upper-a',hands:[[15,71,33,92],[65,70,85,90]],sockets:[[21,79],[74,79]]},
 'walk-upper-b':{source:'walk-upper-b-fixed',hands:[[41,73,59,90],[59,72,77,92]],sockets:[[50,80],[67,80]]}
};
const upperPoses={};
// The offering source exposes the hip formerly hidden by the hand-on-hip pose.
const standingSource=candidate('source/poses/give-extended.png','body'),standingLowerRaw=blank();
for(let y=76;y<H;y++)for(let x=0;x<W;x++){const i=(y*W+x)*4;if(y>=82||x<65)standingSource.copy(standingLowerRaw,i,i,i+4);}
const standingLower=mainComponent(standingLowerRaw);
write('poses/shared/lower-base.png',standingLower);
for(const [id,config] of Object.entries(armSources)){
 const src=candidate(`source/poses/${config.source}.png`,'body'),gloved=candidate(`source/poses/${id}-gloves.png`,'body');
 const allSkin=blank(),handSkin=blank(),u=blank(),front=blank(),g=blank();
 for(let y=60;y<96;y++)for(let x=0;x<W;x++){
  const i=(y*W+x)*4;
  if(src[i+3]&&skin(src.subarray(i,i+4))){allSkin[i+3]=255;if(region(x,y,config.hands))handSkin[i+3]=255;}
 }
 const armBorder=grow(allSkin,1),gloveArea=grow(handSkin,2);
 for(let y=50;y<96;y++)for(let x=0;x<W;x++){
  const i=(y*W+x)*4,p=gloved.subarray(i,i+4);
  if(y<76||armBorder[i+3])src.copy(u,i,i,i+4);
  if(armBorder[i+3])src.copy(front,i,i,i+4);
  if(gloveArea[i+3]&&p[3]&&p[0]<205&&p[1]<150&&p[2]<125)gloved.copy(g,i,i,i+4);
 }
 const b=over(standingLower,u,head,eyes.open,mouth.rest),full=over(b,apron,belt,front,g,hat);
 for(const [name,data] of Object.entries({'body/upper-base.png':u,'body/arms-front.png':front,'equipment/gloves.png':g,'base.png':b,'composed.png':full}))write(`poses/${id}/${name}`,data);
 upperPoses[id]={upper:u,front,gloves:g};poseFrames[id]=full;poseBases[id]=b;
 poseRecipes[id]={layers:['poses/shared/lower-base.png',`poses/${id}/body/upper-base.png`,'head-base.png','eyes/open.png','mouth/rest.png','equipment/apron.png','equipment/belt-tools.png',`poses/${id}/body/arms-front.png`,`poses/${id}/equipment/gloves.png`,'equipment/hat.png'],sockets_px:{neck:[46,55],waist:[48,76],eyes:[[48,37],[59,37]],mouth:[53,45],left_hand:config.sockets[1],right_hand:config.sockets[0]},source:`source/poses/${config.source}.png`};
}
for(let k=0;k<6;k++){
 const id=`walk-${k}`,leg=candidate(`source/poses/${k===1?'walk-1-fixed':id}.png`,'lower');
 // Keep the authored lower-body key on its original canvas, including the waistband.
 const legRaw=blank();for(let y=75;y<H;y++)leg.copy(legRaw,y*W*4,y*W*4,(y+1)*W*4);
 const cleanedLeg=mainComponent(legRaw);
 let bottom=0;for(let y=0;y<H;y++)for(let x=0;x<W;x++)if(cleanedLeg[(y*W+x)*4+3])bottom=Math.max(bottom,y);
 const legOffset=124-bottom,hipBridge=blank();
 for(let y=76;y<83;y++)standingLower.copy(hipBridge,y*W*4,y*W*4,(y+1)*W*4);
 const legPart=over(hipBridge,move(cleanedLeg,0,legOffset));
 // The gold-buckled near leg advances at 0/1: near arm back (A), far arm forward.
 // The far leg advances at 3/4: near arm forward (B), far arm back.
 // Passing keys 2/5 use the neutral arm position between those extremes.
 const owner=k<3?'walk-upper-a':'walk-upper-b';
 const passing=k===2||k===5;
 const a=passing?{upper,front:blank(),gloves}:upperPoses[owner],dy=[0,1,0,0,1,0][k];
 const u=move(a.upper,0,dy),h=move(head,0,dy),eye=move(eyes.open,0,dy),lip=move(mouth.rest,0,dy);
 const cap=move(hat,0,dy),cloth=move(apron,0,dy),tool=move(belt,0,dy),front=move(a.front,0,dy),g=move(a.gloves,0,dy);
 const b=over(legPart,u,h,eye,lip),full=over(b,cloth,tool,front,g,cap);
 const layers={'body/lower-base.png':legPart,'body/upper-base.png':u,'body/arms-front.png':front,'head-base.png':h,'eyes/open.png':eye,'mouth/rest.png':lip,'equipment/hat.png':cap,'equipment/apron.png':cloth,'equipment/belt-tools.png':tool,'equipment/gloves.png':g};
 const layerNames=['body/lower-base.png','body/upper-base.png','head-base.png','eyes/open.png','mouth/rest.png','equipment/apron.png','equipment/belt-tools.png','body/arms-front.png','equipment/gloves.png','equipment/hat.png'];
 if(passing){delete layers['body/arms-front.png'];layerNames.splice(layerNames.indexOf('body/arms-front.png'),1);rmSync(file(`poses/${id}/body/arms-front.png`),{force:true});}
 for(const [name,data] of Object.entries({...layers,'base.png':b,'composed.png':full}))write(`poses/${id}/${name}`,data);
 poseFrames[id]=full;poseBases[id]=b;
 const sockets=passing?[[30,79],[70,85]]:armSources[owner].sockets;
 poseRecipes[id]={layers:layerNames.map(p=>`poses/${id}/${p}`),sockets_px:{neck:[46,55+dy],waist:[48,76+dy],eyes:[[48,37+dy],[59,37+dy]],mouth:[53,45+dy],left_hand:[sockets[1][0],sockets[1][1]+dy],right_hand:[sockets[0][0],sockets[0][1]+dy]},body_offset:[0,dy],leg_offset:[0,legOffset],arm_source:passing?'neutral':owner,leg_phase:['near-contact','near-support','far-passing','far-contact','far-support','near-passing'][k],source:`source/poses/${k===1?'walk-1-fixed':id}.png`};
}
sheet('give-poses.png',['neutral.png','poses/give-mid/composed.png','poses/give-extended/composed.png'],3);
sheet('give-poses-no-equipment.png',['base-neutral.png','poses/give-mid/base.png','poses/give-extended/base.png'],3);
sheet('walk-6.png',Array.from({length:6},(_,k)=>`poses/walk-${k}/composed.png`),3);
sheet('walk-6-no-equipment.png',Array.from({length:6},(_,k)=>`poses/walk-${k}/base.png`),3);
writeFileSync(file('poses/recipes.json'),JSON.stringify({canvas:[W,H],poses:poseRecipes},null,2)+'\n');
const frameStore={...frames,...Object.fromEntries(Object.entries(poseFrames).filter(([id])=>!id.startsWith('walk-upper-')))};
const baseStore={...Object.fromEntries(Object.entries(recipes).map(([id,[e,m]])=>[id,over(base,eyes[e],mouth[m])])),...poseBases};
frameStore['blink-half']=over(base,eyes.half,mouth.rest,equipped);baseStore['blink-half']=over(base,eyes.half,mouth.rest);
frameStore['talk-blink']=over(base,eyes.closed,mouth['small-open'],equipped);baseStore['talk-blink']=over(base,eyes.closed,mouth['small-open']);
const nodHead=move(read('poses/nod-down/head-base.png'),0,-1),nodEye=move(read('poses/nod-down/eyes/open.png'),0,-1),nodLip=move(read('poses/nod-down/mouth/rest.png'),0,-1),nodHat=move(read('poses/nod-down/equipment/hat.png'),0,-1);
baseStore['nod-mid']=over(lower,upper,nodHead,nodEye,nodLip);frameStore['nod-mid']=over(baseStore['nod-mid'],apron,belt,gloves,nodHat);
for(const [p,data] of Object.entries({'head-base.png':nodHead,'eyes/open.png':nodEye,'mouth/rest.png':nodLip,'equipment/hat.png':nodHat,'base.png':baseStore['nod-mid'],'composed.png':frameStore['nod-mid']}))write('poses/nod-mid/'+p,data);
poseRecipes['nod-mid']={layers:['body/lower-base.png','body/upper-base.png','poses/nod-mid/head-base.png','poses/nod-mid/eyes/open.png','poses/nod-mid/mouth/rest.png','equipment/apron.png','equipment/belt-tools.png','equipment/gloves.png','poses/nod-mid/equipment/hat.png'],sockets_px:{...poseRecipes['nod-down'].sockets_px,neck:[46,54],eyes:poseRecipes['nod-down'].sockets_px.eyes.map(p=>p?[p[0],p[1]-1]:null),mouth:poseRecipes['nod-down'].sockets_px.mouth.map((v,k)=>k===1?v-1:v)},head_offset:[0,-1]};
writeFileSync(file('poses/recipes.json'),JSON.stringify({canvas:[W,H],poses:poseRecipes},null,2)+'\n');
const clip=(ids,times,loop=false)=>({loop,frames:ids.map((sprite,k)=>({sprite,duration_ms:times[k]}))});
const clips={
 idle:clip(['neutral','blink-half','blink','blink-half','neutral'],[1600,40,80,40,1200],true),
 talk:clip(['neutral','talk-small','talk-wide','talk-small','talk-round','neutral'],[140,100,100,100,100,140],true),
 blink:clip(['neutral','blink-half','blink','blink-half','neutral'],[300,40,80,40,300]),
 surprised:clip(['surprised','neutral'],[600,140]),
 happy:clip(['happy','neutral'],[700,140]),
 relaxed:clip(['relaxed','neutral'],[700,140]),
 nod:clip(['neutral','nod-mid','nod-down','nod-mid','neutral'],[120,100,140,100,120]),
 shake_head:clip(['neutral','turn-toward','neutral','turn-away','neutral'],[100,140,100,140,100]),
 give:clip(['neutral','give-mid','give-extended','give-extended','give-mid','neutral'],[120,120,180,120,120,120]),
 walk:clip(Array.from({length:6},(_,k)=>`walk-${k}`),Array(6).fill(120),true),
 talk_blink:clip(['talk-small','talk-blink','talk-small','talk-wide','talk-round'],[240,100,140,140,140],true)
};
clips.give.frames[3].event='transfer';
for(const [id,data] of Object.entries(frameStore)){write(`frames/${id}.png`,data);write(`frames/base/${id}.png`,baseStore[id]);}
writeFileSync(file('animation.json'),JSON.stringify({version:1,canvas:[W,H],foot:[48,124],duration_unit:'ms',root_motion:false,frames:Object.fromEntries(Object.keys(frameStore).map(id=>[id,{equipped:`frames/${id}.png`,base:`frames/base/${id}.png`}])),clips},null,2)+'\n');
for(const id of ['talk','blink','nod','shake_head','give','walk','talk_blink']){
 const args=[];for(const frame of clips[id].frames)args.push('-delay',String(Math.round(frame.duration_ms/10)),file(`frames/${frame.sprite}.png`));
 run([...args,'-filter','point','-resize','400%','-background','#efe5cd','-alpha','remove','-dither','None','-loop','0',file(`previews/${id}.gif`)]);
}
function bounds(data){let a=W,b=H,c=0,d=0;for(let y=0;y<H;y++)for(let x=0;x<W;x++)if(data[(y*W+x)*4+3]){a=Math.min(a,x);b=Math.min(b,y);c=Math.max(c,x+1);d=Math.max(d,y+1);}return[a,b,c,d];}
const placements=[],byPixels=new Map(),pageWidth=512;let px=0,py=0,rowHeight=0;
for(const id of Object.keys(frameStore).sort()){
 const signature=frameStore[id].toString('base64');
 if(byPixels.has(signature)){placements.push({...byPixels.get(signature),id});continue;}
 const [a,b,c,d]=bounds(frameStore[id]),w=c-a,h=d-b;
 if(px+w+4>pageWidth){px=0;py+=rowHeight;rowHeight=0;}
 const placement={id,a,b,w,h,x:px+2,y:py+2};placements.push(placement);byPixels.set(signature,placement);px+=w+4;rowHeight=Math.max(rowHeight,h+4);
}
const pageHeight=Math.ceil((py+rowHeight)/16)*16,page=Buffer.alloc(pageWidth*pageHeight*4);
for(const p of placements){
 const src=frameStore[p.id];
 for(let dy=-1;dy<=p.h;dy++)for(let dx=-1;dx<=p.w;dx++){
  const sx=p.a+Math.max(0,Math.min(p.w-1,dx)),sy=p.b+Math.max(0,Math.min(p.h-1,dy));
  const si=(sy*W+sx)*4,di=((p.y+dy)*pageWidth+p.x+dx)*4;src.copy(page,di,si,si+4);
 }
}
execFileSync('magick',['-size',`${pageWidth}x${pageHeight}`,'-depth','8','rgba:-','-define','png:exclude-chunks=date,time','PNG32:'+file('atlas.png')],{input:page});
const manifest={version:1,usage:'authoring_export_not_integrated_into_game_catalog',mode:'baked',sampler:'nearest',mipmaps:false,extrude_px:1,gutter_px:1,pages:[{id:'farmer2-0',path:'atlas.png',size:[pageWidth,pageHeight]}],sprites:placements.map(p=>({id:p.id,page:'farmer2-0',rect:[p.x,p.y,p.w,p.h],source_size:[W,H],trim_offset:[p.a,p.b],pivot:[48,124],source:`frames/${p.id}.png`,rotated:false})),clips};
writeFileSync(file('atlas-manifest.json'),JSON.stringify(manifest,null,2)+'\n');
const previewData=JSON.stringify({clips,frames:Object.fromEntries(Object.keys(frameStore).map(id=>[id,{equipped:`../frames/${id}.png`,base:`../frames/base/${id}.png`}]))});
writeFileSync(file('previews/animation.html'),`<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Farmer 2 표정·포즈</title><style>body{font:16px system-ui;margin:24px;background:#202026;color:#eee}main{max-width:720px;margin:auto}h1{font-size:22px}section{display:flex;gap:12px;flex-wrap:wrap;margin:16px 0}button,select,input{font:inherit}#stage{width:384px;background:#efe5cd}@media(max-width:431px){#stage{width:288px}}@media(max-width:335px){#stage{width:192px}}@media(max-width:239px){#stage{width:96px}}img{display:block;width:100%;image-rendering:pixelated}small{color:#b8b8c2}label{display:flex;gap:6px;align-items:center}output{font-variant-numeric:tabular-nums}</style><main><h1>농부 2 · 표정과 포즈</h1><section><label>동작 <select id="clip"></select></label><button id="play">일시정지</button><button id="prev">이전 키</button><button id="next">다음 키</button></section><section><label><input id="gear" type="checkbox" checked>장비</label><label><input id="repeat" type="checkbox" checked>반복 미리보기</label><label>배경 <select id="bg"><option value="#efe5cd">밝게</option><option value="#24232a">어둡게</option></select></label><label>속도 <select id="speed"><option value="0.5">0.5×</option><option value="1" selected>1×</option><option value="2">2×</option></select></label></section><div id="stage"><img id="sprite" alt="농부 애니메이션 프레임"></div><section><input id="scrub" type="range" min="0" value="0"><output id="info"></output></section><small>96×128 원본의 정수 배율 표시, 기본 4배. 전달 이벤트는 transfer 키에만 표시됩니다. 반복 미리보기는 비반복 클립도 확인용으로 반복합니다.</small></main><script>const data=${previewData};const $=id=>document.getElementById(id);let name='walk',index=0,playing=true,elapsed=0,last=0;for(const key of Object.keys(data.clips))$('clip').add(new Option(key,key));$('clip').value=name;function show(){const c=data.clips[name],f=c.frames[index];$('sprite').src=data.frames[f.sprite][$('gear').checked?'equipped':'base'];$('scrub').max=c.frames.length-1;$('scrub').value=index;$('info').textContent=(index+1)+'/'+c.frames.length+' · '+f.sprite+' · '+f.duration_ms+'ms'+(f.event?' · '+f.event:'');$('play').textContent=playing?'일시정지':'재생';}$('clip').onchange=()=>{name=$('clip').value;index=0;elapsed=0;show()};$('gear').onchange=show;$('bg').onchange=()=>{$('stage').style.background=$('bg').value};$('play').onclick=()=>{playing=!playing;show()};function step(delta){playing=false;const n=data.clips[name].frames.length;index=(index+delta+n)%n;elapsed=0;show()};$('prev').onclick=()=>step(-1);$('next').onclick=()=>step(1);$('scrub').oninput=()=>{playing=false;index=Number($('scrub').value);elapsed=0;show()};function tick(now){if(last&&playing){elapsed+=(now-last)*Number($('speed').value);const c=data.clips[name];while(elapsed>=c.frames[index].duration_ms){elapsed-=c.frames[index].duration_ms;if(index+1<c.frames.length)index++;else if(c.loop||$('repeat').checked)index=0;else{playing=false;elapsed=0;break}show()}}last=now;requestAnimationFrame(tick)}show();requestAnimationFrame(tick);</script></html>`);
console.log(`Derived 5 eye states, 4 mouths, 8 expressions, ${Object.keys(poseRecipes).length} pose recipes, ${Object.keys(clips).length} clips and ${placements.length} atlas frames (${pageWidth}x${pageHeight})`);
