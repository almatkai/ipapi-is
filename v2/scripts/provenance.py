from pathlib import Path
import hashlib,json,subprocess
R=Path(__file__).resolve().parents[1];OWN=R.parent
SDK=OWN.parent/'tezbar-extension-sdk-rust';SPIN='e955911f0bf5b558f9d3eb5183df0adc95c22af2';PIN='401650f61ca3eb1b9a77bd6764007e0f2075387f'
H=lambda b:hashlib.sha256(b).hexdigest()
original={}
paths=subprocess.check_output(['git','ls-tree','-r','--name-only',PIN],cwd=OWN,text=True).splitlines()
assert 'package-lock.json'in paths
for p in paths:
 b=subprocess.check_output(['git','show',PIN+':'+p],cwd=OWN);original[p]=H(b)
 if p=='README.md':assert (OWN/p).read_bytes()==b+b'\n\nStandalone Rust V2 (development/review): [v2/README.md](v2/README.md).\n'
 else:assert (OWN/p).read_bytes()==b,p
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=OWN,text=True).strip()==PIN
assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=OWN)
assert subprocess.check_output(['git','diff','--name-only'],cwd=OWN,text=True).strip()=='README.md'
assert H((R/'resources/ipapi-is.png').read_bytes())==original['assets/ipapi-is.png']=='1136f4bb335e07d7cd30d63033c25302524ff2bcf6e77e4875140ca74e0fdafa'
for p in ['SDK_API.md','PROTOCOL.md','HOST_INTEGRATION.md']:
 b=subprocess.check_output(['git','show',SPIN+':'+p],cwd=SDK);(R/'evidence/sdk'/p).write_bytes(b)
result={'source':{'repository':'https://github.com/almatkai/ipapi-is','commit':PIN,'branch':'feat/native-v2-rust','originalFilesSha256':original},'upstream':{'repository':'https://github.com/raycast/extensions','commit':'6734d85992290066fd31a7f5bee3cca3d3545a1a','path':'extensions/ipapi-is','icon':'assets/ipapi-is.png','iconSha256':original['assets/ipapi-is.png'],'iconFetch':'https://raw.githubusercontent.com/raycast/extensions/6734d85992290066fd31a7f5bee3cca3d3545a1a/extensions/ipapi-is/assets/ipapi-is.png','ownOriginalMatches':True},'sdk':{'version':'0.1.1','repository':'https://github.com/almatkai/tezbar-extension-sdk-rust','commit':SPIN,'docsSha256':{p:H((R/'evidence/sdk'/p).read_bytes())for p in ['SDK_API.md','PROTOCOL.md','HOST_INTEGRATION.md']}},'host':{'commit':'ea100cee3f69711de12485ba061950c5fc65258f','fixtureAccess':'exact git show; no app working-file read/edit/build'},'v1LockSha256':original['package-lock.json'],'checks':{'allOtherV1Bytes':'exact','README':'only discovery suffix','HEAD':'unchanged','staging':'empty'}}
(R/'PROVENANCE.json').write_text(json.dumps(result,indent=2)+'\n')
print('Verified all V1 bytes except exact README discovery suffix; unchanged HEAD, empty staging, exact original icon.')
