"""Hash all Git-visible source/evidence files, excluding only this inventory itself."""
from pathlib import Path
import hashlib,subprocess
R=Path(__file__).resolve().parents[1];OWN=R.parent
paths=subprocess.check_output(['git','ls-files','-c','-o','--exclude-standard','-z'],cwd=OWN).decode().split('\0')
lines=[]
for name in sorted(set(paths)-{'','v2/INDEX.sha256'}):
 p=OWN/name
 if not p.is_file()or p.is_symlink():raise SystemExit('Non-regular Git-visible path: '+name)
 lines.append(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+name)
(R/'INDEX.sha256').write_text('\n'.join(lines)+'\n');print(f'Indexed {len(lines)} Git-visible files; self excluded.')
