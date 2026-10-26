"""Reproducible checks. All writable homes/caches/logs live in ignored .qa/.
No installs/releases/publication; npm ci only installs the existing exact V1 lock.
"""
from pathlib import Path
import json,os,subprocess,sys,time
R=Path(__file__).resolve().parents[1];Q=R/'.qa'
original_home=Path.home();env=os.environ.copy()
for name in ['home','tmp','cache','cargo-home']:(Q/name).mkdir(parents=True,exist_ok=True)
env.update(HOME=str(Q/'home'),TMPDIR=str(Q/'tmp'),XDG_CACHE_HOME=str(Q/'cache'),CARGO_HOME=str(Q/'cargo-home'),RUSTUP_HOME=env.get('RUSTUP_HOME',str(original_home/'.rustup')),npm_config_cache=str(Q/'cache/npm'))
checks=[]
def run(name,args,extra=None,required=True):
 started=time.monotonic();out=Q/(name+'.log')
 with out.open('wb')as f:p=subprocess.run(args,cwd=R,env={**env,**(extra or {})},stdout=f,stderr=subprocess.STDOUT)
 checks.append({'name':name,'command':args,'status':'passed'if p.returncode==0 else'failed','exitCode':p.returncode,'seconds':round(time.monotonic()-started,2)})
 print(name,checks[-1]['status'],flush=True)
 if required and p.returncode:print(out.read_text()[-3000:]);raise SystemExit(p.returncode)
# Extract disposable V1 from original immutable commit. Do not replace the root lock.
PIN='401650f61ca3eb1b9a77bd6764007e0f2075387f';v=Q/'v1';v.mkdir(exist_ok=True)
for name in subprocess.check_output(['git','ls-tree','-r','--name-only',PIN],cwd=R.parent,text=True).splitlines():
 p=v/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(subprocess.check_output(['git','show',PIN+':'+name],cwd=R.parent))
run('v1-install',['npm','ci','--prefix',str(v),'--ignore-scripts','--legacy-peer-deps','--no-audit','--no-fund'])
run('v1-test',['npm','--prefix',str(v),'test']);run('v1-typecheck',['npm','--prefix',str(v),'run','typecheck'])
run('oracle',['node','scripts/oracle.cjs']);run('renderer',['node','scripts/renderer.cjs'])
run('fmt',['cargo','fmt','--all','--','--check'])
run('stable-tests',['cargo','test','--locked','--all-targets','--all-features'])
run('clippy',['cargo','clippy','--locked','--all-targets','--all-features','--','-D','warnings'])
run('msrv-tests',['cargo','+1.85.0','test','--locked','--all-targets','--all-features'],{'CARGO_TARGET_DIR':str(Q/'target-msrv')})
run('msrv-clippy',['cargo','+1.85.0','clippy','--locked','--all-targets','--all-features','--','-D','warnings'],{'CARGO_TARGET_DIR':str(Q/'target-msrv')})
run('build',['cargo','build','--locked']);run('build-pipe-fixture',['cargo','build','--locked','--features','qa','--example','pipe-fixture']);run('pipes',['python3','scripts/pipes.py'])
installed=subprocess.check_output(['rustup','target','list','--installed'],env=env,text=True)
for target in ['x86_64-pc-windows-gnu','x86_64-pc-windows-msvc']:
 if target in installed:run(target,['cargo','check','--locked','--target',target],required=False)
 else:checks.append({'name':target,'status':'not-run','reason':'Target not installed; no tool installation attempted.'})
# A brand new Cargo home verifies HTTPS immutable SDK dependency with no sibling access.
fresh=Q/'cargo-home-fresh'
if fresh.exists():raise SystemExit('Fresh-build home already exists. Preserve evidence; choose a new .qa path in the script for another independent run.')
fresh.mkdir()
run('fresh-https-release',['cargo','build','--locked','--release'],{'CARGO_HOME':str(fresh),'CARGO_TARGET_DIR':str(Q/'target-fresh')})
run('metadata',['cargo','metadata','--locked','--format-version','1'])
(Q/'cargo-metadata.json').write_bytes((Q/'metadata.log').read_bytes())
run('notices',['python3','scripts/notices.py'])
report={'source':PIN,'sdk':'e955911f0bf5b558f9d3eb5183df0adc95c22af2','host':'ea100cee3f69711de12485ba061950c5fc65258f','checks':checks,'tools':{t:subprocess.check_output(cmd,env=env,text=True).strip()for t,cmd in {'rustc':['rustc','--version'],'cargo':['cargo','--version'],'msrv':['rustc','+1.85.0','--version'],'node':['node','--version']}.items()},'notRun':['Real OS clipboard/browser/window','Live API or user public IP lookup','Full app build/GUI','Native Windows execution']}
(R/'evidence/validation.json').write_text(json.dumps(report,indent=2)+'\n')
print('Validation report written. Windows failures are recorded truthfully.')
