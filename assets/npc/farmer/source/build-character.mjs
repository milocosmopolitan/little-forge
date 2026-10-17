// A complete underlayer is the foundation. Equipment only adds opaque pixels.
import {execFileSync} from 'node:child_process';
import {writeFileSync,existsSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const W=96,H=128,N=W*H,empty=()=>Buffer.alloc(N*4);
const read=name=>execFileSync('magick',[path.join(root,name),'-depth','8','rgba:-'],{maxBuffer:16*1024*1024});
const save=(name,data)=>execFileSync('magick',['-size','96x128','-depth','8','rgba:-','-define','png:exclude-chunks=date,time','PNG32:'+path.join(root,name)],{input:data});
function sample(name) {
  const [sw,sh]=execFileSync('magick',['identify','-format','%w %h',path.join(root,name)],{encoding:'utf8'}).split(' ').map(Number);
  const source=read(name),out=empty();
  for(let y=0;y<H;y++) for(let x=0;x<W;x++) {
    const sx=Math.floor((x+11)*10*sw/1122),sy=Math.floor((y+10)*10*sh/1402),i=(y*W+x)*4,j=(sy*sw+sx)*4;
    if(sx<0||sx>=sw||sy<0||sy>=sh||source[j+3]<128) continue;
    source.copy(out,i,j,j+3);out[i+3]=255;
  }
  return out;
}
function shift(source,dx,dy) {
  const out=empty();
  for(let y=0;y<H;y++) for(let x=0;x<W;x++) {
    if(x+dx<0||x+dx>=W||y+dy<0||y+dy>=H) continue;
    const i=(y*W+x)*4;source.copy(out,((y+dy)*W+x+dx)*4,i,i+4);
  }
  return out;
}
const approvedFace=shift(read('source/approved-face/head-base.png'),-2,6);
const eyes=shift(read('source/approved-face/eyes-open.png'),-2,6);
const mouth=shift(read('source/approved-face/mouth-rest.png'),-2,6);
const masterRaw=sample('source/body-rebuilt-master.png'),baseRaw=sample('source/body-rebuilt-base.png'),apronRaw=sample('source/body-rebuilt-apron.png');
// Reserve the face's actual colors, then use the remaining palette for cloth/leather.
const paletteMap=new Map();
for(const layer of [approvedFace,eyes,mouth]) for(let i=0;i<layer.length;i+=4) if(layer[i+3]) paletteMap.set(layer.subarray(i,i+3).toString('hex'),[...layer.subarray(i,i+3)]);
const bodyPixels=[];
for(const layer of [masterRaw,baseRaw,apronRaw]) for(let y=0;y<H;y++) for(let x=0;x<W;x++) {
  const i=(y*W+x)*4;if(layer[i+3]) bodyPixels.push(...layer.subarray(i,i+3));
}
// Hat material also needs its own gold/leaf ramps.
for(let i=0;i<masterRaw.length;i+=4) if(masterRaw[i+3]) bodyPixels.push(...masterRaw.subarray(i,i+3));
const strip=execFileSync('magick',['-size',`${bodyPixels.length/3}x1`,'-depth','8','rgb:-','-dither','None','-colors',String(64-paletteMap.size),'-depth','8','rgb:-'],{input:Buffer.from(bodyPixels),maxBuffer:16*1024*1024});
for(let i=0;i<strip.length;i+=3) paletteMap.set(strip.subarray(i,i+3).toString('hex'),[...strip.subarray(i,i+3)]);
const palette=[...paletteMap.values()];
if(palette.length>64) throw Error('Shared palette exceeds contract');
function mapColors(source) {
  const out=Buffer.from(source);
  for(let i=0;i<out.length;i+=4) if(out[i+3]) {
    let best=palette[0],distance=Infinity;
    for(const color of palette) {
      const d=color.reduce((s,v,k)=>s+(v-source[i+k])**2,0);
      if(d<distance){best=color;distance=d;}
    }
    for(let k=0;k<3;k++) out[i+k]=best[k];
  }
  return out;
}
const master=mapColors(masterRaw),basic=mapColors(baseRaw),apronSource=mapColors(apronRaw);
function inside(x,y,p) {
  let yes=false;
  for(let i=0,j=p.length-1;i<p.length;j=i++) {
    const [a,b]=p[i],[c,d]=p[j];
    if((b>y)!==(d>y)&&x<(c-a)*(y-b)/(d-b)+a) yes=!yes;
  }
  return yes;
}
const rect=(x,y,a,b,c,d)=>x>=a&&x<c&&y>=b&&y<d;
const hatPoly=[[245,445],[333,352],[308,282],[352,264],[359,210],[399,242],[480,180],[534,160],[613,160],[657,193],[691,249],[714,256],[754,235],[764,265],[811,259],[835,289],[856,311],[862,381],[833,425],[796,436],[768,471],[738,481],[718,456],[718,401],[690,361],[658,332],[615,317],[558,337],[496,375],[456,407],[425,429],[404,462],[397,488],[400,533],[376,549],[338,553],[282,533],[249,516]];
const apronPoly=[[437,596],[479,596],[533,692],[680,692],[685,623],[723,626],[755,715],[770,837],[775,938],[790,1125],[454,1125],[350,1070],[376,980],[400,910],[410,838],[442,760]];
const glovePolys=[[[320,800],[425,790],[485,840],[490,940],[450,980],[350,970],[320,890]],[[760,879],[835,880],[862,925],[862,1005],[814,1007],[783,978],[775,943]]];
const foundation=Buffer.from(basic),head=empty(),hat=empty(),apron=empty(),tools=empty(),gloves=empty();
const upper=empty(),lower=empty();
// Keep the improved facial interior, never its obsolete hat-clipped hair silhouette.
const facePoly=[[493,420],[514,385],[548,360],[587,344],[630,346],[660,370],[676,410],[699,416],[705,460],[711,493],[720,515],[716,560],[697,586],[668,603],[671,650],[580,650],[543,616],[513,589],[498,551],[482,540],[456,543],[426,532],[410,515],[408,482],[423,464],[447,460],[472,480],[485,474]];
for(let y=0;y<H;y++) for(let x=0;x<W;x++) {
  const i=(y*W+x)*4,rx=(x+11)*10,ry=(y+10)*10;
  const glove=glovePolys.some(p=>inside(rx,ry,p));
  const cloth=inside(rx,ry,apronPoly)&&!glove;
  if(inside(rx,ry,facePoly)&&approvedFace[i+3]&&basic[i+3]) approvedFace.copy(foundation,i,i,i+4);
  if(inside(rx,ry,hatPoly)) master.copy(hat,i,i,i+4);
  if(y<50) continue;
  if(glove) master.copy(gloves,i,i,i+4);
  if(cloth) apronSource.copy(apron,i,i,i+4);
  const p=master.subarray(i,i+4);
  const toolRegion=rect(rx,ry,438,828,747,896)||rect(rx,ry,480,825,567,1055);
  const material=(p[0]>p[1]*1.1&&p[0]<210)||Math.max(...p.subarray(0,3))<75||(rect(rx,ry,493,879,533,948)&&Math.min(...p.subarray(0,3))>175);
  if(toolRegion&&material&&!glove) master.copy(tools,i,i,i+4);
}
// Partition every foundation pixel exactly once; the seam cannot create gaps.
for(let y=0;y<H;y++) for(let x=0;x<W;x++) {
  const i=(y*W+x)*4;
  const leftHand=inside(x,y,[[19,68],[30,68],[30,73],[35,76],[38,80],[38,84],[34,86],[28,84],[24,80],[20,75]]);
  const rightHand=rect(x,y,65,76,77,94);
  const hand=leftHand||rightHand;
  const headPixel=y<50||(y<55&&x>=39&&x<=62);
  foundation.copy(headPixel?head:y<76||hand?upper:lower,i,i,i+4);
}
function compose(...layers) {
  const out=empty();for(const layer of layers) for(let i=0;i<out.length;i+=4) if(layer[i+3]) layer.copy(out,i,i,i+4);return out;
}
const combined=compose(lower,upper,head,eyes,mouth);
if(!compose(lower,upper,head).equals(foundation)) throw Error('Layer split lost foundation pixels');
const final=compose(combined,apron,tools,gloves,hat);
let foundationPixels=0;
for(let i=0;i<basic.length;i+=4) if(basic[i+3]) {
  foundationPixels++;
  if(!foundation[i+3]||!combined[i+3]||!final[i+3]) throw Error(`Lost underlayer coverage at ${i/4%W},${Math.floor(i/4/W)}`);
}
const files={'head-base.png':head,'eyes/open.png':eyes,'mouth/rest.png':mouth,'body/upper-base.png':upper,'body/lower-base.png':lower,'equipment/apron.png':apron,'equipment/belt-tools.png':tools,'equipment/gloves.png':gloves,'equipment/hat.png':hat,'equipment/apron-belt.png':compose(apron,tools),'base-neutral.png':combined,'neutral.png':final,'source/rebuilt-master-native.png':master};
for(const [name,data] of Object.entries(files)) save(name,data);
save('source/native-underlayer.png',basic);
save('source/foundation-native.png',foundation);
save('previews/no-hat-native.png',compose(combined,apron,tools,gloves));
save('previews/apron-only-native.png',compose(combined,apron));
// This reference is diagnostic only; the retained native comparison survives source cleanup.
if(existsSync(path.join(root,'source/reference.png')))save('source/body-reference-native.png',sample('source/reference.png'));
else if(!existsSync(path.join(root,'source/body-reference-native.png')))throw Error('Missing diagnostic reference and retained native comparison');
const actualColors=new Set();for(const data of Object.values(files)) for(let i=0;i<data.length;i+=4) if(data[i+3]) actualColors.add('#'+data.subarray(i,i+4).toString('hex'));
writeFileSync(path.join(root,'source/palette.json'),JSON.stringify([...actualColors].sort(),null,2)+'\n');
writeFileSync(path.join(root,'source/body-rebuild-geometry.json'),JSON.stringify({native_canvas:[W,H],source_scale:.1,source_offset:[-11,-10],approved_face_translation:[-2,6],waist_y:76,foot:[48,124],palette_colors:actualColors.size,outline_processing:'none; preserve authored source contours and material shadows'},null,2)+'\n');
writeFileSync(path.join(root,'source/coverage-check.json'),JSON.stringify({foundation_pixels:foundationPixels,lost_foundation_pixels:0,split_reconstructs_foundation:true,head_source:'body-rebuilt-base.png; complete hatless hair',face_source:'approved-face facial interior only',eyes_mouth_translation:[-2,6]},null,2)+'\n');
const convert=(...args)=>execFileSync('magick',[...args.slice(0,-1),'-define','png:exclude-chunks=date,time',args.at(-1)].map(String));
for(const [name,source,scale,bg] of [
  ['neutral-4x.png','neutral.png',4,null],['neutral-8x.png','neutral.png',8,null],
  ['base-neutral-4x.png','base-neutral.png',4,null],['light-4x.png','neutral.png',4,'#efe5cd'],
  ['dark-4x.png','neutral.png',4,'#24232a'],['base-light-4x.png','base-neutral.png',4,'#efe5cd']
]) convert(path.join(root,source),'-filter','point','-resize',`${scale*100}%`,...(bg?['-background',bg,'-alpha','remove']:[]),path.join(root,'previews',name));
function sheet(name,paths,tile) {
  convert('montage',...paths.map(p=>path.join(root,p)),'-filter','point','-background','#efe5cd','-geometry','384x512+8+8','-tile',tile,path.join(root,'previews',name));
}
sheet('underlayer-check.png',['source/native-underlayer.png','base-neutral.png','previews/no-hat-native.png','neutral.png'],'4x1');
sheet('head-check.png',['neutral.png','base-neutral.png','head-base.png'],'3x1');
sheet('body-comparison.png',['source/body-reference-native.png','source/body-before.png','neutral.png'],'3x1');
sheet('face-comparison.png',['source/body-reference-native.png','base-neutral.png','neutral.png'],'3x1');
sheet('layers.png',['head-base.png','eyes/open.png','mouth/rest.png','body/upper-base.png','body/lower-base.png','equipment/hat.png','equipment/apron.png','equipment/belt-tools.png','equipment/gloves.png'],'3x3');
console.log('Rebuilt character:',actualColors.size,'colors;',foundationPixels,'underlayer pixels retained; complete hair; approved eyes/mouth unchanged');
