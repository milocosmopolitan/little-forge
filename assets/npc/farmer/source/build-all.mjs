// Rebuild in separate processes so each ImageMagick pipeline closes before export.
import {execFileSync} from 'node:child_process';
for (const name of ['build-character.mjs', 'build-animation.mjs']) {
  execFileSync(process.execPath, [new URL(name, import.meta.url).pathname], {stdio: 'inherit'});
}
execFileSync('python3', [new URL('../../../../scripts/export-farmer-runtime.py', import.meta.url).pathname], {stdio: 'inherit'});
