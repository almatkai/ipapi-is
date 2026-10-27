"""Collect verbatim license/notice texts from exact Cargo.lock packages; no invented grants."""
from pathlib import Path
import json,hashlib,shutil
R=Path(__file__).resolve().parents[1];meta=json.loads((R/'.qa/cargo-metadata.json').read_text());D=R/'notices/texts';D.mkdir(exist_ok=True)
records=[]
for p in sorted(meta['packages'],key=lambda p:(p['name'],p['version'])):
 if p['name']=='tezbar-ipapi-is':continue
 base=Path(p['manifest_path']).parent;found=[]
 for f in sorted(base.iterdir()):
  if f.is_file() and any(f.name.upper().startswith(n)for n in ['LICENSE','LICENCE','COPYING','NOTICE','COPYRIGHT']):
   data=f.read_bytes();digest=hashlib.sha256(data).hexdigest();(D/(digest+'.txt')).write_bytes(data);found.append({'original':f.name,'sha256':digest,'file':'texts/'+digest+'.txt'})
 records.append({'name':p['name'],'version':p['version'],'source':p['source'],'spdx':p.get('license'),'evidence':found,'missingLicenseText':not bool(found)})
(R/'notices/CARGO-DEPENDENCIES.json').write_text(json.dumps(records,indent=2)+'\n')
# The oracle's locked dependencies are QA-only and are not included in the binary.
lock=json.loads((R.parent/'package-lock.json').read_text());npm=[]
for name in ['@raycast/utils','@raycast/api','react','dequal','typescript','tsx']:
 base=R/'.qa/v1/node_modules'/name;p=json.loads((base/'package.json').read_text());texts=[]
 for f in sorted(base.iterdir()):
  if f.is_file()and any(f.name.upper().startswith(n)for n in ['LICENSE','COPYING','NOTICE']):
   data=f.read_bytes();digest=hashlib.sha256(data).hexdigest();(D/(digest+'.txt')).write_bytes(data);texts.append({'original':f.name,'sha256':digest,'file':'texts/'+digest+'.txt'})
 npm.append({'name':name,'version':p['version'],'spdx':p.get('license'),'lock':lock['packages']['node_modules/'+name],'evidence':texts,'missingLicenseText':not bool(texts)})
(R/'notices/ORACLE-DEPENDENCIES.json').write_text(json.dumps(npm,indent=2)+'\n')
print(f'{len(records)} locked Cargo packages; {sum(x["missingLicenseText"]for x in records)} without bundled license text (explicit).')
print(f'{len(npm)} direct oracle dependencies; missing text:',[x['name']for x in npm if x['missingLicenseText']])
