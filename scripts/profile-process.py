#!/usr/bin/env python3
"""Sample a running build without changing its interaction/focus state. Requires psutil."""
import argparse,json,platform,time,statistics
from pathlib import Path
import psutil
p=argparse.ArgumentParser();p.add_argument('pid',type=int);p.add_argument('--seconds',type=int,default=60);p.add_argument('--scenario',required=True,choices=['ambient','expanded','unfocused','minimized','resumed']);p.add_argument('--output',required=True);a=p.parse_args()
process=psutil.Process(a.pid);process.cpu_percent();rows=[]
for _ in range(a.seconds):
 time.sleep(1);rows.append({'cpu_percent':process.cpu_percent(),'rss_bytes':process.memory_info().rss})
result={'platform':platform.platform(),'scenario':a.scenario,'seconds':len(rows),'cpu_definition':'100 percent = one logical CPU','cpu_mean':statistics.mean(x['cpu_percent'] for x in rows),'cpu_peak':max(x['cpu_percent'] for x in rows),'rss_peak_bytes':max(x['rss_bytes'] for x in rows),'gpu':'not measured; use native tooling','samples':rows}
Path(a.output).parent.mkdir(parents=True,exist_ok=True);Path(a.output).write_text(json.dumps(result,indent=2));print(a.output)
