import {motionFactory,writeMotion} from './mobile-poses-v14/motion.mjs';
import {pack} from './pack.mjs';
import {benchPoses} from './work-poses.mjs';
import {coherentWork} from './work-coherent.mjs';
import {neckRows} from './layer-ownership/assembly.mjs';
import fs from 'node:fs';import path from 'node:path';import {fileURLToPath} from 'node:url';
import {W,H,N,magick,empty,read,save,select,over,move,bounds,remap,skin,cloth,inside,grow,fit,tile,register} from './raster.mjs';
const ROOT=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..'),SRC=path.join(ROOT,'source/animation');
const states={neutral:['open','rest'],'talk-small':['open','small-open'],'talk-wide':['open','wide-open'],'talk-round':['open','round'],blink:['closed','rest'],happy:['happy-closed','rest'],surprised:['wide','round'],relaxed:['half','rest']};
const eyeKinds=['open','half','closed','happy-closed','wide'],mouthKinds=['rest','small-open','wide-open','round'];
const jobs=JSON.parse(fs.readFileSync(path.join(SRC,'prompts.json')));
const all=[];
for(const sex of ['female','male']){
 const dir=path.join(ROOT,sex),f=n=>path.join(dir,n),put=(n,b)=>save(f(n),b),load=n=>read(f(n));
 const colors=JSON.parse(fs.readFileSync(f('source/palette.json'))).map(h=>[...Buffer.from(h.slice(1),'hex')]);
 const native=b=>remap(b,colors),head=load('head-base.png'),originalEyes=load('eyes/open.png'),originalMouth=load('mouth/rest.png');
 const recipe=JSON.parse(fs.readFileSync(f('character.json'))),gearNames=recipe.equipped.filter(n=>n.startsWith('equipment/'));
 const originalGear=Object.fromEntries(gearNames.map(n=>[n,load(n)]));
 const foundation=load('source/foundation.png'),body=over(load('body/lower-base.png'),load('body/upper-base.png'));
 const eyes={open:originalEyes},mouths={rest:originalMouth},clips={},poses={},registrations={};
 const expressionSource=path.join(SRC,sex+'-expressions.png');
 const hb=bounds(head).box,cx=(hb[0]+hb[2])/2;
 function alignFace(b){let best={score:Infinity,out:b,shift:[0,0]};for(let dy=-4;dy<=4;dy++)for(let dx=-4;dx<=4;dx++){
  const shifted=move(b,dx,dy);let score=0;for(let y=24;y<64;y++)for(let x=35;x<84;x++){const i=(y*W+x)*4;if(inside(x,y,[54,40,81,64]))continue;const a=head.subarray(i,i+4),p=shifted.subarray(i,i+4);if(a[3]!==p[3])score+=50000;else if(a[3])for(let j=0;j<3;j++)score+=(a[j]-p[j])**2;}
  if(score<best.score)best={score,out:shifted,shift:[dx,dy]};}return best;}
 for(const [k,kind]of [[1,'small-open'],[2,'wide-open'],[3,'round'],[4,'closed'],[5,'happy-closed'],[6,'wide'],[7,'half']]){
  const r=register(tile(expressionSource,4,2,k),{height:47,foot:71,head:true});const a=alignFace(move(r.image,Math.round(cx-64),0));const b=native(a.out);
  // Registered portraits also contain hair, cheeks and a second jaw contour.
  // Only the measured feature sites belong to interchangeable expression layers.
  const eyeState=k>=4;
  const eyeSites=sex==='female'?[[58,48,69,57],[60,45,69,48],[73,45,78,57]]:[[57,45,65,56],[70,45,74,56]];
  const mouthSites=sex==='female'?{'small-open':[66,58,71,61],'wide-open':[65,57,72,62],round:[68,58,70,61]}:{'small-open':[65,58,69,60],'wide-open':[63,56,70,61],round:[65,58,68,61]};
  const sites=eyeState?eyeSites:[mouthSites[kind]];
  const part=select(b,(x,y,p)=>{const base=head.subarray((y*W+x)*4,(y*W+x)*4+4);return sites.some(roi=>inside(x,y,roi))&&skin(base)&&(p[0]<220&&p[1]<160&&p[2]<135||p[0]>145&&p[0]-p[1]<45&&p[1]-p[2]<45);});
  if(bounds(part).pixels<2)throw Error(sex+' empty expression '+kind);
  (eyeState?eyes:mouths)[kind]=part;put((eyeState?'eyes/':'mouth/')+kind+'.png',part);registrations['expression-'+kind]={...r.registration,alignment:a.shift};
 }
 function composite(parts){return over(parts.lower,parts.upper,parts.head,parts.eyes,parts.mouth);}
 const fixed={head,eyes:originalEyes,mouth:originalMouth,upper:load('body/upper-base.png'),lower:load('body/lower-base.png')};
 function splitHeadFeatures(full,k){
  const t=tile(path.join(SRC,'angle-heads-blank.png'),4,2,k+(sex==='male'?4:0)),blankHead=empty();
  for(let y=4;y<80;y++)for(let x=16;x<112;x++){const sx=Math.floor((x-16+.5)*t.w/96),sy=Math.floor((y-4+.5)*t.h/76),i=(sy*t.w+sx)*4;if(t.b[i+3])t.b.copy(blankHead,(y*W+x)*4,i,i+4);}
  const clean=native(blankHead),base=Buffer.from(full),e=empty(),m=empty();
  for(let y=39;y<68;y++)for(let x=49;x<92;x++){const i=(y*W+x)*4,p=full.subarray(i,i+4),q=clean.subarray(i,i+4);if(!p[3]||!skin(q))continue;
   const delta=Math.abs(p[0]-q[0])+Math.abs(p[1]-q[1])+Math.abs(p[2]-q[2]);
   const pigment=p[0]<220&&p[1]<150&&p[2]<125||p[0]>145&&p[0]-p[1]<45&&p[1]-p[2]<45;
   if(pigment&&delta>24){full.copy(y<59?e:m,i,i,i+4);clean.copy(base,i,i,i+4);}
  }return{head:base,eyes:e,mouth:m};
 }
 function storePose(id,parts,{gear=originalGear,front=load('body/arms-front.png'),prop=empty(),source='',facing='right_3_4',sockets={},featuresSeparated=false}={}){
  const angleIndex=id==='nod-down'?0:id==='turn-toward'?1:id==='turn-away'?2:id==='front-neutral'||id.startsWith('researching-')?3:null;
  if(angleIndex!==null&&!featuresSeparated)parts={...parts,...splitHeadFeatures(parts.head,angleIndex)};
  if(!featuresSeparated&&id.startsWith('researching-')&&!id.endsWith('-4'))parts.eyes=select(frontWorkingEyes,(x,y)=>skin(parts.head.subarray((y*W+x)*4,(y*W+x)*4+4)));
  const p='poses/'+id;const base=composite(parts),equipped=over(base,...Object.entries(gear).filter(([n])=>!n.includes('gloves')&&!n.includes('headband')&&!n.includes('hair-tie')).map(([,b])=>b),prop,front,gear['equipment/gloves.png']||empty(),...Object.entries(gear).filter(([n])=>n.includes('headband')||n.includes('hair-tie')).map(([,b])=>b));
  const layers={'head-base.png':parts.head,'eyes/open.png':parts.eyes,'mouth/rest.png':parts.mouth,'body/upper-base.png':parts.upper,'body/lower-base.png':parts.lower,'body/arms-front.png':front,'props/held.png':prop,...gear};
  for(const[n,b]of Object.entries(layers))put(p+'/'+n,b);
  put(p+'/foundation.png',over(parts.lower,parts.upper,parts.head));put(p+'/base.png',base);put(p+'/composed.png',equipped);put('frames/base/'+id+'.png',base);put('frames/'+id+'.png',equipped);
  poses[id]={canvas:[W,H],pivot:[64,140],facing,source,sockets,layers:Object.keys(layers).map(n=>p+'/'+n),base:'frames/base/'+id+'.png',composed:'frames/'+id+'.png'};
 }
 for(const[id,[e,m]]of Object.entries(states)){
  const parts={...fixed,eyes:eyes[e],mouth:mouths[m]};storePose(id,parts);put('expressions/base/'+id+'.png',composite(parts));put('expressions/'+id+'.png',load('frames/'+id+'.png'));
 }
 const clip=(name,ids,times,loop=false,extra={})=>{clips[name]={loop,frames:ids.map((sprite,i)=>({sprite,duration_ms:times[i%times.length]})),...extra};};
 clip('idle',['neutral'],[1000],true);clip('blink',['neutral','relaxed','blink','relaxed','neutral'],[800,40,80,40,800]);
 clip('talk',['neutral','talk-small','talk-wide','talk-small','talk-round','neutral'],[120,100,100,100,100,120],true);
 clip('talk_blink',['neutral','talk-small','talk-wide','blink','talk-round','neutral'],[120,100,100,80,100,120],true);
 clip('happy',['happy'],[1000]);clip('surprised',['surprised'],[1000]);clip('relaxed',['relaxed'],[1000]);
 const lockedSources={};
 function sourcePose(job,k,height=117){const j=jobs.find(j=>j.id===sex+'-'+(sex==='male'&&job==='smith'?'smith-compact':job)),p=path.join(SRC,j.id+'.png');if(!lockedSources[job])lockedSources[job]=register(tile(p,j.cols||4,j.rows||2,0),{height}).registration;const t=tile(p,j.cols||4,j.rows||2,k),measured=register(t,{height}).registration,lock={...lockedSources[job],sample_origin:[measured.sample_origin[0]+64*measured.sample_step-64*lockedSources[job].sample_step,lockedSources[job].sample_origin[1]]};const r=register(t,{height,locked:lock,rightFit:job==='smith'&&sex==='male'?0.7:1});registrations[job+'-'+k]=r.registration;return job==='smith'?remap(r.image,[...colors,...['ff5a12','ffad22','ffe679','ad3020','636570','969a9c'].map(h=>[...Buffer.from(h,'hex')])]):native(r.image);}
 function sourceHead(b){return select(b,(x,y,p)=>y<65||y<68&&!cloth(p));}
 function split(raw,{keepHead=true}={}){const shoulder=grow(select(raw,(x,y,p)=>y>=60&&y<67&&cloth(p)));const torso=select(raw,(x,y,p)=>y>=72||y>=60&&shoulder[(y*W+x)*4+3]||y>=63&&inside(x,y,[53,63,80,74])&&skin(p));const skinArea=grow(select(torso,(x,y,p)=>y>=78&&y<119&&skin(p)),1);return {head:keepHead?head:sourceHead(raw),eyes:keepHead?originalEyes:empty(),mouth:keepHead?originalMouth:empty(),upper:select(torso,(x,y)=>y<90||skinArea[(y*W+x)*4+3]),lower:select(torso,(x,y)=>y>=90&&!skinArea[(y*W+x)*4+3])};}
 function gearFor(raw,{kind='walk',facing='right_3_4',armRegion=null}={}){
  const armSkin=select(raw,(x,y,p)=>(armRegion?armRegion(x,y):(y>=78&&y<118||y>=34&&y<78&&!head[(y*W+x)*4+3]&&!(x<79&&y>61)))&&skin(p));const nearby=grow(armSkin,1);
  const front=select(raw,(x,y,p)=>nearby[(y*W+x)*4+3]&&(skin(p)||p[0]<110&&p[1]<90&&p[2]<75));
  const gloves=empty(),handPoints=[];
  const seen=new Set(),components=[];
  for(let y=38;y<118;y++)for(let x=20;x<120;x++){const key=y*W+x;if(seen.has(key)||!armSkin[key*4+3])continue;const q=[[x,y]],points=[];seen.add(key);for(let z=0;z<q.length;z++){const [xx,yy]=q[z];points.push([xx,yy]);for(const[dx,dy]of [[1,0],[-1,0],[0,1],[0,-1]]){const nx=xx+dx,ny=yy+dy,k=ny*W+nx;if(nx<20||nx>=120||ny<38||ny>=118||seen.has(k)||!armSkin[k*4+3])continue;seen.add(k);q.push([nx,ny]);}}if(points.length>=6)components.push(points);}
  components.sort((a,b)=>b.length-a.length);
  const chosen=components.slice(0,2).sort((a,b)=>a.reduce((s,p)=>s+p[0],0)/a.length-b.reduce((s,p)=>s+p[0],0)/b.length);
  for(let n=0;n<chosen.length;n++){const points=chosen[n],shoulder=n===0?[48,74]:[79,74];const tip=points.reduce((a,b)=>(a[0]-shoulder[0])**2+(a[1]-shoulder[1])**2>(b[0]-shoulder[0])**2+(b[1]-shoulder[1])**2?a:b);
   const distal=points.filter(([x,y])=>Math.hypot(x-tip[0],y-tip[1])<=5);const xs=distal.map(p=>p[0]),ys=distal.map(p=>p[1]);const rect=[Math.min(...xs)-1,Math.min(...ys)-1,Math.max(...xs)+2,Math.max(...ys)+2];
   const src=select(originalGear['equipment/gloves.png'],x=>n===0?x<70:x>=70),fitted=fit(src,rect);for(let i=0;i<N;i+=4)if(fitted[i+3])fitted.copy(gloves,i,i,i+4);handPoints.push({side:n===0?'right':'left',point:tip,rect});
  }
  return {gear:{...originalGear,'equipment/gloves.png':gloves},front,sockets:{hands:handPoints}};
 }
 // Head changes are complete generated heads; facial features remain coherent
 // with that source rather than independently resized neutral eyes.
 for(const[k,id]of [[0,'nod-down'],[1,'turn-toward'],[2,'turn-away']]){
  const raw=sourcePose('gestures',k),h=sourceHead(raw),parts={...fixed,head:h,eyes:empty(),mouth:empty()};
  storePose(id,parts,{source:'source/animation/'+sex+'-gestures.png',gear:originalGear});
 }
 storePose('nod-mid',{...fixed,head:move(head,0,1),eyes:move(originalEyes,0,1),mouth:move(originalMouth,0,1)},{gear:{...originalGear,[sex==='female'?'equipment/hair-tie.png':'equipment/headband.png']:move(originalGear[sex==='female'?'equipment/hair-tie.png':'equipment/headband.png'],0,1)}});
 clip('nod',['neutral','nod-mid','nod-down','nod-mid','neutral'],[120,100,140,100,120]);clip('shake_head',['neutral','turn-toward','neutral','turn-away','neutral'],[100,140,100,140,100]);
 const mobileMotion=motionFactory({dir,sex,source:SRC,colors});
 for(const[k,id]of [[3,'give-half'],[4,'give-full'],[5,'give-release']]){const raw=sourcePose('gestures',k);poses[id]=writeMotion(dir,id,mobileMotion(raw,id,'source/animation/'+sex+'-gestures.png'));}
 clip('give',['neutral','give-half','give-full','give-release','give-half','neutral'],[120,120,180,120,120,120],false,{events:[{frame:3,event:'transfer'}]});
 const walkKeys=[['walk-opposite',0],['walk-opposite',1],['walk',2],['walk',0],['walk',1],['walk-opposite',2]];
 for(let k=0;k<6;k++){const[j,n]=walkKeys[k],raw=sourcePose(j,n),id='walk-'+k;poses[id]=writeMotion(dir,id,mobileMotion(raw,id,'source/animation/'+sex+'-'+j+'.png'));}
 clip('walk',Array.from({length:6},(_,k)=>'walk-'+k),[120],true);
 // New directions use their own coherent heads and never receive 3/4 eyes.
 const dhead=k=>native(register(tile(path.join(SRC,'direction-heads.png'),2,2,k),{height:41,foot:65,head:true}).image);
 const benchHeads={front:dhead(sex==='female'?0:2),back:dhead(sex==='female'?1:3)};
 const halfFront=native(register(tile(path.join(SRC,'front-half.png'),2,2,sex==='female'?0:2),{height:41,foot:65,head:true}).image);
 const frontWorkingEyes=select(halfFront,(x,y,p)=>inside(x,y,[48,40,82,59])&&(p[0]<220&&p[1]<160&&p[2]<135||p[0]>145&&p[0]-p[1]<45&&p[1]-p[2]<45));
 put('eyes/front-half.png',frontWorkingEyes);
 function benchBody(k,direction){const raw=sourcePose('bench',k),shifted=move(select(raw,(x,y)=>y>=64),0,-4),standing=sourcePose('directions',direction==='front'?0:1);const pant=grow(select(standing,(x,y,p)=>y>=90&&y<125&&p[0]<165&&p[1]>45&&p[1]>p[2]*1.05&&p[0]<p[1]*1.5),2);return over(select(standing,(x,y)=>y>=120||y>=90&&pant[(y*W+x)*4+3]),select(shifted,(x,y)=>y<90));}
 for(const[direction,k]of [['front',0],['back',6]]){
  const raw=benchBody(k,direction),parts=split(raw,{keepHead:false});parts.head=benchHeads[direction];storePose(direction+'-neutral',parts,{gear:{},front:empty(),source:'source/animation/'+sex+'-bench.png',facing:direction});
 }
 const frontBlinkRaw=native(register(tile(path.join(SRC,'front-blink.png'),2,2,sex==='female'?0:2),{height:41,foot:65,head:true}).image);
 const frontParts=splitHeadFeatures(benchHeads.front,3),closed=select(frontBlinkRaw,(x,y,p)=>inside(x,y,[48,40,82,59])&&skin(frontParts.head.subarray((y*W+x)*4,(y*W+x)*4+4))&&p[0]<200&&p[1]<145&&p[2]<130);
 const fb=split(benchBody(0,'front'),{keepHead:false});storePose('front-blink',{...fb,...frontParts,eyes:closed},{gear:{},front:empty(),facing:'front'});
 clip('front_blink',['front-neutral','front-blink','front-neutral'],[900,80,900]);
 clip('hammering',Array.from({length:6},(_,k)=>'hammering-'+k),[140,180,80,80,120,160],true,{station_id:'anvil',facing:'right_3_4',loop_range:[1,5],events:[{frame:3,event:'hammer_contact'}]});
 clip('quenching',Array.from({length:6},(_,k)=>'quenching-'+k),[160,140,100,400,180,160],false,{station_id:'quenching-tub',facing:'right_3_4',events:[{frame:2,event:'water_enter'},{frame:4,event:'water_exit'}]});
 clip('forging',Array.from({length:6},(_,k)=>'forging-'+k),[160,180,300,300,180,160],false,{station_id:'forge',facing:'right_3_4',loop_range:[2,3],events:[{frame:2,event:'heat_enter'},{frame:4,event:'heat_exit'}]});
 const coherentBench=benchPoses(SRC,sex,colors);
 for(const [id,n]of [['front-neutral',0],['back-neutral',4]]){const p=coherentBench[n];storePose(id,p.parts,{...p,featuresSeparated:true});}
 {const p=coherentBench[0];storePose('front-blink',{...p.parts,eyes:p.closed},{...p,featuresSeparated:true});}
 clip('researching',Array.from({length:6},(_,k)=>'researching-'+k),[220,120,200,200,360,180],false,{station_id:'research-table',facing:'front',loop_range:[2,3]});
 clip('salvaging',Array.from({length:6},(_,k)=>'salvaging-'+k),[160,160,180,180,200,160],false,{station_id:'salvage-table',facing:'back',loop_range:[2,3],events:[{frame:4,event:'parts_place'}]});
 const coherentWorkPoses=coherentWork({dir,sex,source:SRC,colors});
 Object.assign(poses,coherentWorkPoses);
 const animation={version:1,common_motion_revision:'collar-v15',role:'player_blacksmith',canvas:[W,H],pivot:[64,140],sampler:'nearest',mipmaps:false,expression_recipes:states,eye_states:eyeKinds,mouth_states:mouthKinds,channels:{blink:'eyes',talk:'mouth',surprised_suspends_blink:true},direction_baselines:{right_3_4:'neutral',front:'front-neutral',back:'back-neutral'},poses,clips,registrations,walking:{near_side:'anatomical_right',far_side:'anatomical_left',keys:[{left_leg:'front',right_arm:'front',right_leg:'back',left_arm:'back'},{support:'left',arms:'transition'},{passing:'right',arms:'transition_toward_left_forward'},{right_leg:'front',left_arm:'front',left_leg:'back',right_arm:'back'},{support:'right',arms:'transition'},{passing:'left',arms:'transition_toward_right_forward'}]},runtime_integration:false,generator:'built-in image_gen',source_prompts:'../source/animation/prompts.json',work_source_prompts:'../source/animation/hip-pivot/prompts.json'};
 const stationGeometry={anvil:[18,8],'quenching-tub':[29,18],forge:[48,76],'research-table':[64,26],'salvage-table':[64,39]};
 animation.head_ownership={scope:'visible-face work poses only; approved neutral unchanged',removed_neck_collar_rows:neckRows[sex],coordinates:'root native 128x160 before head translation',eyes_mouth:'unchanged approved root pixels'};
 animation.workshop={world_canvas:[256,192],character_origin:[32,16],stations:{}};
 for(const action of ['hammering','quenching','forging','researching','salvaging']){
  const c=clips[action],fr=action==='hammering'?3:action==='quenching'?3:2,p=load('poses/'+action+'-'+fr+'/props/held.png'),hot=[];
  for(let y=0;y<H;y++)for(let x=0;x<W;x++){const i=(y*W+x)*4;if(p[i+3]&&p[i]>240&&p[i+1]>65&&p[i+1]<235&&p[i+2]<110)hot.push([x,y]);}
  let point=action==='researching'?[64,83]:action==='salvaging'?[64,78]:[108,94];
  if(hot.length){const xs=hot.map(p=>p[0]),ys=hot.map(p=>p[1]);point=action==='hammering'?[Math.min(...xs)+1,Math.round(ys.reduce((a,b)=>a+b)/ys.length)]:action==='quenching'?[Math.round(xs.reduce((a,b)=>a+b)/xs.length),Math.max(...ys)-8]:[Math.max(...xs)-2,Math.round(ys.reduce((a,b)=>a+b)/ys.length)];}
  const station=c.station_id,socket=stationGeometry[station],origin=[point[0]+32-socket[0],point[1]+16-socket[1]];
  const placement={station_id:station,origin,contact_points:{character:point,station:socket,world:[point[0]+32,point[1]+16]},occlusion_order:action==='researching'?['character','station','front_hands']:action==='salvaging'||action==='forging'?['station','character']:['station_back','station_water_fire','character','held_tools','station_front'],held_tool_owner:{hammer:'anatomical_right',tongs:'anatomical_left'},events:c.events||[]};
  animation.workshop.stations[action]=placement;
  if(action==='quenching'){
   placement.submerged_tool_clip={station_rect:[6,18,59,48],source:'held_tools_only'};
   for(const fr of c.frames){
    const id=fr.sprite,tool=load('poses/'+id+'/props/held.png');
    for(const mode of ['base/','']){
     const actor=load('frames/'+mode+id+'.png');
     const clipped=select(actor,(x,y)=>{const sx=x+32-origin[0],sy=y+16-origin[1];return !tool[(y*W+x)*4+3]||!inside(sx,sy,[6,18,59,48]);});
     put('frames/scene-'+(mode?'base/':'')+id+'.png',clipped);
    }
   }
  }
 }
 fs.writeFileSync(f('animation.json'),JSON.stringify(animation,null,2)+'\n');
 pack(dir,animation);
 fs.mkdirSync(f('previews'),{recursive:true});
 if(!process.argv.includes('--no-previews')){
 for(const[id,c]of Object.entries(clips))for(const mode of ['base','equipped']){
  const args=['-dispose','background','-background','#eee5cd'];for(const fr of c.frames)args.push('-delay',Math.round(fr.duration_ms/10),'(',f('frames/'+(mode==='base'?'base/':'')+fr.sprite+'.png'),'-background','#eee5cd','-alpha','remove','-filter','point','-resize','400%',')');
  magick([...args,'-loop',c.loop?'0':'1','-strip',f('previews/'+id+(mode==='base'?'-base':'')+'.gif')]);
  if(id==='walk')magick([...args.map((v,i)=>i&&args[i-1]==='-delay'?Number(v)*2:v),'-loop','0','-strip',f('previews/walk'+(mode==='base'?'-base':'')+'-slow.gif')]);
 }
 magick(['montage',...Object.keys(states).flatMap(id=>['-label',id,f('frames/'+id+'.png')]),'-font','Arial','-pointsize','14','-background','#eee5cd','-alpha','remove','-filter','point','-geometry','384x480+6+6','-tile','4x2','-strip',f('previews/expressions-8.png')]);
 const combos=[];for(const e of eyeKinds)for(const m of mouthKinds){const n='qa/combination-'+e+'-'+m+'.png';put(n,over(foundation,eyes[e],mouths[m]));combos.push(f(n));}magick(['montage',...combos,'-background','#eee5cd','-alpha','remove','-filter','point','-geometry','256x320+4+4','-tile','4x5','-strip',f('previews/expression-combinations-20.png')]);
 magick(['montage',...Array.from({length:6},(_,k)=>f('frames/base/walk-'+k+'.png')),'-background','#eee5cd','-alpha','remove','-filter','point','-geometry','384x480+6+6','-tile','3x2','-strip',f('previews/walk-6-base.png')]);
 }
 all.push({id:sex,frames:Object.keys(poses).length,clips:Object.keys(clips).length});
}
console.log(JSON.stringify(all));
