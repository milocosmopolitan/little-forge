// Sample the full-resolution generated sources, never upscale the older 64x96 art.
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
// Reserve neutral hair shades so palette reduction cannot turn hair olive/brown.
const palette=['150909','30231d','3d4230','483e41','4b4f35','4c362f','5c5c4a','5e301f','69534c','696a4b','728c36','734730','7a774d','8e5733','917351','a38751','a69286','b46f36','c6b2a0','c87837','dccab9','e17745','e38a4c','f3e6d5','f8ab55','fbb568','fbf9f6','fca862','211f22','403e43','626066','858187'];
const bytes=Buffer.from(palette.join(''),'hex');
const palettePath=path.join(root,'source/native-palette.png');
execFileSync('magick',['-size','32x1','-depth','8','rgb:-',palettePath],{input:bytes});
function thinLines(input) {
  const mask=Uint8Array.from(input);
  let changed=true;
  while(changed) {
    changed=false;
    for(let pass=0;pass<2;pass++) {
      const remove=[];
      for(let y=1;y<127;y++) for(let x=1;x<95;x++) {
        const at=y*96+x;
        if(!mask[at]) continue;
        const n=[mask[at-96],mask[at-95],mask[at+1],mask[at+97],mask[at+96],mask[at+95],mask[at-1],mask[at-97]];
        const count=n.reduce((a,b)=>a+b,0);
        const transitions=n.reduce((a,b,i)=>a+(!b&&n[(i+1)%8]?1:0),0);
        if(count<2||count>6||transitions!==1) continue;
        if(pass===0 ? (n[0]*n[2]*n[4]||n[2]*n[4]*n[6]) : (n[0]*n[2]*n[6]||n[0]*n[4]*n[6])) continue;
        remove.push(at);
      }
      for(const at of remove) mask[at]=0;
      if(remove.length) changed=true;
    }
  }
  return mask;
}
for(const [input,output] of [['generated-master.png','master-native.png'],['generated-underlayer.png','underlayer-native.png'],['generated-apron-underlay.png','apron-underlay-native.png']]) {
  const pixels=execFileSync('magick',['-size','96x128','canvas:none','(',path.join(root,'source',input),'-filter','point','-resize','90x135!',')','-geometry','+3-5','-compose','Over','-composite','-depth','8','rgba:-']);
  const result=Buffer.alloc(pixels.length);
  for(let i=0;i<pixels.length;i+=4) {
    if(pixels[i+3]<128) continue;
    const rgb=[...pixels.subarray(i,i+3)];
    const neutralHair=Math.floor(i/4/96)<48 && Math.max(...rgb)-Math.min(...rgb)<20 && Math.max(...rgb)>20 && Math.max(...rgb)<165;
    let best=0, distance=Infinity;
    for(let c=0;c<palette.length;c++) {
      if(neutralHair && c<28) continue;
      const d=rgb.reduce((sum,v,k)=>sum+(v-bytes[c*3+k])**2,0);
      if(d<distance) {distance=d;best=c;}
    }
    bytes.copy(result,i,best*3,best*3+3);result[i+3]=255;
  }
  // A one-pixel silhouette ring is distinct from material shadows inside the shape.
  const mapped=Buffer.from(result);
  const darkMask=new Uint8Array(96*128);
  const borderMask=new Uint8Array(96*128);
  for(let y=0;y<128;y++) for(let x=0;x<96;x++) {
    const p=y*96+x,i=p*4;
    if(!mapped[i+3]) continue;
    borderMask[p]=[[1,0],[-1,0],[0,1],[0,-1]].some(([dx,dy])=>x+dx<0||x+dx>=96||y+dy<0||y+dy>=128||!mapped[((y+dy)*96+x+dx)*4+3])?1:0;
    if(y>=49&&((mapped[i]===21&&mapped[i+1]===9&&mapped[i+2]===9)||(mapped[i]===33&&mapped[i+1]===31&&mapped[i+2]===34))) darkMask[p]=1;
  }
  const seams=thinLines(darkMask);
  for(let y=0;y<128;y++) for(let x=0;x<96;x++) {
    const i=(y*96+x)*4;
    if(!mapped[i+3]) continue;
    const border=borderMask[y*96+x];
    const face=x>=45&&x<=63&&y>=25&&y<=45;
    const hair=y<48&&x>=28&&x<=68&&y>=14&&!face;
    if(border) {
      bytes.copy(result,i,(hair?28:0)*3,(hair?28:0)*3+3);
    } else if(!face && ((mapped[i]===21&&mapped[i+1]===9&&mapped[i+2]===9)||(mapped[i]===33&&mapped[i+1]===31&&mapped[i+2]===34))) {
      // Interior seams and occlusion boundaries are lines, not material shadows.
      const besideBorder=[[1,0],[-1,0],[0,1],[0,-1]].some(([dx,dy])=>borderMask[(y+dy)*96+x+dx]);
      if(y>=49&&seams[y*96+x]&&!besideBorder) {
        bytes.copy(result,i,0,3);
        continue;
      }
      let shade=11;
      if(hair) shade=29;
      else if(y<48) shade=14;
      else if(y<76) shade=(input!=='generated-underlayer.png'&&x>=37&&x<=67&&y>56)?2:18;
      else if(input!=='generated-underlayer.png'&&x>=35&&x<=70&&y<100) shade=2;
      bytes.copy(result,i,shade*3,shade*3+3);
    }
  }
  // Preserve binary alpha separately from palette mapping.
  execFileSync('magick',['-size','96x128','-depth','8','rgba:-','PNG32:'+path.join(root,'source',output)],{input:result});
}
