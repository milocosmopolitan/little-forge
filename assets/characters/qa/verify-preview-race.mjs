import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
const html=fs.readFileSync('assets/characters/previews/animation.html','utf8');
const source=html.slice(html.indexOf('let renderVersion=0;'),html.indexOf("$('play').onclick"));
const pending=new Map(),visible={},controls={clip:{value:'quenching'},gear:{checked:true},dark:{checked:false},stations:{checked:false},seek:{},frame:{}};
function canvas(){const operations=[];return {operations,getContext(){return {fillRect(...args){operations.push(['fill',...args])},drawImage(...args){operations.push(['draw',...args])}}}};}
for(const sex of ['female','male'])controls[sex]=visible[sex]=canvas();
const fixtures={};for(const sex of ['female','male'])fixtures[sex]={clips:{quenching:{frames:[{sprite:'quenching-0'},{sprite:'quenching-1'}]}},poses:{'quenching-0':{},'quenching-1':{}},workshop:{stations:{}}};
const context=vm.createContext({$:id=>controls[id],sexes:['female','male'],data:fixtures,frame:0,playing:false,loop:0,entered:new Set(),window:{},document:{createElement:()=>canvas()},clip:()=>fixtures.female.clips.quenching,image:path=>new Promise(resolve=>pending.set(path,()=>resolve({path})))});
vm.runInContext(source,context);
const old=context.render();
context.frame=1;controls.gear.checked=false;controls.dark.checked=true;
const latest=context.render();
pending.get('../male/frames/base/quenching-1.png')();
assert.equal(visible.male.operations.length,0,'A partially loaded pair became visible');
pending.get('../female/frames/base/quenching-1.png')();
assert.equal(await latest,true);
for(const sex of ['female','male']){
 assert.equal(visible[sex].operations.length,1);
 const buffer=visible[sex].operations[0][1];
 assert.equal(buffer.operations.filter(op=>op[0]==='draw').length,1);
 assert.equal(buffer.operations.find(op=>op[0]==='draw')[1].path,`../${sex}/frames/base/quenching-1.png`);
}
pending.get('../female/frames/quenching-0.png')();pending.get('../male/frames/quenching-0.png')();
assert.equal(await old,false,'An obsolete render was committed');
for(const sex of ['female','male'])assert.equal(visible[sex].operations.length,1,'Old and current actors were mixed');
assert.equal(context.window.reviewState.frame,1);assert.equal(controls.frame.textContent,'1 / 1');
const report={status:'pass',scope:'Actual preview render function with deliberately reordered image promises',both_characters_commit_together:true,stale_render_discarded:true,latest_frame_and_equipment_snapshot_retained:true};
fs.writeFileSync('assets/characters/qa/render-race-validation.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
