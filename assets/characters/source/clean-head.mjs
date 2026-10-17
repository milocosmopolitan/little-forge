// Keep the approved silhouette, sockets and separate expression layers fixed.
// Only generated material clusters inside the head are adopted.
export function cleanHead({id,file,sample,remap,colors,old}) {
 const candidate=sample(file('source/revision-2/generated-head.png'),(x,y,w,h)=>[(x-8)*w/80,y*h/56]);
 const nearest=hex=>{const rgb=[...Buffer.from(hex,'hex')];return colors.reduce((a,b)=>distance(rgb,a)<distance(rgb,b)?a:b);};
 const distance=(a,b)=>a.reduce((s,v,k)=>s+(v-b[k])**2,0);
 const hair=['281713','60301e','9d4c26','c46b35','e28b47'].map(nearest);
 const skin=['fdb773','f4a565','de8855','e17d50'].map(nearest);
 const out=Buffer.from(old);
 for(let y=0;y<46;y++)for(let x=0;x<96;x++){
  const i=(y*96+x)*4;if(!old[i+3]||!candidate[i+3])continue;
  // Retain the exact outer contour; the generated candidate owns interior shading.
  const boundary=[[-1,0],[1,0],[0,-1],[0,1]].some(([dx,dy])=>x+dx<0||x+dx>=96||y+dy<0||!old[((y+dy)*96+x+dx)*4+3]);
  if(boundary)continue;
  const p=[...candidate.subarray(i,i+3)];
  const isSkin=p[0]>185&&p[1]>100&&p[2]>60&&p[1]>p[0]*.5;
  const q=(isSkin?skin:hair).reduce((a,b)=>distance(p,a)<distance(p,b)?a:b);
  for(let k=0;k<3;k++)out[i+k]=q[k];
 }
 return remap(out,colors);
}
