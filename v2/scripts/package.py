"""Ignored local ARM64 development ZIP only. No URL, install, catalog, or release."""
import hashlib,json,os,shutil,stat,subprocess,uuid,zipfile
from pathlib import Path
R=Path(__file__).resolve().parents[1];Q=R/'.qa';D=R/'dist';D.mkdir(exist_ok=True)
exe=Q/'target-fresh/release/tezbar-ipapi-is';assert exe.is_file();assert 'arm64'in subprocess.check_output(['file',str(exe)],text=True)
files={'bin/tezbar-ipapi-is':exe,'resources/ipapi-is.png':R/'resources/ipapi-is.png','LICENSE':R/'LICENSE','NOTICE':R/'NOTICE'}
for p in (R/'notices').rglob('*'):
 if p.is_file():files['notices/'+p.relative_to(R/'notices').as_posix()]=p
for name,p in files.items():assert not p.is_symlink();assert '..'not in Path(name).parts
manifest=json.loads((R/'extension.json').read_text());manifest['files']={n:hashlib.sha256(p.read_bytes()).hexdigest()for n,p in sorted(files.items())};assert manifest['artifacts']=={}
archive=D/'ipapi-is-aarch64-apple-darwin-DEVELOPMENT.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED)as z:
 for name,p in sorted(files.items()):
  i=zipfile.ZipInfo(name,(2026,1,1,0,0,0));i.external_attr=(stat.S_IFREG|(0o755 if name.startswith('bin/')else 0o644))<<16;z.writestr(i,p.read_bytes(),compress_type=zipfile.ZIP_DEFLATED)
 z.writestr('extension.json',json.dumps(manifest,indent=2)+'\n')
extract=Q/'extracted-package';extract.mkdir(exist_ok=True)
with zipfile.ZipFile(archive)as z:
 assert set(z.namelist())==set(manifest['files'])|{'extension.json'};z.extractall(extract)
 for n,digest in manifest['files'].items():assert hashlib.sha256((extract/n).read_bytes()).hexdigest()==digest
binary=extract/'bin/tezbar-ipapi-is';binary.chmod(0o755)
# Actual extracted production executable, ordinary JSONL; invalid input blocks network.
p=subprocess.Popen([str(binary)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env={**os.environ,'HOME':str(Q/'home'),'TMPDIR':str(Q/'tmp')},text=True)
def call(method,params):
 rid='h:'+str(uuid.uuid4());p.stdin.write(json.dumps({'type':'request','id':rid,'method':method,'params':params})+'\n');p.stdin.flush();r=json.loads(p.stdout.readline());assert r['id']==rid;return r
assert json.loads(p.stdout.readline())['id']==manifest['id']
assert call('initialize',{'protocol':1,'extensionId':manifest['id'],'platform':'macOS','packageDir':str(extract),'dataDir':str(Q/'package-profile'),'tempDir':str(Q/'tmp'),'grantedCapabilities':[]})['result']['ready']
v=call('session.open',{'sessionId':'smoke','commandName':'lookup-ip-or-asn','preferences':{},'argumentValues':{'ip':'bad\ninput'}})['result'];assert v['mode']=='view'and v['actions'][0]['title']=='Retry IP or ASN';assert v['refreshMs']is None
assert 'error'in call('session.call',{'sessionId':'smoke','operation':'extension:invoke-action','payload':{'actionId':'forged'}})
call('session.dispose',{'sessionId':'smoke'});p.stdin.close();assert p.wait(timeout=8)==0
report={'kind':'unpublished development ZIP, not installed','target':'aarch64-apple-darwin','archive':archive.name,'archiveSha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'executableSha256':manifest['files']['bin/tezbar-ipapi-is'],'files':manifest['files'],'membersIncludingManifest':len(files)+1,'extractedExecutableSmoke':'passed: initialize, recoverable invalid-input view, forged action rejection, dispose, EOF; no API or OS effect','notRun':['Public API success in production client','real desktop UI','Windows artifact/execution']}
(R/'evidence/package.json').write_text(json.dumps(report,indent=2)+'\n');print('Development ZIP closed inventory and extracted executable smoke passed:',report['archiveSha256'])
