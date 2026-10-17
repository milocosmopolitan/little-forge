// Rebuild approved base layers before deriving expressions, poses and the atlas.
await import('./build-character.mjs');
await import('./build-animation.mjs');
const {execFileSync} = await import('node:child_process');
execFileSync('python3', [new URL('../../../../scripts/export-farmer-runtime.py', import.meta.url).pathname], {stdio: 'inherit'});
