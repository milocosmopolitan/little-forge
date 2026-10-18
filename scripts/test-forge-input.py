#!/usr/bin/env python3
"""Real OS keyboard acceptance for the forge spaces under a disposable X11 display (#89, #96).

Drives the actual app: ambient exterior → Tab enters the workshop → 3 showroom → 1 outside →
2 workshop → Escape collapses → F12 diagnostics at each step read back real resource state.
The QA clock advances one eligible second per 30 frames, so the running job completes and
sells during the walk-through; the asserted counts prove the simulation never duplicated.
"""
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
out = Path('qa-output/forge-input')
out.mkdir(parents=True, exist_ok=True)
log = (out / 'app.log').open('w')
proc = subprocess.Popen([exe, '--safe-window', '--qa', 'forge-input', '--save-dir', str(out / 'session'),
                         '--screenshot', str(out / 'flow.png')], stdout=log, stderr=subprocess.STDOUT)
x = m.Input()


def wait_for(test, seconds=45):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            raise RuntimeError(f'app exited {proc.returncode}; see {out}/app.log')
        value = test()
        if value:
            return value
        time.sleep(.15)
    raise TimeoutError('forge acceptance timed out')


try:
    window = wait_for(x.find)
    time.sleep(3)

    def key(k):
        x.keys(window, [k])
        time.sleep(.7)

    def diagnostics():
        path = out / 'session/diagnostics.json'
        if path.exists():
            path.unlink()
        key('F12')
        wait_for(lambda: path.exists(), 10)
        return json.loads(path.read_text())

    r = diagnostics()
    assert r['space'] == 'Workshop' and 'expanded: true' in r['shell'], r['shell']
    key('F10')
    key('3')
    r = diagnostics()
    assert r['space'] == 'Showroom', r['space']
    key('1')
    assert diagnostics()['space'] == 'Exterior'
    key('2')
    r = diagnostics()
    assert r['space'] == 'Workshop'
    key('q')  # queue one more dagger through the real command path
    r = diagnostics()
    assert r['state']['command_id'] >= 2, r['state']['command_id']
    key('Escape')
    r = diagnostics()
    assert 'expanded: false' in r['shell'], r['shell']
    key('Tab')
    r = diagnostics()
    assert 'expanded: true' in r['shell'] and r['space'] == 'Workshop'
    # Every view read the same simulation: finished goods never exceed capacity, sales were
    # counted once, and the reservation ledger stayed consistent.
    s = r['state']
    room = r['showroom']
    assert room['displayed'] + room['held'] + room['overflow'] == s['daggers'], (room, s['daggers'])
    assert room['reserved'] == (1 if s['life']['work'] > 0 else 0)
    assert s['daggers'] == s['crafted'] - s['sold']
    (out / 'acceptance.json').write_text(json.dumps(r, indent=2))
    proc.terminate()
    proc.wait(timeout=10)
    print('Actual X11 keys: workshop → showroom → exterior → workshop → queue → collapse → expand; ledger consistent', flush=True)
finally:
    if proc.poll() is None:
        proc.terminate()
        proc.wait(timeout=10)
    log.close()
