#!/usr/bin/env python3
"""Capture isolated shared-player stories; no game simulation or user saves."""
import os,subprocess,sys
from pathlib import Path
exe=Path(sys.argv[1] if len(sys.argv)>1 else 'target/debug/npc-preview').resolve()
out=Path(sys.argv[2] if len(sys.argv)>2 else 'qa-output/npc');out.mkdir(parents=True,exist_ok=True)
for story in ['idle','talk','blink','surprised','nod','shake_head','walk','give']:
    at=.38 if story=='shake_head' else .3
    command=[str(exe),'--story',story,'--at',str(at),'--capture',str(out/f'{story}.png')]
    with (out/f'{story}.log').open('w') as log:subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=90)
for name,story,at,extra in [('talk-blink','talk',3.33,[]),('give-left','give',.3,['--left']),('give-after','give',.8,[])]:
    with (out/f'{name}.log').open('w') as log:subprocess.run([str(exe),'--story',story,'--at',str(at),'--capture',str(out/f'{name}.png'),*extra],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=90)
print('Captured standalone NPC stories in',out)
