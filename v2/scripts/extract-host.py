"""Read immutable Git objects only. Write fixtures/UNAPPLIED patches in this clone."""
from pathlib import Path
import subprocess,hashlib,json,difflib,re
R=Path(__file__).resolve().parents[1];APP=Path('/Users/almat/Desktop/code/Tezbar APP/Tezbar');PIN='ea100cee3f69711de12485ba061950c5fc65258f'
def read(p):return subprocess.check_output(['git','show',PIN+':'+p],cwd=APP).decode()
def function(s,name):
 start=re.search(r'fn '+name+r'(?:<[^>]+>)?\(',s).start();start=s.rfind('\n',0,start)+1;end=s.index('\n}',start)+2;return s[start:end]+'\n'
files={p:read(p)for p in ['src-tauri/src/native_core/extensions/registry.rs','src-tauri/src/native_core/extensions/process_install.rs','src-tauri/src/native_core/extensions/launcher.rs','src-tauri/src/native_core/extensions/process_host.rs','src-tauri/src/native_core/launcher/clipboard.rs','src-tauri/src/native_core/platform.rs','src/renderer/src/raycast-api/detail-runtime.tsx','src/renderer/src/raycast-api/index.tsx','schemas/extension-v3.schema.json']}
E=R/'evidence/host';E.mkdir(exist_ok=True);P=R/'host-prerequisites';P.mkdir(exist_ok=True)
(E/'provenance.json').write_text(json.dumps({'pin':PIN,'files':{p:hashlib.sha256(s.encode()).hexdigest()for p,s in files.items()}},indent=2)+'\n')
(E/'extension-v3.schema.json').write_text(files['schemas/extension-v3.schema.json'])
regpath='src-tauri/src/native_core/extensions/registry.rs';reg=files[regpath];patched=reg
patched=patched.replace('    path: PathBuf,\n    manifest:', '    path: PathBuf,\n    resource_dir: PathBuf,\n    manifest:',1)
patched=patched.replace('            let mut manifest = if native {','            let mut resource_dir = path.clone();\n            let mut manifest = if native {',1)
patched=patched.replace('Ok(package) => package.manifest,','Ok(package) => {\n                        resource_dir = package.dir;\n                        package.manifest\n                    },',1)
patched=patched.replace('                path,\n                manifest,','                path,\n                resource_dir,\n                manifest,',1)
patched=patched.replace('        path,\n        manifest: Value::Null,','        resource_dir: path.clone(),\n        path,\n        manifest: Value::Null,',1)
# Resource roots are distinct from canonical preferences/uninstall roots.
for name in ['icon_path','command_icon_path']:
 original=function(patched,name);replacement=original.replace('extension.path.join(relative)','extension.resource_dir.join(relative)').replace('extension.path.join("assets")','extension.resource_dir.join("assets")');patched=patched.replace(original,replacement)
old='''    } else {
        None
    };
    if let Some(extension) = native {'''
new='''    } else {
        // collect() accepts canonical manifests only after active package verification.
        collect(ctx).into_iter().find(|entry| {
            entry.id.starts_with("com.tezbar.native.")
                && entry.manifest["replaces"].as_array().is_some_and(|aliases| {
                    aliases.iter().any(|alias| alias.as_str() == Some(id))
                })
        })
    };
    if let Some(extension) = native {'''
assert old in patched;patched=patched.replace(old,new,1)
(P/'registry.patch').write_text(''.join(difflib.unified_diff(reg.splitlines(True),patched.splitlines(True),fromfile='a/'+regpath,tofile='b/'+regpath)))
for name in ['slug','fallback','installed_command','icon_path','command_icon_path']:
 (E/(name+'-before.rs')).write_text(function(reg,name));(E/(name+'-after.rs')).write_text(function(patched,name))
# Host validator exact source bodies, not a port-authored facsimile.
s=files['src-tauri/src/native_core/extensions/process_install.rs'];prefix=s[s.index('const CAPABILITY_GROUPS:'):s.index('// Shared by installer')]
(E/'validate-package.rs').write_text('use serde_json::Value;\nuse std::{path::PathBuf,collections::HashSet};\n'+prefix+'\n'+''.join(function(s,n)for n in ['component','relative','validate']))
s=files['src-tauri/src/native_core/extensions/launcher.rs'];(E/'validate-launcher.rs').write_text('use serde_json::Value;\nuse std::collections::HashSet;\nuse super::process_install;\n'+function(s,'bounded_text')+function(s,'validate'))
# Used host service producers only. No fabricated generic ACK defaults.
host=files['src-tauri/src/native_core/extensions/process_host.rs'];clipboard=files['src-tauri/src/native_core/launcher/clipboard.rs'];platform=files['src-tauri/src/native_core/platform.rs']
extract=host[host.index('            "host.clipboard.write"=>'):host.index('            "host.clipboard.writeImage"=>')]
extract+='\n'+host[host.index('            "host.shell.open"=>'):host.index('            "host.shell.openWithApp"=>')]
extract+='\n'+clipboard[clipboard.index('            "clipboard:write" => {'):clipboard.index('            "clipboard:write-png" => {')]
extract+='\n'+function(platform,'open_target')
(E/'host-method-producers.txt').write_text(extract)
assert 'return Ok(json!({"ok":true}));'in clipboard
assert 'Ok(json!({"ok":true,"message":"Opened"}))'in platform
(E/'host-replies.json').write_text(json.dumps({'host.clipboard.write':{'ok':True},'host.shell.open':{'ok':True,'message':'Opened'}},indent=2)+'\n')
detailpath='src/renderer/src/raycast-api/detail-runtime.tsx';detail=files[detailpath];patched=detail.replace('onClick={onRunPrimaryAction}', 'onClick={() => onRunPrimaryAction()}');assert patched!=detail
(P/'detail-primary.patch').write_text(''.join(difflib.unified_diff(detail.splitlines(True),patched.splitlines(True),fromfile='a/'+detailpath,tofile='b/'+detailpath)))
(E/'DetailRuntime-before.tsx').write_text(detail[detail.index('export function DetailRuntime('):]);(E/'DetailRuntime-after.tsx').write_text(patched[patched.index('export function DetailRuntime('):]);index=files['src/renderer/src/raycast-api/index.tsx'];start=index.index('  const onRunPrimaryAction =');end=index.index('\n\n',start);(E/'primary-callback.ts').write_text(index[start:end]+'\n')
# Keep exact producer excerpts proving shell ACK and routing, without unused visibility/paste patches.

print('Exact pinned extracts, schema, method defaults, and 2 UNAPPLIED used prerequisite patches generated.')
