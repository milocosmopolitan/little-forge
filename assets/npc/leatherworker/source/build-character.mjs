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
function registered(name, toSource) {
  const file=path.join(root,'source',name);
  const [w,h]=magick(['identify','-format','%w %h',file]).toString().split(' ').map(Number);
  const rgba=magick([file,'-alpha','on','-depth','8','rgba:-']);
  const out=Buffer.alloc(N);
  for(let y=0;y<H;y++)for(let x=0;x<W;x++){
    const [u,v]=toSource(x+.5,y+.5);
    const sx=Math.floor((90+u*10)*w/1086),sy=Math.floor((110+v*10)*h/1448);
    if(sx<0||sx>=w||sy<0||sy>=h)continue;
    const from=(sy*w+sx)*4,to=(y*W+x)*4;
    if(rgba[from+3]>=128){rgba.copy(out,to,from,from+3);out[to+3]=255;}
  }return out;
}
function write(name,buf) {
  const dest=path.join(root,name);fs.mkdirSync(path.dirname(dest),{recursive:true});
  magick(['-size',`${W}x${H}`,'-depth','8','rgba:-','-strip',`PNG32:${dest}`],buf);
}
function readNative(name){return magick([path.join(root,name),'-alpha','on','-depth','8','rgba:-']);}
const under=compose(...['body/lower-base.png','body/upper-base.png','head-base.png'].map(n=>readNative(`source/revision-1-before/${n}`)));
const master=read('generated-master.png');
const reference=read('reference.png');
// Keep the supplied reference RGB; the extraction supplies only its alpha.
for(let i=0;i<N;i+=4)if(master[i+3])reference.copy(master,i,i,i+3);
function select(buf,predicate) {
  const out=Buffer.alloc(N);
  for(let y=0;y<H;y++)for(let x=0;x<W;x++) {
    const i=(y*W+x)*4;if(buf[i+3]&&predicate(x,y,buf.subarray(i,i+4)))buf.copy(out,i,i,i+4);
  }return out;
}
function poly(x,y,points){let inside=false;for(let i=0,j=points.length-1;i<points.length;j=i++){
  const[a,b]=points[i],[c,d]=points[j];if(((b>y)!==(d>y))&&(x<(c-a)*(y-b)/(d-b)+a))inside=!inside;
}return inside;}
const gear={hairpin:select(master,(x,y)=>poly(x+.5,y+.5,[[26,24],[32,24],[33,29],[30,31],[26,30]]))};
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

// Complete isolated material layers replace clipped polygons and color filters.
// Registration is evaluated in the high-resolution source before native sampling.
gear.apron=registered('isolated-apron.png',(x,y)=>[(x-8.5)/.82,y<=57?y:57+(y-57)/(.868)]);
gear['belt-tools']=registered('isolated-belt-tools.png',(x,y)=>[(x-9)/.7,(y-(x<37?30:27))/.7]);
gear.scarf=registered('isolated-scarf.png',(x,y)=>[(x-11.65)/.75,(y-11.1)/.75]);
const hideBounds=[61,56.5,90,113.4], hideTarget=[64,59,88,103];
gear['leather-hide']=registered('corrections/leather-hide-closed.png',(x,y)=>[
  hideBounds[0]+(x-hideTarget[0])*(hideBounds[2]-hideBounds[0])/(hideTarget[2]-hideTarget[0]),
  hideBounds[1]+(y-hideTarget[1])*(hideBounds[3]-hideBounds[1])/(hideTarget[3]-hideTarget[1])
]);
const isHead=(x,y)=>y<45||(y<48&&x>=60);
const nearHand=[[24,63],[35,63],[41,67],[43,71],[41,76],[35,76],[29,72],[24,68]];
const isUpper=(x,y)=>!isHead(x,y)&&(y<71||poly(x+.5,y+.5,nearHand));
const head=select(under,isHead);
const upper=select(under,isUpper);
const lower=select(under,(x,y)=>!isHead(x,y)&&!isUpper(x,y));
const eyes=readNative('source/revision-1-before/eyes/open.png');
const mouth=readNative('source/revision-1-before/mouth/rest.png');
// Restore only the supplied cheek pigments. Head silhouette and eyes stay fixed.
const blush=select(master,(x,y,r)=>((x>=40&&x<=44&&y>=34&&y<=37)||(x>=54&&x<=56&&y>=33&&y<=36))&&r[0]>190&&r[1]>90&&r[1]<165&&r[2]<130);
for(let i=0;i<N;i+=4)if(blush[i+3]&&head[i+3]&&!eyes[i+3]&&!mouth[i+3]){
  blush.copy(head,i,i,i+4);blush.copy(under,i,i,i+4);
}
// Shared palette construction and fixed-color remapping.
function quantizePalette(buffers,count,predicate=()=>true){
  const rgb=[];for(const buf of buffers)for(let y=0;y<H;y++)for(let x=0;x<W;x++){
    const i=(y*W+x)*4;if(buf[i+3]&&predicate(x,y))rgb.push(buf[i],buf[i+1],buf[i+2]);
  }
  const raw=magick(['-size',`${rgb.length/3}x1`,'-depth','8','rgb:-','-dither','None','-colors',String(count),'-unique-colors','-depth','8','rgb:-'],Buffer.from(rgb));
  const colors=[];for(let i=0;i<raw.length;i+=3)colors.push([...raw.subarray(i,i+3)]);return colors;
}
// Keep the accepted bare body, hands and eyes byte-identical during gear repair.
// Reserve 16 additional material slots instead of washing blush/red/gold into
// nearby hair browns. Existing 64 entries remain available to the locked body.
const lockedPalette=JSON.parse(fs.readFileSync(path.join(root,'source/revision-1-before/palette.json'),'utf8')).map(c=>[...Buffer.from(c.slice(1),'hex')]);
const referenceScarf=select(master,(x,y,r)=>x>=35&&x<=53&&y>=45&&y<=56&&r[0]>r[1]*1.7&&r[1]>35&&r[1]<90&&r[2]>20);
const scarfPalette=quantizePalette([referenceScarf],3);
const palette=[...lockedPalette,[245,128,86],...scarfPalette,...quantizePalette([gear.apron,gear['belt-tools']],12)];
const unique=[...new Map(palette.map(c=>[c.join(','),c])).values()];
function remap(buf,colors=unique){const out=Buffer.from(buf);for(let i=0;i<N;i+=4)if(out[i+3]){
  let best=colors[0],dist=Infinity;for(const c of colors){const d=(c[0]-out[i])**2+(c[1]-out[i+1])**2+(c[2]-out[i+2])**2;if(d<dist){dist=d;best=c;}}
  for(let k=0;k<3;k++)out[i+k]=best[k];
}return out;}
function compose(...layers){const out=Buffer.alloc(N);for(const b of layers)for(let i=0;i<N;i+=4)if(b[i+3])b.copy(out,i,i,i+4);return out;}
const layers={'head-base.png':head,'body/upper-base.png':upper,'body/lower-base.png':lower,'eyes/open.png':eyes,'mouth/rest.png':mouth};
for(const [id,buf]of Object.entries(gear))layers[id==='leather-hide'?'props/leather-hide.png':`equipment/${id}.png`]=buf;
for(const key of Object.keys(layers))layers[key]=remap(layers[key],key==='equipment/scarf.png'?[...scarfPalette,...lockedPalette.filter(c=>c[0]<65)]:unique);
const foundation=compose(layers['body/lower-base.png'],layers['body/upper-base.png'],layers['head-base.png']);
const base=compose(foundation,layers['eyes/open.png'],layers['mouth/rest.png']);
const equipment=['apron','scarf','belt-tools','gloves','hairpin'].map(id=>layers[`equipment/${id}.png`]);
const noProp=compose(base,...equipment);
// The folded leather covers the lower grip. Knuckles above y=59 stay visible;
// putting the entire hand over the hide exposed fingers through its fold.
const neutral=compose(base,...equipment,layers['props/leather-hide.png']);
for(const [name,buf]of Object.entries(layers))write(name,buf);
write('source/native-underlayer.png',remap(under));write('source/foundation.png',foundation);
write('source/corrections/blush-mask.png',select(layers['head-base.png'],(x,y)=>blush[(y*W+x)*4+3]>0));
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
preview(['source/revision-1-before/neutral.png','neutral.png'],'correction-before-after','#eee5cd','2x1');
magick(['montage',path.join(root,'source/revision-1-before/neutral.png'),path.join(root,'neutral.png'),'-crop','29x34+61+52','+repage','-background','#eee5cd','-alpha','remove','-filter','point','-resize','800%','-tile','2x1','-geometry','+8+8','-strip',path.join(root,'previews/hand-before-after.png')]);
preview(['base-neutral.png','neutral-no-prop.png','neutral.png'],'toggles-dark','#292a36');
preview(Object.keys(layers),'layers-light','#eee5cd','4x3');
preview(Object.keys(layers),'layers-dark','#292a36','4x3');
magick(['montage',path.join(root,'source/native-reference.png'),path.join(root,'base-neutral.png'),path.join(root,'neutral.png'),'-crop','44x44+18+5','+repage','-background','#eee5cd','-alpha','remove','-filter','point','-resize','800%','-tile','3x1','-geometry','+8+8','-strip',path.join(root,'previews/face-comparison.png')]);
console.log(JSON.stringify(report,null,2));
