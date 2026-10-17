import {execFileSync} from 'node:child_process';
import {readFileSync,mkdirSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
export const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
export const W=96,H=128,N=W*H*4,blank=()=>Buffer.alloc(N),file=n=>path.join(root,n);
export const run=(args,input)=>execFileSync('magick',args.map(String),{input,maxBuffer:64*1024*1024});
export const read=n=>run([file(n),'-alpha','on','-depth','8','rgba:-']);
export const palette=JSON.parse(readFileSync(file('source/palette.json'))).map(c=>[...Buffer.from(c.slice(1),'hex')]);
const colorCache=new Map();
export function color(r,g,b){const key=(r<<16)|(g<<8)|b;if(colorCache.has(key))return colorCache.get(key);let best=palette[0],score=Infinity;for(const c of palette){const d=(r-c[0])**2+(g-c[1])**2+(b-c[2])**2;if(d<score){score=d;best=c;}}colorCache.set(key,best);return best;}
export function write(n,b){mkdirSync(path.dirname(file(n)),{recursive:true});run(['-size','96x128','-depth','8','rgba:-','-strip','PNG32:'+file(n)],b);}
export function over(...layers){const b=blank();for(const a of layers)for(let i=0;i<N;i+=4)if(a[i+3])a.copy(b,i,i,i+4);return b;}
export function select(a,fn){const b=blank();for(let y=0;y<H;y++)for(let x=0;x<W;x++){const i=(y*W+x)*4;if(a[i+3]&&fn(x,y,a.subarray(i,i+4)))a.copy(b,i,i,i+4);}return b;}
export function move(a,dx,dy){const b=blank();for(let y=0;y<H;y++)for(let x=0;x<W;x++){const i=(y*W+x)*4;if(!a[i+3])continue;const xx=x+dx,yy=y+dy;if(xx<0||yy<0||xx>=W||yy>=H)throw Error('Layer overflow '+[x,y,dx,dy]);a.copy(b,(yy*W+xx)*4,i,i+4);}return b;}
export function bounds(a){let x0=W,y0=H,x1=0,y1=0;for(let y=0;y<H;y++)for(let x=0;x<W;x++)if(a[(y*W+x)*4+3]){x0=Math.min(x0,x);y0=Math.min(y0,y);x1=Math.max(x1,x+1);y1=Math.max(y1,y+1);}return[x0,y0,x1,y1];}
export function candidate(n,{cw=96,ch=128,ox=0,oy=0,cols=1,rows=1,index=0}={}){
 const [sw,sh]=run(['identify','-format','%w %h',file(n)]).toString().split(' ').map(Number),raw=read(n),out=blank();
 const col=index%cols,row=Math.floor(index/cols);
 for(let y=0;y<ch;y++)for(let x=0;x<cw;x++){
  const sx=Math.floor((col+(x+.5)/cw)*sw/cols),sy=Math.floor((row+(y+.5)/ch)*sh/rows),i=(sy*sw+sx)*4,j=((y+oy)*W+x+ox)*4;
  if(raw[i+3]<128)continue;const c=color(raw[i],raw[i+1],raw[i+2]);for(let k=0;k<3;k++)out[j+k]=c[k];out[j+3]=255;
 }return out;
}
export function grow(a,r=1){const b=blank();for(let y=0;y<H;y++)for(let x=0;x<W;x++)if(a[(y*W+x)*4+3])for(let dy=-r;dy<=r;dy++)for(let dx=-r;dx<=r;dx++){const xx=x+dx,yy=y+dy;if(xx>=0&&xx<W&&yy>=0&&yy<H)b[(yy*W+xx)*4+3]=255;}return b;}
export const skin=c=>c[0]>175&&c[1]>100&&c[2]>55&&c[0]>c[1]*1.22;
export function sheet(n,paths,cols=3,scale=4,bg='#eee5cd'){run(['montage',...paths.map(file),'-filter','point','-background',bg,'-alpha','remove','-geometry',`${96*scale}x${128*scale}+6+6`,'-tile',`${cols}x`,'-strip',file('previews/'+n)]);}
