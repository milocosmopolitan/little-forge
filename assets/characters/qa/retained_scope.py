"""Exclude only the explicitly superseded walk/give frames from older baselines."""
import json,re

def current_retained(root, snapshots):
    retained={};superseded=[]
    for name,digest in snapshots.items():
        match=re.fullmatch(r'(female|male)/frames/(?:base/)?(walk-[0-5]|give-half|give-full|give-release)\.png',name)
        if match:
            sex,key=match.groups()
            data=json.loads((root/sex/'animation.json').read_text())
            if data['poses'][key].get('layer_finish')in {'mobile-poses-v14','collar-v15'}:
                superseded.append(name);continue
        retained[name]=digest
    return retained,superseded
