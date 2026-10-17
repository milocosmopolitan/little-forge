// Mechanical extraction from generated master/underlayer, on one native canvas.
import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const width = 96, height = 128;
const read = name => execFileSync('magick', [path.join(root, name), '-depth', '8', 'rgba:-']);
const master = read('source/master-native.png');
const base = read('source/underlayer-native.png');
const apronSource = read('source/apron-underlay-native.png');
const empty = () => Buffer.alloc(width * height * 4);
const index = (x, y) => (y * width + x) * 4;
const rect = (x, y, x0, y0, x1, y1) => x >= x0 && x < x1 && y >= y0 && y < y1;
function inside(x, y, polygon) {
  let result = false;
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    const [xi, yi] = polygon[i], [xj, yj] = polygon[j];
    if ((yi > y) !== (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi) result = !result;
  }
  return result;
}
function extract(source, predicate) {
  const result = empty();
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    const i = index(x, y);
    // Keep source-space masks registered when rebuilding at the larger native size.
    const mx=(x-3)*64/90, my=(y+5)*96/135;
    if (source[i + 3] && predicate(mx, my, source.subarray(i, i + 4))) source.copy(result, i, i, i + 4);
  }
  return result;
}
function composite(...layers) {
  const result = empty();
  for (const layer of layers) for (let i = 0; i < result.length; i += 4) {
    if (layer[i + 3]) layer.copy(result, i, i, i + 4);
  }
  return result;
}
function save(name, buffer) {
  execFileSync('magick', ['-size', `${width}x${height}`, '-depth', '8', 'rgba:-', `PNG32:${path.join(root, name)}`], { input: buffer });
}
function mainComponent(buffer) {
  const visited = new Set(), components = [];
  for(let p=0;p<width*height;p++) {
    if(visited.has(p) || !buffer[p*4+3]) continue;
    const component=[], queue=[p]; visited.add(p);
    while(queue.length) {
      const current=queue.pop(); component.push(current);
      const x=current%width, y=Math.floor(current/width);
      for(let dy=-1;dy<=1;dy++) for(let dx=-1;dx<=1;dx++) {
        const nx=x+dx,ny=y+dy,next=ny*width+nx;
        if(nx<0||nx>=width||ny<0||ny>=height||visited.has(next)||!buffer[next*4+3]) continue;
        visited.add(next);queue.push(next);
      }
    }
    components.push(component);
  }
  const keep=new Set(components.sort((a,b)=>b.length-a.length)[0]);
  const result=empty();
  for(const p of keep) buffer.copy(result,p*4,p*4,p*4+4);
  return result;
}
const hatPolygon = [[12,26],[19,21],[18,16],[16,15],[18,14],[18,10],[21,12],[25,8],[29,6],[35,6],[39,12],[40,15],[43,14],[45,17],[51,18],[52,25],[48,29],[44,30],[44,25],[42,21],[39,19],[35,19],[31,22],[27,24],[26,27],[22,29],[22,33],[18,33],[12,30]];
const apronPolygon = [[25,36],[30,38],[30,43],[40,43],[40,38],[44,39],[46,45],[46,57],[48,74],[23,74],[20,71],[22,63],[22,57],[25,54]];
const head = mainComponent(composite(
  extract(base, (x, y) => y < 38),
  // Preserve the generated nose that the feature-removal edit also erased.
  extract(master, (x, y) => rect(x,y,37,28,42,32)),
));
const upper = mainComponent(extract(base, (x,y) => y >= 38 && (y < 58 || (x < 24 && y < 66) || (x > 47 && y < 72))));
const lower = extract(base, (x,y) => (y >= 58 && x >= 24 && x <= 47) || (y >= 66 && x < 24) || (y >= 72 && x > 47));
const eyes = extract(master, (x,y,p) =>
  (rect(x,y,31,22,37,29) || rect(x,y,39,24,42,29)) &&
  (Math.max(...p.subarray(0,3)) < 105 || (p[0] > 200 && p[1] > 180 && p[2] > 140)));
const mouth = extract(master, (x,y,p) => rect(x,y,34,32,41,35) && Math.max(...p.subarray(0,3)) < 110);
const hat = extract(master, (x,y) => inside(x+.5,y+.5,hatPolygon));
const apronOnly = extract(apronSource, (x,y) => inside(x+.5,y+.5,apronPolygon));
const beltTools = extract(master,(x,y,p)=>
  (rect(x,y,23,54,47,58)||rect(x,y,25,52,32,68)) &&
  ((p[0]>p[1]*1.15&&p[0]<210) || Math.max(...p.subarray(0,3))<45 ||
    (rect(x,y,26,58,30,62)&&Math.min(...p.subarray(0,3))>180)));
const apron=composite(apronOnly,beltTools);
const glovePolygons=[[[13,55],[21,55],[22,57],[25,59],[25,64],[23,67],[17,67],[14,64]],[[47,59],[52,59],[55,64],[55,70],[53,73],[48,71]]];
const gloves = extract(master, (x,y) => glovePolygons.some(polygon=>inside(x+.5,y+.5,polygon)));
const layers = {'head-base.png':head,'body/upper-base.png':upper,'body/lower-base.png':lower,'eyes/open.png':eyes,'mouth/rest.png':mouth,'equipment/hat.png':hat,'equipment/apron-belt.png':apron,'equipment/apron.png':apronOnly,'equipment/belt-tools.png':beltTools,'equipment/gloves.png':gloves};
for (const [name, layer] of Object.entries(layers)) save(name, layer);
save('base-neutral.png', composite(lower,upper,head,eyes,mouth));
save('neutral.png', composite(lower,upper,head,eyes,mouth,apron,gloves,hat));
const palette = new Set();
for (const buffer of Object.values(layers)) for(let i=0;i<buffer.length;i+=4) if(buffer[i+3]) palette.add('#'+buffer.subarray(i,i+4).toString('hex'));
writeFileSync(path.join(root,'source/palette.json'), JSON.stringify([...palette].sort(),null,2)+'\n');
// Apply the registered original face after packing the unchanged body/equipment.
execFileSync(process.execPath,[path.join(root,'source/restore-reference-face.mjs')]);
