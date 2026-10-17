import fs from 'node:fs';import path from 'node:path';
import {read,save,over} from '../raster.mjs';import {restoreResearchMaterials} from './materials.mjs';
const root=path.resolve('assets/characters/female'),out=path.resolve('assets/characters/source/animation/research-material-v13/candidates');
const a=JSON.parse(fs.readFileSync(path.join(root,'animation.json')));
for(const[k,index]of [[0,0],[1,1],[2,2],[4,3]]){
 const p=path.join(root,'poses/researching-'+k),load=n=>read(path.join(p,n));
 const patched=restoreResearchMaterials({upper:load('rendered/body/upper.png'),neck:load('rendered/body/neck-skin.png'),garment:load('rendered/body/source-connection.png'),required:load('rendered/body/required-coverage.png'),point:a.poses['researching-'+k].neck_assembly.body_owned_mount.equipped_point},load('source/equipped-before-head-restoration.png'),index,true,read(path.join(root,'head-base.png')));
 save(path.join(out,k+'-upper.png'),patched.upper);save(path.join(out,k+'-straps.png'),patched.restored);
 save(path.join(out,k+'-composed.png'),over(load('rendered/body/lower.png'),patched.upper,load('rendered/head-base.png'),load('rendered/eyes/open.png'),load('rendered/mouth/rest.png')));
}
