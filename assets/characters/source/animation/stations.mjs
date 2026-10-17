import fs from 'node:fs';import path from 'node:path';import {magick} from './raster.mjs';
const root=path.resolve('assets/characters/stations');fs.mkdirSync(root,{recursive:true});
const defs={anvil:[64,64,[18,8]],'quenching-tub':[64,48,[29,18]],forge:[128,144,[48,76]],'research-table':[128,96,[64,26]],'salvage-table':[128,96,[64,39]]};
const out={version:1,world_canvas:[256,192],character_origin:[32,16],sampler:'nearest',mipmaps:false,stations:{},runtime_integration:false};
for(const[id,[w,h,point]]of Object.entries(defs)){
 const src=path.resolve(id==='research-table'?'assets/characters/source/animation/work-repair/research-table-tall.png':path.join('assets/forge/workshop/source',id+'.png')),dest=path.join(root,id+'.png');
 magick([src,'-channel','A','-threshold','75%','+channel','-trim','+repage','-filter','point','-resize',id==='anvil'?'52x52!':`${w}x${h}!`,'-background','none','-gravity','northwest','-extent',`${w}x${h}`,'-dither','None','-colors','48','-strip','PNG32:'+dest]);
 const raw=magick([dest,'-depth','8','rgba:-']),front=Buffer.alloc(raw.length),back=Buffer.from(raw),water=Buffer.alloc(raw.length),fire=Buffer.alloc(raw.length);
 for(let y=0;y<h;y++)for(let x=0;x<w;x++){const i=(y*w+x)*4,p=raw.subarray(i,i+4);if(!p[3])continue;
  let foreground=id==='quenching-tub'&&y>=27||id==='forge'&&x<80&&y>=90&&y<107;
  if(foreground){raw.copy(front,i,i,i+4);back.fill(0,i,i+4);}
  if(id==='quenching-tub'&&y>=10&&y<29&&x>5&&x<59&&p[2]>p[0]*1.1&&p[2]>p[1]){raw.copy(water,i,i,i+4);back.fill(0,i,i+4);}
  if(id==='forge'&&y>46&&y<106&&x>24&&x<94&&p[0]>180&&p[0]>p[1]*1.3&&p[2]<90){raw.copy(fire,i,i,i+4);back.fill(0,i,i+4);}
 }
 for(const[name,b]of [['back',back],['front',front],['water',water],['fire',fire]])magick(['-size',`${w}x${h}`,'-depth','8','rgba:-','-strip','PNG32:'+path.join(root,id+'-'+name+'.png')],b);
 out.stations[id]={size:[w,h],path:'stations/'+id+'.png',back:'stations/'+id+'-back.png',front:'stations/'+id+'-front.png',water:'stations/'+id+'-water.png',fire:'stations/'+id+'-fire.png',contact:point,source:id==='research-table'?'source/animation/work-repair/research-table-tall.png':'../forge/workshop/source/'+id+'.png',source_status:'generated_reference',derived_native:true};
}
fs.writeFileSync(path.join(root,'catalog.json'),JSON.stringify(out,null,2)+'\n');console.log('5 native station candidates registered');
