#!/usr/bin/env python3
"""Real OS key/pointer acceptance in disposable X11, never the user's desktop.

Run under Xvfb. QA has an isolated fixed-clock 24-gold fixture; restart persistence
is independently tested in Rust. Diagnostics read back real observer/domain state.
"""
import ctypes as c
import importlib.util
import json
import subprocess
import sys
import time
from pathlib import Path
spec = importlib.util.spec_from_file_location('xinput', Path(__file__).with_name('x11-test-input.py'))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)

exe = str(Path(sys.argv[1]).resolve())
out = Path('qa-output/village-input')
out.mkdir(parents=True, exist_ok=True)
# No existing save is used by --qa. Distinct directory also holds diagnostics.
log = (out/'app.log').open('w')
proc = subprocess.Popen([exe, '--safe-window', '--qa', 'village-input', '--save-dir', str(out/'session'), '--screenshot', str(out/'flow.png')], stdout=log, stderr=subprocess.STDOUT)
x = m.Input()
def wait_for(test, seconds=45):
    deadline = time.monotonic()+seconds
    while time.monotonic()<deadline:
        if proc.poll() is not None:
            raise RuntimeError(f'app exited {proc.returncode}; see {out}/app.log')
        value = test()
        if value: return value
        time.sleep(.15)
    raise TimeoutError('app acceptance timed out')
try:
    window = wait_for(x.find)
    time.sleep(3)  # asset upload/layout, measured on software renderer
    def key(k):
        x.keys(window,[k]);time.sleep(.7)
    def diagnostics():
        path=out/'session/diagnostics.json'
        if path.exists(): path.unlink()
        key('F12')
        wait_for(lambda:path.exists(),10)
        return json.loads(path.read_text())
    key('2')  # real navigation input to carpenter
    key('Return')
    r=diagnostics();s=r['state'];assert s['commerce']['timber']==1,(r['village_screen'],s)
    assert r['village_screen']=='Destination(Carpenter)'
    # Mouse hits letterboxed actual map: center well is unmapped; carpenter polygon is pickable.
    key('m')
    x.t.XTestFakeMotionEvent.argtypes=[c.c_void_p,c.c_int,c.c_int,c.c_int,c.c_ulong]
    x.t.XTestFakeButtonEvent.argtypes=[c.c_void_p,c.c_uint,c.c_int,c.c_ulong]
    x.x.XTranslateCoordinates.argtypes=[c.c_void_p,c.c_ulong,c.c_ulong,c.c_int,c.c_int,c.POINTER(c.c_int),c.POINTER(c.c_int),c.POINTER(c.c_ulong)]
    ox=c.c_int();oy=c.c_int();child=c.c_ulong()
    x.x.XTranslateCoordinates(x.d,window,x.x.XDefaultRootWindow(x.d),0,0,c.byref(ox),c.byref(oy),c.byref(child))
    # Map image rectangle in the 960x540 layout is (152,67)-(378,349).
    # Point (0.69,0.48) is the carpenter yard; it is traced in locations.json.
    x.t.XTestFakeMotionEvent(x.d,0,ox.value+308,oy.value+202,0);x.x.XFlush(x.d);time.sleep(.7)
    key('F10')  # preserve real pointer-hover before navigation
    x.t.XTestFakeButtonEvent(x.d,1,1,0);x.t.XTestFakeButtonEvent(x.d,1,0,0);x.x.XFlush(x.d);time.sleep(.7)
    assert diagnostics()['village_screen']=='Destination(Carpenter)'
    key('Escape');key('4');key('Return')
    r=diagnostics();assert r['village_screen']=='Destination(Farm)';assert r['state']['commerce']['stock'][2]==59
    key('Escape');key('5');key('Return')
    r=diagnostics();assert r['village_screen']=='Destination(Tavern)';assert r['state']['commerce']['stock'][3]==11
    key('Escape');key('1')
    r=diagnostics();assert r['village_screen']=='Home';assert r['state']['tick']>0
    # Home selection returns to working management; explicit close collapses it.
    key('Escape');r=diagnostics();assert 'expanded: false' in r['shell']
    (out/'acceptance.json').write_text(json.dumps(r,indent=2))
    proc.terminate()  # end isolated acceptance process after assertions; no user save is involved
    proc.wait(timeout=10)
    print('Actual X11 keys + pointer: map → carpenter → buy → farm → buy → tavern → buy → home → close; simulation advanced',flush=True)
finally:
    if proc.poll() is None: proc.terminate();proc.wait(timeout=10)
    log.close()
