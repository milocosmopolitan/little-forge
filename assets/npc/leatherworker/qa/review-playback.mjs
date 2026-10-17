// Capture actual browser playback at normal/slow speed, with/without equipment.
import {writeFileSync,mkdirSync} from 'node:fs';
import {file} from '../source/animation-lib.mjs';
const target=(await(await fetch('http://127.0.0.1:9237/json')).json()).find(t=>t.type==='page');
if(!target)throw Error('Start isolated headless Chrome on port 9237');
const ws=new WebSocket(target.webSocketDebuggerUrl),pending=new Map();let seq=0;
ws.onmessage=e=>{const m=JSON.parse(e.data);if(m.id){const p=pending.get(m.id);pending.delete(m.id);m.error?p.reject(m.error):p.resolve(m.result);}};
await new Promise((resolve,reject)=>{ws.onopen=resolve;ws.onerror=reject;});
const send=(method,params={})=>new Promise((resolve,reject)=>{const id=++seq;pending.set(id,{resolve,reject});ws.send(JSON.stringify({id,method,params}));});
const evaluate=async expression=>{const r=await send('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});if(r.exceptionDetails)throw Error(JSON.stringify(r.exceptionDetails));return r.result.value;};
const wait=ms=>new Promise(r=>setTimeout(r,ms));
await send('Page.enable');await send('Runtime.enable');
await send('Emulation.setDeviceMetricsOverride',{width:1000,height:900,deviceScaleFactor:1,mobile:false});
await send('Page.navigate',{url:'http://127.0.0.1:8771/assets/npc/leatherworker/previews/animation.html'});
for(let n=0;n<100;n++){await wait(50);if(await evaluate("document.readyState==='complete'&&!!document.getElementById('clip')&&document.getElementById('sprite').complete"))break;}
await evaluate("Promise.all(Object.values(data.frames).flatMap(f=>Object.values(f)).map(src=>new Promise(r=>{const i=new Image;i.onload=r;i.onerror=r;i.src=src})))");
mkdirSync(file('qa/playback-captures'),{recursive:true});
const report={browser:'isolated headless Chrome, actual requestAnimationFrame clock',runtime_integration:false,walk:[],other_clips:[],gesture_captures:[]};
for(const scale of [1,4])for(const gear of [false,true])for(const speed of [1,.25]){
 const id=`walk-${scale}x-${gear?'equipped':'base'}-${speed}speed`;
 await evaluate(`(()=>{playing=false;name='walk';index=0;elapsed=0;$('clip').value=name;$('gear').checked=${gear};$('speed').value='${speed}';$('scale').value='${scale}';$('scale').onchange();$('bg').value='${gear?'#24232a':'#efe5cd'}';$('bg').onchange();window.observed=[];if(window.observer)window.observer.disconnect();window.observer=new MutationObserver(()=>{window.observed.push({t:performance.now(),src:$('sprite').getAttribute('src'),info:$('info').textContent})});window.observer.observe($('sprite'),{attributes:true,attributeFilter:['src']});show();playing=true;})()`);
 const observedKeys=new Set(),deadline=Date.now()+Math.max(10000,Math.ceil(720/speed)*4);
 while(Date.now()<deadline&&observedKeys.size<6){
  await wait(55);
  const state=await evaluate("({index,src:$('sprite').getAttribute('src'),ready:$('sprite').complete&&$('sprite').naturalWidth>0,rect:(()=>{const r=$('stage').getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height,scale:1}})()})");
  if(state.ready&&!observedKeys.has(state.index)){
   const shot=await send('Page.captureScreenshot',{format:'png',clip:state.rect});
   if(await evaluate('index')!==state.index)continue;
   writeFileSync(file(`qa/playback-captures/${id}-key-${state.index}.png`),Buffer.from(shot.data,'base64'));observedKeys.add(state.index);
  }
 }
 const transitions=await evaluate('window.observed');
 if(observedKeys.size!==6)throw Error(`${id}: captured only ${[...observedKeys]}`);
 report.walk.push({scale,gear,speed,keys_captured:[...observedKeys].sort(),transitions});
}
for(const name of ['talk','blink','nod','shake_head','give','talk_blink']){
 await evaluate(`(()=>{playing=false;name='${name}';index=0;elapsed=0;$('clip').value=name;$('gear').checked=true;$('speed').value='1';$('scale').value='4';$('scale').onchange();window.observed=[];show();playing=true;})()`);
 const duration=await evaluate('data.clips[name].frames.reduce((n,f)=>n+f.duration_ms,0)');
 await wait(duration+80);
 report.other_clips.push({name,transitions:await evaluate('window.observed')});
}
for(const clipName of ['nod','shake_head','give']){
 await evaluate(`(()=>{playing=false;name='${clipName}';index=0;elapsed=0;$('clip').value=name;$('gear').checked=false;$('speed').value='0.25';$('scale').value='4';$('scale').onchange();$('bg').value='#efe5cd';$('bg').onchange();window.observed=[];show();playing=true;})()`);
 const count=await evaluate('data.clips[name].frames.length');
 const duration=await evaluate('data.clips[name].frames.reduce((n,f)=>n+f.duration_ms,0)');
 const captured=new Set(),deadline=Date.now()+duration*12+1000;
 while(Date.now()<deadline&&captured.size<count){
  await wait(50);
  const state=await evaluate("({index,ready:$('sprite').complete&&$('sprite').naturalWidth>0,rect:(()=>{const r=$('stage').getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height,scale:1}})()})");
  if(state.ready&&!captured.has(state.index)){
   const shot=await send('Page.captureScreenshot',{format:'png',clip:state.rect});
   if(await evaluate('index')!==state.index)continue;
   writeFileSync(file(`qa/playback-captures/${clipName}-base-key-${state.index}.png`),Buffer.from(shot.data,'base64'));captured.add(state.index);
  }
 }
 if(captured.size!==count)throw Error(`${clipName}: captured ${[...captured]}, expected ${count}`);
 report.gesture_captures.push({name:clipName,scale:4,gear:false,speed:.25,keys_captured:[...captured].sort(),transitions:await evaluate('window.observed')});
}
await evaluate("playing=false;name='walk';index=0;show()");
writeFileSync(file('qa/playback-report.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({walk_modes:report.walk.length,keys_per_mode:6,other_clips:report.other_clips.length,playback:'captured actual normal/slow browser replay; visual inspection remains separate'}));
ws.close();
