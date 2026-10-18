#!/usr/bin/env python3
"""Create portable Windows folder or unsigned macOS .app; run from repository root."""
import platform,shutil,json,subprocess,plistlib
from pathlib import Path
system=platform.system()
base=Path('dist');base.mkdir(exist_ok=True)
name='little-forge-'+system.lower()
stage=base/name
if stage.exists():shutil.rmtree(stage)
exe='little-forge'+('.exe' if system=='Windows' else '')
if system=='Darwin':
    contents=stage/'Little Forge.app'/'Contents';dest=contents/'MacOS';dest.mkdir(parents=True)
    (contents/'Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':'com.omiologic.littleforge','CFBundleName':'Little Forge','CFBundleExecutable':exe,'CFBundleVersion':'0.1.0','CFBundlePackageType':'APPL','NSHighResolutionCapable':True}))
else:dest=stage;dest.mkdir(parents=True)
shutil.copy2(Path('target/release')/exe,dest/exe)
preview='npc-preview'+('.exe' if system=='Windows' else '')
shutil.copy2(Path('target/release')/preview,dest/preview)
shutil.copy2('docs/npc/PREVIEW.md',stage/'NPC-PREVIEW.md')
ui_preview='ui-preview'+('.exe' if system=='Windows' else '')
shutil.copy2(Path('target/release')/ui_preview,dest/ui_preview)
shutil.copy2('docs/ui/PREVIEW.md',stage/'UI-PREVIEW.md')
shutil.copytree('assets',dest/'assets')
shutil.copytree('docs/licenses',stage/'licenses')
shutil.copy2('docs/qa/HANDOFF.md',stage/'READ-ME-FIRST.md')
shutil.copy2('docs/qa/LIFESTYLE.md',stage/'LIFESTYLE.md')
shutil.copy2('docs/qa/VILLAGE.md',stage/'VILLAGE.md')
shutil.copy2('docs/qa/FORGE_SPACES.md',stage/'FORGE-SPACES.md')
shutil.copy2('docs/qa/EXPEDITION-COMBAT.md',stage/'EXPEDITION-COMBAT.md')
(stage/'build.json').write_text(json.dumps({'commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'platform':system,'arch':platform.machine(),'signed':False},indent=2))
shutil.make_archive(str(base/name),'zip',base_dir=name,root_dir=base)
shutil.rmtree(stage)
print(base/(name+'.zip'))
