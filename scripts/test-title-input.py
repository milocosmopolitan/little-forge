"""Run only on a disposable X11 display; verify actual title routing and capture it."""
import importlib.util, os, subprocess, time, json
from pathlib import Path
from PIL import ImageGrab
spec=importlib.util.spec_from_file_location('xinput',Path(__file__).with_name('x11-test-input.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
out=Path('qa-output/title-input');out.mkdir(parents=True,exist_ok=True)
# Every run starts in a fresh, isolated save directory.
save=out/str(time.time_ns())
with (out/'runtime.log').open('w') as log:
    p=subprocess.Popen(['target/debug/little-forge','--safe-window','--save-dir',str(save)],stdout=log,stderr=log)
    try:
        x=m.Input();w=None
        for _ in range(100):
            if p.poll() is not None: raise RuntimeError('App exited before input')
            w=x.find()
            if w: break
            time.sleep(.2)
        assert w
        x.x.XSetInputFocus(x.d,w,1,0);x.x.XFlush(x.d)
        time.sleep(5)
        def key(k):
            code=x.x.XKeysymToKeycode(x.d,x.x.XStringToKeysym(k.encode()))
            x.t.XTestFakeKeyEvent(x.d,code,1,0);x.x.XFlush(x.d);time.sleep(.2)
            x.t.XTestFakeKeyEvent(x.d,code,0,0);x.x.XFlush(x.d);time.sleep(1)
        def capture(name): ImageGrab.grab(xdisplay=os.environ['DISPLAY']).save(out/(name+'.png'))
        capture('01-title')
        assert not list(save.glob('save-*.json')), 'Fresh title must not create a save'
        key('Return');capture('02-new')
        key('Return');capture('02b-choose')
        assert not list(save.glob('save-*.json')), 'Choosing a smith must not write before the pick'
        key('Right');key('Left');key('Return');time.sleep(1);capture('03-game')
        assert list(save.glob('save-*.json')), 'Picking a smith must persist the first journey'
        chosen=json.loads(sorted(save.glob('save-*.json'))[0].read_text())['state']['smith']
        assert chosen=='female', chosen
        key('Tab');key('m');capture('03b-village')
        key('F9');capture('04-return')
        key('Down');key('Down');key('Down');key('Return');capture('05-settings')
        key('Return');capture('06-still')
        key('Escape');key('Up');key('Return');capture('07-quit-confirm')
        key('Escape');capture('08-cancel-quit')
        assert p.poll() is None, 'Escape must not quit'
        key('Up');key('Return');key('Return')
        p.wait(timeout=20);assert p.returncode==0
    finally:
        if p.poll() is None: p.terminate();p.wait(timeout=10)
log=(out/'runtime.log').read_text()
for marker in ['Title route: New','Title route: Choose','Title smith: Female','Title route: InGame','Title route: Title (F9)','Title route: Settings','Title still effects: true','Title route: QuitConfirm','Title route: Title (Escape)']:
    assert marker in log, marker
assert 'panicked' not in log and 'ERROR' not in log
(out/'result.json').write_text(json.dumps({'passed':True,'checks':['fresh title does not write save','choose screen writes nothing','picked smith persists','F9 recovery','settings/still mode','escape cancels quit','save and quit'],'display':os.environ['DISPLAY']},indent=2))
print('Title input acceptance passed')
