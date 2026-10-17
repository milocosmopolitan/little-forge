import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const W=96, H=128, N=W*H*4;
const magick=(args,input)=>execFileSync('magick',args,{input,maxBuffer:32*1024*1024});
function read(name) {
  const file=path.join(root,'source',name);
  const [w,h]=magick(['identify','-format','%w %h',file]).toString().split(' ').map(Number);
  if(Math.abs(w-1086)>2 || Math.abs(h-1448)>2) throw Error(`Unregistered source ${name}: ${w}x${h}`);
  const rgba=magick([file,'-alpha','on','-depth','8','rgba:-']);
  const out=Buffer.alloc(N);
  for(let y=0;y<H;y++) for(let x=0;x<W;x++) {
    const sx=Math.floor((95+x*10)*w/1086), sy=Math.floor((115+y*10)*h/1448);
    const src=(sy*w+sx)*4, dst=(y*W+x)*4;
    if(rgba[src+3]>=128) {rgba.copy(out,dst,src,src+3);out[dst+3]=255;}
  }
  return out;
}
function write(name,buf) {
  const dest=path.join(root,name);fs.mkdirSync(path.dirname(dest),{recursive:true});
  magick(['-size',`${W}x${H}`,'-depth','8','rgba:-','-strip',`PNG32:${dest}`],buf);
}
const under=read('generated-underlayer.png');
const master=read('generated-master.png');
const reference=read('reference.png');
// Keep the supplied reference RGB; the extraction supplies only its alpha.
for(let i=0;i<N;i+=4)if(master[i+3])reference.copy(master,i,i,i+3);
const apronSource=read('generated-apron-underlay.png');
function select(buf,predicate) {
  const out=Buffer.alloc(N);
  for(let y=0;y<H;y++)for(let x=0;x<W;x++) {
    const i=(y*W+x)*4;if(buf[i+3]&&predicate(x,y,buf.subarray(i,i+4)))buf.copy(out,i,i,i+4);
  }return out;
}
function poly(x,y,points){let inside=false;for(let i=0,j=points.length-1;i<points.length;j=i++){
  const[a,b]=points[i],[c,d]=points[j];if(((b>y)!==(d>y))&&(x<(c-a)*(y-b)/(d-b)+a))inside=!inside;
}return inside;}
// Ownership masks use the registered full-figure sources. Isolated imagegen
// candidates changed object scale and are retained only as rejected sources.
const regions={
  hairpin:[[[26,24],[32,24],[33,29],[30,31],[26,30]]],
  scarf:[[[30,44],[37,42],[42,46],[48,46],[51,44],[55,47],[55,54],[52,58],[46,56],[43,52],[36,50],[31,48]]],
  apron:[[[33,45],[38,46],[40,53],[54,53],[55,47],[58,48],[60,56],[59,72],[62,81],[67,98],[67,101],[47,103],[26,100],[29,88],[33,76],[36,69],[36,54]]],
  'belt-tools':[
    [[37,65],[44,66],[48,65],[52,65],[54,66],[59,66],[60,72],[58,75],[40,75],[37,71]],
    [[15,69],[19,68],[24,70],[27,74],[30,76],[34,74],[38,73],[40,78],[38,85],[35,91],[35,101],[32,103],[29,100],[24,100],[21,97],[18,97],[18,93],[14,92],[16,88],[16,83],[18,81],[15,78]],
    [[39,73],[43,73],[45,78],[46,84],[47,94],[44,99],[40,99],[38,94],[39,87],[37,84],[38,78]],
    [[46,74],[51,74],[53,76],[54,80],[53,85],[54,88],[51,90],[48,87],[46,80]],
    [[53,74],[57,73],[60,75],[60,80],[58,83],[57,85],[54,85],[54,81],[52,78]]
  ],
  gloves:[
    [[25,61],[30,62],[33,66],[37,66],[40,68],[39,71],[35,73],[32,73],[30,71],[26,70],[23,67]],
    [[68,60],[70,58],[74,58],[76,60],[75,64],[72,67],[69,66],[66,64]]
  ],
  'leather-hide':[[[75,59],[79,59],[82,63],[88,70],[86,75],[83,76],[83,83],[87,89],[85,94],[82,93],[82,99],[79,102],[75,100],[71,97],[69,86],[67,79],[64,75],[65,69],[68,65],[71,64]]]
};
const gear={};
for(const[id,polygons]of Object.entries(regions)) {
  const source=id==='apron'?apronSource:master;
  gear[id]=select(source,(x,y,r)=>polygons.some(p=>poly(x+.5,y+.5,p)) && (id!=='apron'||r[0]>r[1]*1.28&&r[0]-r[1]>30&&r[1]-r[2]>10||r[0]<70&&r[1]<50&&r[2]<45) && (id!=='scarf'||r[0]>r[1]*1.7||r[0]<70&&r[1]<50&&r[2]<45) && !(id==='belt-tools'&&y>=98&&x<38&&r[1]>r[0]*.7&&r[2]>r[1]*.55));
}
// The restored bare hand is slightly lower than the reference glove. Register
// each complete isolated glove onto that hand, preserving the body underneath.
const gloveCandidate=read('isolated-gloves.png');
const gloveRegistration=[{source:[23,63,45,80],target:[23,62,42,76]},{source:[67,58,81,70],target:[67,58,77,67]}];
gear.gloves=Buffer.alloc(N);
for(const{source:s,target:t}of gloveRegistration)for(let y=t[1];y<t[3];y++)for(let x=t[0];x<t[2];x++){
  const sx=Math.floor(s[0]+(x-t[0]+.5)*(s[2]-s[0])/(t[2]-t[0])),sy=Math.floor(s[1]+(y-t[1]+.5)*(s[3]-s[1])/(t[3]-t[1]));
  const from=(sy*W+sx)*4,to=(y*W+x)*4;
  const skin=gloveCandidate[from]>190&&gloveCandidate[from+1]>120&&gloveCandidate[from+2]>65;
  if(gloveCandidate[from+3]&&!skin)gloveCandidate.copy(gear.gloves,to,from,from+4);
}
fs.writeFileSync(path.join(root,'source/glove-registration.json'),JSON.stringify(gloveRegistration,null,2)+'\n');
fs.writeFileSync(path.join(root,'source/ownership-regions.json'),JSON.stringify(regions,null,2)+'\n');
const isHead=(x,y)=>y<45||(y<48&&x>=60);
const nearHand=[[24,63],[35,63],[41,67],[43,71],[41,76],[35,76],[29,72],[24,68]];
const isUpper=(x,y)=>!isHead(x,y)&&(y<71||poly(x+.5,y+.5,nearHand));
const head=select(under,isHead);
const upper=select(under,isUpper);
const lower=select(under,(x,y)=>!isHead(x,y)&&!isUpper(x,y));
const feature=(r)=>r[0]<170&&r[1]<125&&r[2]<105 || r[0]>205&&r[1]>205&&r[2]>175;
const eyes=select(master,(x,y,r)=>((x>=39&&x<=48&&y>=27&&y<=35)||(x>=53&&x<=56&&y>=25&&y<=35))&&feature(r));
const mouth=select(master,(x,y,r)=>x>=45&&x<=51&&y>=37&&y<=39&&r[0]<170&&r[1]<125&&r[2]<105);
// Reserve a face palette before quantizing the larger clothing area.
function quantizePalette(buffers,count,predicate=()=>true){
  const rgb=[];for(const buf of buffers)for(let y=0;y<H;y++)for(let x=0;x<W;x++){
    const i=(y*W+x)*4;if(buf[i+3]&&predicate(x,y))rgb.push(buf[i],buf[i+1],buf[i+2]);
  }
  const raw=magick(['-size',`${rgb.length/3}x1`,'-depth','8','rgb:-','-dither','None','-colors',String(count),'-unique-colors','-depth','8','rgb:-'],Buffer.from(rgb));
  const colors=[];for(let i=0;i<raw.length;i+=3)colors.push([...raw.subarray(i,i+3)]);return colors;
}
const palette=[...quantizePalette([master,under],20,(x,y)=>x>=35&&x<=57&&y>=20&&y<=43),...quantizePalette([under,master,...Object.values(gear)],44)];
const unique=[...new Map(palette.map(c=>[c.join(','),c])).values()];
function remap(buf){const out=Buffer.from(buf);for(let i=0;i<N;i+=4)if(out[i+3]){
  let best=unique[0],dist=Infinity;for(const c of unique){const d=(c[0]-out[i])**2+(c[1]-out[i+1])**2+(c[2]-out[i+2])**2;if(d<dist){dist=d;best=c;}}
  for(let k=0;k<3;k++)out[i+k]=best[k];
}return out;}
function compose(...layers){const out=Buffer.alloc(N);for(const b of layers)for(let i=0;i<N;i+=4)if(b[i+3])b.copy(out,i,i,i+4);return out;}
const layers={'head-base.png':head,'body/upper-base.png':upper,'body/lower-base.png':lower,'eyes/open.png':eyes,'mouth/rest.png':mouth};
for(const [id,buf]of Object.entries(gear))layers[id==='leather-hide'?'props/leather-hide.png':`equipment/${id}.png`]=buf;
for(const key of Object.keys(layers))layers[key]=remap(layers[key]);
const foundation=compose(layers['body/lower-base.png'],layers['body/upper-base.png'],layers['head-base.png']);
const base=compose(foundation,layers['eyes/open.png'],layers['mouth/rest.png']);
const equipment=['apron','scarf','belt-tools','gloves','hairpin'].map(id=>layers[`equipment/${id}.png`]);
const noProp=compose(base,...equipment);
const neutral=compose(layers['props/leather-hide.png'],base,...equipment);
for(const [name,buf]of Object.entries(layers))write(name,buf);
write('source/native-underlayer.png',remap(under));write('source/foundation.png',foundation);
write('source/native-reference.png',remap(master));
write('base-neutral.png',base);write('neutral-no-prop.png',noProp);write('neutral.png',neutral);
const report={canvas_px:[W,H],source_size_px:[1086,1448],sample_source_xy:'(95 + 10*x, 115 + 10*y)',scale:.1,translation_px:[-9,-11],alpha_values:[0,255],palette_colors:unique.length,partition_mismatches:0,foundation_pixels_lost:{base:0,no_prop:0,full:0},layer_bounds:{}};
if(!foundation.equals(remap(under)))throw Error('Body partition does not reconstruct complete underlayer');
for(const[name,buf]of Object.entries({base,no_prop:noProp,full:neutral}))for(let i=0;i<N;i+=4)if(foundation[i+3]&&!buf[i+3])report.foundation_pixels_lost[name]++;
for(const[name,buf]of Object.entries(layers)){
  let x0=W,y0=H,x1=-1,y1=-1,count=0;for(let y=0;y<H;y++)for(let x=0;x<W;x++)if(buf[(y*W+x)*4+3]){count++;x0=Math.min(x0,x);x1=Math.max(x1,x);y0=Math.min(y0,y);y1=Math.max(y1,y);}
  report.layer_bounds[name]={bbox_half_open:[x0,y0,x1+1,y1+1],opaque_pixels:count};
}
for(const[name,count]of Object.entries(report.foundation_pixels_lost))if(count)throw Error(`${name} loses ${count} foundation pixels`);
fs.writeFileSync(path.join(root,'source/palette.json'),JSON.stringify(unique.map(c=>'#'+c.map(v=>v.toString(16).padStart(2,'0')).join('')),null,2)+'\n');
fs.writeFileSync(path.join(root,'qa/coverage-report.json'),JSON.stringify(report,null,2)+'\n');
for(const name of ['neutral','base-neutral','neutral-no-prop'])magick([path.join(root,`${name}.png`),'-filter','point','-resize','400%','-strip',path.join(root,`previews/${name}-4x.png`)]);
const preview=(files,out,background='#eee5cd',tile='3x1')=>magick(['montage',...files.map(f=>path.join(root,f)),'-background',background,'-alpha','remove','-filter','point','-resize','400%','-tile',tile,'-geometry','+8+8','-strip',path.join(root,`previews/${out}.png`)]);
preview(['source/native-reference.png','base-neutral.png','neutral.png'],'reference-base-equipped');
preview(['base-neutral.png','neutral-no-prop.png','neutral.png'],'toggles-dark','#292a36');
preview(Object.keys(layers),'layers-light','#eee5cd','4x3');
preview(Object.keys(layers),'layers-dark','#292a36','4x3');
magick(['montage',path.join(root,'source/native-reference.png'),path.join(root,'base-neutral.png'),path.join(root,'neutral.png'),'-crop','44x44+18+5','+repage','-background','#eee5cd','-alpha','remove','-filter','point','-resize','800%','-tile','3x1','-geometry','+8+8','-strip',path.join(root,'previews/face-comparison.png')]);
console.log(JSON.stringify(report,null,2));
