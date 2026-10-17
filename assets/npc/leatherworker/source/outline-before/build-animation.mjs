import {writeFileSync,mkdirSync,readFileSync} from 'node:fs';
import {W,H,N,blank,file,run,read,write,over,select,move,bounds,candidate,grow,skin,sheet} from './animation-lib.mjs';

const head=read('head-base.png'),upper=read('body/upper-base.png'),lower=read('body/lower-base.png');
const gear=Object.fromEntries(['apron','scarf','belt-tools','gloves','hairpin'].map(n=>[n,read(`equipment/${n}.png`)]));
const hide=read('props/leather-hide.png');
const eyes={open:read('eyes/open.png')},mouth={rest:read('mouth/rest.png')};
const feature=p=>p[0]<185&&p[1]<125&&p[2]<110||p[0]>205&&p[1]>205&&p[2]>180;
const inBox=(x,y,b)=>x>=b[0]&&y>=b[1]&&x<b[2]&&y<b[3];
const eyeBoxes=[[39,24,50,37],[53,23,59,37]],mouthBox=[44,36,54,44];
for(const id of ['half','closed','happy-closed','wide']){
 const src=candidate(`source/expressions/eye-${id}.png`,{cw:64,ch:64,ox:12});
 eyes[id]=select(src,(x,y,p)=>eyeBoxes.some(b=>inBox(x,y,b))&&skin(head.subarray((y*W+x)*4,(y*W+x)*4+4))&&feature(p));
 write(`eyes/${id}.png`,eyes[id]);
}
for(const id of ['small-open','wide-open','round']){
 const src=candidate(`source/expressions/mouth-${id}.png`,{cw:64,ch:64,ox:12});
 mouth[id]=select(src,(x,y,p)=>inBox(x,y,mouthBox)&&skin(head.subarray((y*W+x)*4,(y*W+x)*4+4))&&feature(p));
 write(`mouth/${id}.png`,mouth[id]);
}
const eyeMask=over(...Object.values(eyes)),mouthMask=over(...Object.values(mouth));
write('source/expression-eye-mask.png',eyeMask);write('source/expression-mouth-mask.png',mouthMask);
const recipes={neutral:['open','rest'],'talk-small':['open','small-open'],'talk-wide':['open','wide-open'],'talk-round':['open','round'],blink:['closed','rest'],happy:['happy-closed','rest'],surprised:['wide','round'],relaxed:['half','rest']};
const foundation=over(lower,upper,head),fixedGear=['apron','scarf','belt-tools','gloves','hairpin'].map(n=>gear[n]);
const frameStore={},baseStore={},poseRecipes={};
function frame(id,b,full){frameStore[id]=full;baseStore[id]=b;write(`frames/${id}.png`,full);write(`frames/base/${id}.png`,b);}
for(const[id,[e,m]]of Object.entries(recipes)){
 const b=over(foundation,eyes[e],mouth[m]),full=over(b,...fixedGear,hide);
 write(`expressions/${id}.png`,full);write(`expressions/base/${id}.png`,b);frame(id,b,full);
}
for(const[e,a]of Object.entries(eyes))for(const[m,b]of Object.entries(mouth))write(`previews/combinations/${e}--${m}.png`,over(foundation,a,b,...fixedGear,hide));
writeFileSync(file('expressions/recipes.json'),JSON.stringify({canvas:[W,H],eyebrow_owner:'eyes',eyes:Object.fromEntries(Object.keys(eyes).map(n=>[n,`eyes/${n}.png`])),mouth:Object.fromEntries(Object.keys(mouth).map(n=>[n,`mouth/${n}.png`])),recipes:Object.fromEntries(Object.entries(recipes).map(([id,[e,m]])=>[id,{eyes:e,mouth:m}]))},null,2)+'\n');
sheet('expressions-8.png',Object.keys(recipes).map(n=>`expressions/${n}.png`),4);
sheet('expressions-20.png',Object.keys(eyes).flatMap(e=>Object.keys(mouth).map(m=>`previews/combinations/${e}--${m}.png`)),4,3);
const headConfigs=[
 {id:'nod-down',eye:[[39,26,51,37],[53,25,61,37]],lip:[47,39,55,44],pin:[2,2]},
 {id:'turn-toward',eye:[[38,24,49,37],[52,23,62,37]],lip:[44,38,54,43],pin:[1,0]},
 {id:'turn-away',eye:[[44,23,57,36],[60,23,65,36]],lip:[51,37,60,42],pin:[5,-1]}
];
const heads={};
for(const[k,c]of headConfigs.entries()){
 const raw=candidate(`source/poses/${c.id}.png`,{cw:64,ch:64,ox:12});
 const h=candidate('source/poses/head-blank-sheet.png',{cw:64,ch:64,ox:12,cols:3,index:k});
 const e=select(raw,(x,y,p)=>c.eye.some(b=>inBox(x,y,b))&&skin(h.subarray((y*W+x)*4,(y*W+x)*4+4))&&feature(p));
 const m=select(raw,(x,y,p)=>inBox(x,y,c.lip)&&skin(h.subarray((y*W+x)*4,(y*W+x)*4+4))&&feature(p));
 heads[c.id]={h,e,m,pin:move(gear.hairpin,...c.pin)};
}
heads['nod-mid']=Object.fromEntries(Object.entries(heads['nod-down']).map(([n,b])=>[n,move(b,0,-1)]));
for(const[id,a]of Object.entries(heads)){
 const b=over(lower,upper,a.h,a.e,a.m),full=over(b,gear.apron,gear.scarf,gear['belt-tools'],gear.gloves,a.pin,hide);
 const parts={'body/lower-base.png':lower,'body/upper-base.png':upper,'head-base.png':a.h,'eyes/open.png':a.e,'mouth/rest.png':a.m,'equipment/apron.png':gear.apron,'equipment/scarf.png':gear.scarf,'equipment/belt-tools.png':gear['belt-tools'],'equipment/gloves.png':gear.gloves,'equipment/hairpin.png':a.pin,'props/leather-hide.png':hide};
 savePose(id,parts,b,full,{neck:[45,45],waist:[47,71],eyes:center(a.e),mouth:center(a.m),right_hand:[36,71],left_hand:[75,58]},{head_source:`source/poses/${id==='nod-mid'?'nod-down':id}.png`,facial_states:{eyes:['open'],mouth:['rest']}});
}
const configs=[
 {id:'walk-0',hands:[[54,58,74,73],[17,63,31,78]],sockets:[[66,65],[23,72]]},
 {id:'walk-1',hands:[[49,61,69,76],[18,65,32,81]],sockets:[[61,68],[25,75]]},
 {id:'walk-2',hands:[[18,69,34,85],[59,62,77,79]],sockets:[[26,78],[69,71]]},
 {id:'walk-3',hands:[[14,70,30,86],[62,60,83,76]],sockets:[[22,79],[76,68]]},
 {id:'walk-4',hands:[[14,70,31,87],[59,60,81,77]],sockets:[[22,79],[72,69]]},
 {id:'walk-5',hands:[[17,71,34,88],[59,65,77,81]],sockets:[[25,80],[68,75]]},
 {id:'give-mid',hands:[[43,66,66,80],[67,50,87,69]],sockets:[[55,73],[80,60]]},
 {id:'give-extended',hands:[[54,65,79,81],[70,49,90,69]],sockets:[[68,73],[83,59]]}
];
const plan=JSON.parse(readFileSync(file('source/animation-plan.json')));
// Register neck and waist independently, keeping the same source body intact.
// This closes the neck seam while the soles stay at the shared ground line.
function bodyRegister(src,dy,hd){
 const out=blank();for(let y=0;y<H;y++){
  const sy=y<45+hd?y-hd:y<71+dy?Math.floor(45+(y-(45+hd)+.5)*26/(26+dy-hd)):y-dy;
  if(sy>=0&&sy<H)src.copy(out,y*W*4,sy*W*4,(sy+1)*W*4);
 }return out;
}
function bodySocketY(y,dy,hd){return Math.round(y<45?y+hd:y<71?45+hd+(y-45)*(26+dy-hd)/26:y+dy);}
for(const[k,c]of configs.entries()){
 const raw=candidate(`source/poses/${c.id}.png`);
 const gloved=candidate(`source/poses/${k<6?'walk':'give'}-gloves-sheet.png`,{cols:k<6?3:2,rows:k<6?2:1,index:k<6?k:k-6});
 const body=select(raw,(x,y)=>y>=45&&!(y<47&&(x<36||x>51))&&!(y<49&&(x<33||x>59)));
 const groundOffset=124-(bounds(body)[3]-1);
 const armSkin=select(body,(x,y,p)=>y>=54&&y<93&&skin(p));
 const armArea=grow(armSkin,1);
 const up=select(body,(x,y)=>y<71||armArea[(y*W+x)*4+3]>0),lo=select(body,(x,y)=>y>=71&&!armArea[(y*W+x)*4+3]);
 const front=select(body,(x,y)=>armArea[(y*W+x)*4+3]>0);
 const handArea=grow(select(body,(x,y,p)=>c.hands.some(b=>inBox(x,y,b))&&skin(p)),1);
 const gloves=select(gloved,(x,y,p)=>{
  const exposedSkin=p[0]>200&&p[1]>110&&p[2]>p[1]*.5&&p[0]>p[1]*1.22;
  const cream=p[0]>200&&p[1]>185&&p[2]>145;
  return c.hands.some(b=>inBox(x,y,b))&&handArea[(y*W+x)*4+3]>0&&body[(y*W+x)*4+3]>0&&!exposedSkin&&!cream;
 });
 const dy=groundOffset,hd=k<6?[1,2,0,1,2,0][k]:0;
 const h=move(head,0,hd),e=move(eyes.open,0,hd),m=move(mouth.rest,0,hd);
 const u=bodyRegister(up,dy,hd),l=bodyRegister(lo,dy,hd),f=bodyRegister(front,dy,hd),g=bodyRegister(gloves,dy,hd);
 const a=bodyRegister(gear.apron,dy,hd),s=bodyRegister(gear.scarf,dy,hd),t=bodyRegister(gear['belt-tools'],dy,hd),pin=move(gear.hairpin,0,hd);
 const b=over(l,u,h,e,m),equipped=over(b,a,s,t,f,g,pin);
 const hand=[c.sockets[1][0],bodySocketY(c.sockets[1][1],dy,hd)];
 const prop=k>=6?move(hide,hand[0]-76,hand[1]-58):blank();
 const full=over(equipped,prop);
 const parts={'body/lower-base.png':l,'body/upper-base.png':u,'head-base.png':h,'eyes/open.png':e,'mouth/rest.png':m,'equipment/apron.png':a,'equipment/scarf.png':s,'equipment/belt-tools.png':t,'body/arms-front.png':f,'equipment/gloves.png':g,'equipment/hairpin.png':pin};
 if(k>=6)parts['props/leather-hide.png']=prop;
 const extra={source:`source/poses/${c.id}.png`,body_registration:{neck:[45,45+hd],waist:[71,71+dy],lower_translation:[0,dy]},head_translation_px:[0,hd],facial_states:{eyes:Object.keys(eyes),mouth:Object.keys(mouth)},limb_identity:{near:'anatomical right',far:'anatomical left',gold_buckled_boot:'right'},held_hide:k>=6};
 if(k<6)extra.walk_phase=plan.walk.phases[k].label;
 savePose(c.id,parts,b,full,{neck:[45,45+hd],waist:[47,71+dy],eyes:[48,31+hd],mouth:[48,38+hd],left_hand:hand,right_hand:[c.sockets[0][0],bodySocketY(c.sockets[0][1],dy,hd)]},extra);
 write(`poses/${c.id}/complete-underlayer.png`,over(l,u,h));
 write(`poses/${c.id}/source-body.png`,bodyRegister(body,dy,hd));
 if(k>=6)frame(`${c.id}-empty`,b,equipped);
}
function center(b){const[a,c,d,e]=bounds(b);return[Math.round((a+d-1)/2),Math.round((c+e-1)/2)];}
function savePose(id,parts,b,full,sockets,extra){
 for(const[n,a]of Object.entries(parts))write(`poses/${id}/${n}`,a);
 write(`poses/${id}/base.png`,b);write(`poses/${id}/composed.png`,full);
 write(`poses/${id}/complete-underlayer.png`,over(parts['body/lower-base.png'],parts['body/upper-base.png'],parts['head-base.png']));
 poseRecipes[id]={layers:Object.keys(parts).map(n=>`poses/${id}/${n}`),sockets_px:sockets,...extra};frame(id,b,full);
}
const noProp=over(foundation,eyes.open,mouth.rest,...fixedGear);
frame('neutral-no-prop',read('base-neutral.png'),noProp);
for(const[id,e,m]of [['blink-half','half','rest'],['talk-blink','closed','small-open']]){const b=over(foundation,eyes[e],mouth[m]);frame(id,b,over(b,...fixedGear,hide));}
const clip=(ids,times,loop=false)=>({loop,frames:ids.map((sprite,k)=>({sprite,duration_ms:times[k]}))});
const clips={
 idle:clip(['neutral','blink-half','blink','blink-half','neutral'],[1600,40,80,40,1200],true),
 talk:clip(['neutral','talk-small','talk-wide','talk-small','talk-round','neutral'],[140,100,100,100,100,140],true),
 blink:clip(['neutral','blink-half','blink','blink-half','neutral'],[300,40,80,40,300]),
 happy:clip(['happy','neutral'],[700,140]),surprised:clip(['surprised','neutral'],[600,140]),relaxed:clip(['relaxed','neutral'],[700,140]),
 nod:clip(['neutral','nod-mid','nod-down','nod-mid','neutral'],[120,100,140,100,120]),
 shake_head:clip(['neutral','turn-toward','neutral','turn-away','neutral'],[100,140,100,140,100]),
 give:clip(['neutral','give-mid','give-extended','give-extended-empty','give-mid-empty','neutral-no-prop'],[120,120,180,120,120,120]),
 walk:clip(Array.from({length:6},(_,k)=>`walk-${k}`),Array(6).fill(120),true),
 talk_blink:clip(['talk-small','talk-blink','talk-small','talk-wide','talk-round'],[240,100,140,140,140],true)
};
clips.give.frames[3].event='transfer';
const channels={eyes:'Independent blink except head turns lock open, and surprised locks wide',mouth:'Independent talk on neutral/walk/give; rotated heads currently rest only',priority:'Explicit pose/expression locks override automatic blink/talk'};
writeFileSync(file('animation.json'),JSON.stringify({version:1,canvas:[W,H],foot:[48,124],duration_unit:'ms',root_motion:false,channels,frames:Object.fromEntries(Object.keys(frameStore).map(id=>[id,{equipped:`frames/${id}.png`,base:`frames/base/${id}.png`}])),clips},null,2)+'\n');
writeFileSync(file('poses/recipes.json'),JSON.stringify({canvas:[W,H],poses:poseRecipes},null,2)+'\n');
sheet('head-poses.png',['neutral.png','poses/nod-mid/composed.png','poses/nod-down/composed.png','poses/turn-toward/composed.png','poses/turn-away/composed.png'],5,3);
sheet('give-poses.png',clips.give.frames.map(f=>`frames/${f.sprite}.png`),3,3);
sheet('walk-6.png',Array.from({length:6},(_,k)=>`poses/walk-${k}/composed.png`),3);
sheet('walk-6-no-equipment.png',Array.from({length:6},(_,k)=>`poses/walk-${k}/base.png`),3);
for(const id of ['talk','blink','nod','shake_head','give','walk','talk_blink']){
 for(const mode of ['equipped','base'])for(const speed of id==='walk'?[1,.25]:[1]){
  const args=[];for(const f of clips[id].frames)args.push('-delay',String(Math.round(f.duration_ms/10/speed)),file(`frames/${mode==='base'?'base/':''}${f.sprite}.png`));
  run([...args,'-filter','point','-resize','400%','-background','#eee5cd','-alpha','remove','-dither','None','-loop','0','-strip',file(`previews/${id}${mode==='base'?'-base':''}${speed===.25?'-slow':''}.gif`)]);
 }
}
// Packing and preview UI follow below; frame payloads always come from layers.
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
run(['-size',`${pageWidth}x${pageHeight}`,'-depth','8','rgba:-','-strip','PNG32:'+file('atlas.png')],page);
const manifest={version:1,usage:'authoring_export_not_integrated_into_game_catalog',mode:'baked',sampler:'nearest',mipmaps:false,extrude_px:1,gutter_px:1,pages:[{id:'leatherworker-0',path:'atlas.png',size:[pageWidth,pageHeight]}],sprites:placements.map(p=>({id:p.id,page:'leatherworker-0',rect:[p.x,p.y,p.w,p.h],source_size:[W,H],trim_offset:[p.a,p.b],pivot:[48,124],source:`frames/${p.id}.png`,rotated:false})),clips};
writeFileSync(file('atlas-manifest.json'),JSON.stringify(manifest,null,2)+'\n');
const previewData=JSON.stringify({clips,frames:Object.fromEntries(Object.keys(frameStore).map(id=>[id,{equipped:`../frames/${id}.png`,base:`../frames/base/${id}.png`}]))});
writeFileSync(file('previews/animation.html'),`<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Leatherworker 표정·포즈</title><style>body{font:16px system-ui;margin:24px;background:#202026;color:#eee}main{max-width:720px;margin:auto}h1{font-size:22px}section{display:flex;gap:12px;flex-wrap:wrap;margin:16px 0}button,select,input{font:inherit}#stage{width:384px;background:#efe5cd}@media(max-width:431px){#stage{width:288px}}@media(max-width:335px){#stage{width:192px}}@media(max-width:239px){#stage{width:96px}}img{display:block;width:100%;image-rendering:pixelated}small{color:#b8b8c2}label{display:flex;gap:6px;align-items:center}output{font-variant-numeric:tabular-nums}</style><main><h1>가죽세공사 · 표정과 포즈</h1><section><label>동작 <select id="clip"></select></label><button id="play">일시정지</button><button id="prev">이전 키</button><button id="next">다음 키</button></section><section><label><input id="gear" type="checkbox" checked>장비</label><label><input id="repeat" type="checkbox" checked>반복 미리보기</label><label>배율 <select id="scale"><option value="1">1×</option><option value="4" selected>4×</option></select></label><label>배경 <select id="bg"><option value="#efe5cd">밝게</option><option value="#24232a">어둡게</option></select></label><label>속도 <select id="speed"><option value="0.25">0.25×</option><option value="0.5">0.5×</option><option value="1" selected>1×</option><option value="2">2×</option></select></label></section><div id="stage"><img id="sprite" alt="가죽세공사 애니메이션 프레임"></div><section><input id="scrub" type="range" min="0" value="0"><output id="info"></output></section><small>96×128 원본의 정수 배율 표시, 기본 4배. 전달 이벤트는 transfer 키에만 표시됩니다. 반복 미리보기는 비반복 클립도 확인용으로 반복합니다.</small></main><script>const data=${previewData};const $=id=>document.getElementById(id);let name='walk',index=0,playing=true,elapsed=0,last=0;for(const key of Object.keys(data.clips))$('clip').add(new Option(key,key));$('clip').value=name;function show(){const c=data.clips[name],f=c.frames[index];$('sprite').src=data.frames[f.sprite][$('gear').checked?'equipped':'base'];$('scrub').max=c.frames.length-1;$('scrub').value=index;$('info').textContent=(index+1)+'/'+c.frames.length+' · '+f.sprite+' · '+f.duration_ms+'ms'+(f.event?' · '+f.event:'');$('play').textContent=playing?'일시정지':'재생';}$('clip').onchange=()=>{name=$('clip').value;index=0;elapsed=0;show()};$('gear').onchange=show;$('scale').onchange=()=>{$('stage').style.width=(96*Number($('scale').value))+'px'};$('bg').onchange=()=>{$('stage').style.background=$('bg').value};$('play').onclick=()=>{playing=!playing;show()};function step(delta){playing=false;const n=data.clips[name].frames.length;index=(index+delta+n)%n;elapsed=0;show()};$('prev').onclick=()=>step(-1);$('next').onclick=()=>step(1);$('scrub').oninput=()=>{playing=false;index=Number($('scrub').value);elapsed=0;show()};function tick(now){if(last&&playing){elapsed+=(now-last)*Number($('speed').value);const c=data.clips[name];while(elapsed>=c.frames[index].duration_ms){elapsed-=c.frames[index].duration_ms;if(index+1<c.frames.length)index++;else if(c.loop||$('repeat').checked)index=0;else{playing=false;elapsed=0;break}show()}}last=now;requestAnimationFrame(tick)}show();requestAnimationFrame(tick);</script></html>`);
console.log(JSON.stringify({eye_states:5,mouth_states:4,expression_recipes:8,pose_recipes:Object.keys(poseRecipes).length,clips:Object.keys(clips).length,frames:placements.length,atlas:[pageWidth,pageHeight]}));
