#!/usr/bin/env python3
"""Real production-clock desktop soak under a disposable X11 display; never uses --qa.
Run with xvfb-run; records process samples, journal continuity and scripted player keys.
"""
import argparse,json,os,subprocess,time
from pathlib import Path
import psutil
import importlib.util
spec=importlib.util.spec_from_file_location("x11_input",Path(__file__).with_name("x11-test-input.py")); module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
keyboard=module.Input()
p=argparse.ArgumentParser(); p.add_argument('executable'); p.add_argument('--seconds',type=int,default=1800); p.add_argument('--output',default='qa-output/soak'); a=p.parse_args()
out=Path(a.output).resolve(); out.mkdir(parents=True,exist_ok=True)
save=out/'save'; save.mkdir(exist_ok=True)
if list(save.glob('save-*.json')): raise SystemExit('Use a fresh output directory; do not reuse a prior soak save')
log=(out/'desktop.log').open('w')
app=subprocess.Popen([str(Path(a.executable).resolve()),'--safe-window','--save-dir',str(save)],stdout=log,stderr=subprocess.STDOUT)
process=psutil.Process(app.pid); process.cpu_percent(); samples=[]; actions=[]; start=time.monotonic(); window=None
try:
    while time.monotonic()-start<a.seconds:
        if app.poll() is not None: raise RuntimeError(f'app exited {app.returncode}')
        elapsed=time.monotonic()-start
        if window is None:
            window=keyboard.find()
        files=list(save.glob('save-*.json'))
        state=None
        if files:
            snapshots=[]
            for f in files:
                try: snapshots.append(json.loads(f.read_text()))
                except (OSError,json.JSONDecodeError): pass
            if snapshots: state=max(snapshots,key=lambda s:s['generation'])['state']
        if window and state and int(elapsed)%15==0:
            keys=['Tab']
            if state['ore']<2 and state['gold']>=4: keys.append('b')
            if state['life']['queue']<2: keys.append('q')
            if state['gold']>=2 and sum(s['count'] for s in state['life']['pantry']['slots'] if s and s['food']=='Staple')<3: keys.append('f')
            # Explicit player rest exercises the production sleep timer during the soak.
            if 300<=elapsed<301: keys.append('r')
            keyboard.keys(window,keys)
            actions.append({'elapsed':elapsed,'keys':keys})
        samples.append({'elapsed':elapsed,'cpu':process.cpu_percent(),'rss':process.memory_info().rss,'tick':state['tick'] if state else None})
        time.sleep(1)
    snapshots=[json.loads(f.read_text()) for f in save.glob('save-*.json')]; last=max(snapshots,key=lambda s:s['generation'])
    assert last['state']['tick']>=a.seconds-15, 'production clock did not advance with wall time'
    assert last['state']['sold']>1, 'scripted live queue/sale did not execute'
    result={'clock':'production monotonic, no --qa','wall_seconds':time.monotonic()-start,'platform':os.uname().sysname,'renderer':'virtual X11; see adapter in desktop.log','gpu':'not measured','state':last['state'],'samples':samples,'player_actions':actions,'rss_peak':max(s['rss'] for s in samples),'cpu_mean':sum(s['cpu'] for s in samples)/len(samples)}
    (out/'report.json').write_text(json.dumps(result,indent=2))
finally:
    app.terminate()
    try: app.wait(timeout=10)
    except subprocess.TimeoutExpired: app.kill(); app.wait()
    log.close()
