import {execFileSync} from 'node:child_process';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const dir=path.dirname(fileURLToPath(import.meta.url));
for(const name of ['build-character.mjs','build-animation.mjs'])execFileSync(process.execPath,[path.join(dir,name)],{stdio:'inherit'});
