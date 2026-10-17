// Registered reference-pixel composition; no body or equipment pixels are redrawn.
import {execFileSync} from 'node:child_process';
import {writeFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const w=96,h=128;
const read=name=>execFileSync('magick',[path.join(root,name),'-depth','8','rgba:-']);
const save=(name,data)=>execFileSync('magick',['-size',`${w}x${h}`,'-depth','8','rgba:-','PNG32:'+path.join(root,name)],{input:data});
const reference=execFileSync('magick',[path.join(root,'../farmer/source/reference.png'),'-depth','8','rgba:-'],{maxBuffer:16*1024*1024});
const refWidth=1122;
const refPolygon=[[493,420],[514,385],[548,360],[587,344],[630,346],[660,370],[676,410],[699,416],[705,460],[711,493],[720,515],[716,560],[697,586],[668,603],[671,650],[580,650],[543,616],[513,589],[498,551],[482,540],[456,543],[426,532],[410,515],[408,482],[423,464],[447,460],[472,480],[485,474]];
const fringePolygon=[[474,435],[500,399],[553,365],[607,333],[650,312],[687,326],[710,366],[716,410],[699,432],[678,403],[654,371],[630,350],[588,368],[546,397],[510,438],[486,460]];
function inside(x,y,points) {
  let yes=false;
  for(let i=0,j=points.length-1;i<points.length;j=i++) {
    const [xi,yi]=points[i],[xj,yj]=points[j];
    if((yi>y)!==(yj>y)&&x<(xj-xi)*(y-yi)/(yj-yi)+xi) yes=!yes;
  }
  return yes;
}
const rect=(x,y,a,b,c,d)=>x>=a&&x<c&&y>=b&&y<d;
const raw=Buffer.alloc(w*h*4);
const registered=Buffer.alloc(raw.length);
for(let y=0;y<h;y++) for(let x=0;x<w;x++) {
  const rx=Math.floor((x+9)/.1),ry=Math.floor((y+16)/.1),i=(y*w+x)*4;
  const j=(ry*refWidth+rx)*4;
  if(rx>=0&&rx<1122&&ry>=0&&ry<1402&&reference[j+3]>=128) {
    reference.copy(registered,i,j,j+3);registered[i+3]=255;
  }
  const hair=inside(rx,ry,fringePolygon)&&Math.max(...reference.subarray(j,j+3))<185;
  if(!inside(rx,ry,refPolygon)&&!hair) continue;
  if(reference[j+3]<128) continue;
  reference.copy(raw,i,j,j+3);raw[i+3]=255;
  if(hair) {
    const light=(reference[j]+reference[j+1]+reference[j+2])/3;
    const gray=light<45?[33,31,34]:light<90?[64,62,67]:light<140?[98,96,102]:[133,129,135];
    for(let k=0;k<3;k++) raw[i+k]=gray[k];
  }
}
save('source/reference-face-raw.png',raw);
save('source/reference-registered.png',registered);
execFileSync('magick',[path.join(root,'source/reference-face-raw.png'),'-dither','None','-colors','40','PNG32:'+path.join(root,'source/reference-face-native.png')]);
const face=read('source/reference-face-native.png');
const head=read('source/face-locked/head-base.png');
const remove=[[43,20],[57,20],[66,26],[68,34],[68,45],[61,49],[42,49],[38,43],[34,40],[31,37],[31,32],[37,30],[40,33],[43,30]];
for(let y=0;y<h;y++) for(let x=0;x<w;x++) {
  const i=(y*w+x)*4;
  if(inside(x+.5,y+.5,remove)) head.fill(0,i,i+4);
  if(face[i+3]) face.copy(head,i,i,i+4);
}
const eye=Buffer.alloc(raw.length),mouth=Buffer.alloc(raw.length);
const skin=[249,170,82];
const faceColors=[];
for(let i=0;i<face.length;i+=4) if(face[i+3]) faceColors.push([...face.subarray(i,i+3)]);
const skinColor=faceColors.sort((a,b)=>a.reduce((s,v,k)=>s+(v-skin[k])**2,0)-b.reduce((s,v,k)=>s+(v-skin[k])**2,0))[0];
for(let y=0;y<h;y++) for(let x=0;x<w;x++) {
  const i=(y*w+x)*4,p=face.subarray(i,i+4);
  if(!p[3]) continue;
  const eyeArea=rect(x,y,45,22,55,26)||rect(x,y,58,24,63,27)||rect(x,y,45,27,55,35)||rect(x,y,58,27,63,34);
  const eyeFeature=eyeArea&&(Math.max(...p.subarray(0,3))<160||Math.min(...p.subarray(0,3))>195);
  const mouthFeature=rect(x,y,51,38,60,40)&&Math.max(...p.subarray(0,3))<125;
  if(eyeFeature||mouthFeature) {
    face.copy(eyeFeature?eye:mouth,i,i,i+4);
    for(let k=0;k<3;k++) head[i+k]=skinColor[k];
  }
}
save('head-base.png',head);save('eyes/open.png',eye);save('mouth/rest.png',mouth);
const compose=(...layers)=>{
  const out=Buffer.alloc(raw.length);
  for(const layer of layers) for(let i=0;i<out.length;i+=4) if(layer[i+3]) layer.copy(out,i,i,i+4);
  return out;
};
const lower=read('body/lower-base.png'),upper=read('body/upper-base.png');
const base=compose(lower,upper,head,eye,mouth);
save('base-neutral.png',base);
save('neutral.png',compose(base,read('equipment/apron.png'),read('equipment/belt-tools.png'),read('equipment/gloves.png'),read('equipment/hat.png')));
const colors=new Set();
for(const name of ['head-base.png','eyes/open.png','mouth/rest.png','body/upper-base.png','body/lower-base.png','equipment/hat.png','equipment/apron.png','equipment/belt-tools.png','equipment/gloves.png']) {
  const data=read(name);
  for(let i=0;i<data.length;i+=4) if(data[i+3]) colors.add('#'+data.subarray(i,i+4).toString('hex'));
}
writeFileSync(path.join(root,'source/palette.json'),JSON.stringify([...colors].sort(),null,2)+'\n');
console.log('Reference face composed; shared palette',colors.size);
