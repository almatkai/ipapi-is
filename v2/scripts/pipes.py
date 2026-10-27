"""Real JSONL OS pipes; injected loopback TLS client only in the QA example.
Packaged executable has no certificate/proxy/endpoint override.
"""
import json,os,queue,selectors,socketserver,ssl,subprocess,threading,time,uuid
from pathlib import Path
from urllib.parse import urlsplit,parse_qs
R=Path(__file__).resolve().parents[1];Q=R/'.qa';Q.mkdir(exist_ok=True)
cert=Q/'tls.pem';key=Q/'tls-key.pem'
subprocess.run(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-keyout',str(key),'-out',str(cert),'-subj','/CN=api.ipapi.is','-addext','subjectAltName=DNS:api.ipapi.is'],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
TLS=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);TLS.load_cert_chain(cert,key)
cases=json.loads((R/'fixtures/cases.json').read_text());oracle=json.loads((R/'fixtures/oracle.json').read_text());requests=[];seen=queue.Queue();modes={};stop=threading.Event()
class Handler(socketserver.BaseRequestHandler):
 def handle(self):
  try:
   self.request.settimeout(4);raw=b''
   while b'\r\n\r\n' not in raw:raw+=self.request.recv(1)
   assert raw==b'CONNECT api.ipapi.is:443 HTTP/1.1\r\nHost: api.ipapi.is:443\r\n\r\n',raw
   self.request.sendall(b'HTTP/1.1 200 Connection established\r\n\r\n')
   with TLS.wrap_socket(self.request,server_side=True) as conn:
    raw=b''
    while b'\r\n\r\n' not in raw:raw+=conn.recv(4096)
    q=parse_qs(urlsplit(raw.split(b'\r\n')[0].decode().split(' ')[1]).query,keep_blank_values=True)['q'][0]
    assert b'authorization:'not in raw.lower();requests.append(q);seen.put(q);mode=modes.get(q,'ok')
    if mode=='hold':stop.wait(.15)
    status=200;body=json.dumps(next((c['data']for c in cases if c['input']==q),{'ip':'192.0.2.99','elapsed_ms':0})).encode();extra=b''
    if mode in ['429','401']:status=int(mode);body=b'synthetic private error body'
    elif mode=='malformed':body=b'{broken'
    elif mode=='partial':conn.sendall(b'HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2000\r\n\r\n{}');return
    elif mode=='oversize':conn.sendall(b'HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 1048577\r\n\r\n');return
    elif mode=='redirect':status=302;extra=b'Location: https://evil.invalid/?key=synthetic\r\n'
    elif mode=='reflected':body=b'{"elapsed_ms":0,"error":"https://evil.invalid/?key=synthetic"}'
    conn.sendall(b'HTTP/1.1 '+str(status).encode()+b' Synthetic\r\nContent-Type: application/json\r\nContent-Length: '+str(len(body)).encode()+b'\r\nConnection: close\r\n'+extra+b'\r\n'+body)
  except (OSError,ssl.SSLError):pass
class Server(socketserver.ThreadingTCPServer):allow_reuse_address=True;daemon_threads=False;block_on_close=True
server=Server(('127.0.0.1',0),Handler);thread=threading.Thread(target=server.serve_forever);thread.start()
class Pipe:
 def __init__(self,cmd):
  self.err=open(Q/'pipe-stderr.log','w');self.p=subprocess.Popen(cmd,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=self.err,env={**os.environ,'HOME':str(Q/'home'),'TMPDIR':str(Q/'tmp')},bufsize=0);self.buf=b'';self.responses={};self.reverse=[];self.reply=json.loads((R/'evidence/host/host-replies.json').read_text());self.manual=False
  assert self.read()['type']=='hello';self.call('initialize',{'protocol':1,'extensionId':'com.tezbar.native.ipapi-is','platform':'macOS','packageDir':str(R),'dataDir':str(Q/'profile-a'),'tempDir':str(Q/'tmp'),'grantedCapabilities':['clipboard.write','shell.open'],'migrationPaths':{}})
 def send(self,v):self.p.stdin.write((json.dumps(v)+'\n').encode());self.p.stdin.flush()
 def request(self,m,p):
  rid='h:'+str(uuid.uuid4());self.send({'type':'request','id':rid,'method':m,'params':p});return rid
 def read(self,timeout=8):
  end=time.monotonic()+timeout
  while b'\n'not in self.buf:
   with selectors.DefaultSelector() as sel:
    sel.register(self.p.stdout,selectors.EVENT_READ);assert sel.select(max(0,end-time.monotonic())),'pipe timeout'
   b=os.read(self.p.stdout.fileno(),65536);assert b,'unexpected EOF';self.buf+=b
  line,self.buf=self.buf.split(b'\n',1);return json.loads(line)
 def wait(self,rid):
  while rid not in self.responses:
   m=self.read()
   if m['type']=='request':
    assert m['method']in self.reply,'unmocked host producer';self.reverse.append(m)
    if self.manual:return m
    self.send({'type':'response','id':m['id'],'result':self.reply[m['method']]})
   elif m['type']=='response':self.responses[m['id']]=m
   else:assert m['type']in ['cancel','event']
  return self.responses.pop(rid)
 def call(self,m,p):return self.wait(self.request(m,p))
 def open(self,s,q,prefs=None):return self.call('session.open',{'sessionId':s,'commandName':'lookup-ip-or-asn','argumentValues':{'ip':q},'preferences':prefs or {}})
 def action(self,s,a):return self.call('session.call',{'sessionId':s,'operation':'extension:invoke-action','payload':{'actionId':a}})
 def refresh(self,s):return self.call('session.call',{'sessionId':s,'operation':'extension:refresh-session','payload':{}})
 def dispose(self,s):return self.call('session.dispose',{'sessionId':s})
 def close(self):self.p.stdin.close();assert self.p.wait(timeout=8)==0;self.err.close()
def result(r):assert 'result'in r,r;return r['result']
def view(r):v=result(r);assert v['mode']=='view'and v['refreshMs']is None;return v
def action(v,t):return next(a['id']for a in v['actions']if a['title']==t)
def drain(q):
 while seen.get(timeout=4)!=q:pass
summary=[];p=None
try:
 p=Pipe([str(R/'target/debug/examples/pipe-fixture'),str(cert),'http://127.0.0.1:'+str(server.server_address[1])])
 for i,c in enumerate(oracle):
  s='fixture-'+str(i);v=view(p.open(s,c['input']));assert v['root']['metadata']['children']==c['metadata'],c['name'];copy=action(v,'Copy All as JSON');v=view(p.action(s,copy));assert p.reverse[-1]['params']=={'text':c['copy']};assert 'error'in p.action(s,copy)
  n=len(requests);assert result(p.refresh(s))['mode']=='unchanged';assert len(requests)==n;v=view(p.action(s,action(v,'Revalidate IP or ASN')));assert len(requests)==n+1;p.dispose(s)
 summary.append('10 original oracle fixtures via TLS+JSONL; copy/revalidate/stale IDs/heartbeat')
 s='strict';v=view(p.open(s,'192.0.2.1'))
 for bad in [None,True,False,{}, {'ok':False},{'ok':True,'error':'contradiction'},{'ok':True,'message':7},{'ok':True,'extra':0}]:
  p.reply['host.clipboard.write']=bad;v=view(p.action(s,action(v,'Copy All as JSON')));first=p.reverse[-1]['params'];assert len(v['actions'])==1;p.reply['host.clipboard.write']={'ok':True};v=view(p.action(s,action(v,'Retry Exact Copy')));assert p.reverse[-1]['params']==first
 summary.append('Closed clipboard ACK negatives including null; exact-byte retry')
 v=view(p.action(s,next(a['id']for a in v['actions']if a['kind']=='open')));assert p.reverse[-1]['method']=='host.shell.open'and p.reverse[-1]['params']=={'target':'https://example.invalid/whois'}
 # Shell has its own exact contract and retry, not the clipboard mock default.
 p.reply['host.shell.open']=None;v=view(p.action(s,next(a['id']for a in v['actions']if a['kind']=='open')));target=p.reverse[-1]['params'];assert v['actions'][0]['title']=='Retry Exact Open';p.reply['host.shell.open']={'ok':True,'message':'Opened'};v=view(p.action(s,action(v,'Retry Exact Open')));assert p.reverse[-1]['params']==target
 p.manual=True;rid=p.request('session.call',{'sessionId':s,'operation':'extension:invoke-action','payload':{'actionId':action(v,'Copy All as JSON')}});rev=p.wait(rid);assert rev['type']=='request';before=rev['params'];p.send({'type':'cancel','requestId':rid,'reason':'fixture cancel'});assert 'error'in p.wait(rid);p.manual=False;v=view(p.refresh(s));v=view(p.action(s,action(v,'Retry Exact Copy')));assert p.reverse[-1]['params']==before;p.dispose(s)
 summary.append('Shell request; cancelled write retains uncertainty; refresh never replays')
 for mode in ['429','401','malformed','partial','oversize','redirect','reflected']:
  q='synthetic-'+mode;modes[q]=mode;s='error-'+mode;v=view(p.open(s,q));raw=json.dumps(v);assert 'evil.invalid'not in raw and 'private error body'not in raw;title='Revalidate IP or ASN'if mode=='reflected'else'Retry IP or ASN';modes[q]='ok';v=view(p.action(s,action(v,title)));assert v['actions'][0]['title']=='Copy All as JSON';p.dispose(s)
 summary.append('401/429/malformed/partial/oversize/redirect/reflected errors and completed failure Retry')
 s='cache';q='192.0.2.1';v=view(p.open(s,q));modes[q]='429';v=view(p.action(s,action(v,'Revalidate IP or ASN')));assert 'cached response'in v['root']['props']['markdown'];p.dispose(s);v=view(p.open('identity',q,{'unusedIdentity':'different'}));assert all(a['title']!='Copy All as JSON'for a in v['actions']);p.dispose('identity');modes[q]='ok'
 summary.append('Cache hit on failure and preference identity separation (profile/expiry in unit tests)')
 s='search';v=view(p.open(s,'AS64512'));old=action(v,'Copy All as JSON');n=len(requests)
 for q in ['A','B','A']:assert result(p.call('session.call',{'sessionId':s,'operation':'extension:search-text-changed','payload':{'searchText':q}}))['mode']=='unchanged'
 assert len(requests)==n;view(p.action(s,old));p.dispose(s)
 summary.append('Source has no search or Form; A/B/A searches are no-op without extra HTTP')
 modes['hold-A']='hold';rid=p.request('session.open',{'sessionId':'held-A','commandName':'lookup-ip-or-asn','argumentValues':{'ip':'hold-A'},'preferences':{}});drain('hold-A');view(p.open('latest-B','192.0.2.1'));p.send({'type':'cancel','requestId':rid,'reason':'new query'});assert 'error'in p.wait(rid);p.dispose('latest-B');view(p.open('new-A','hold-A'));p.dispose('new-A')
 summary.append('Overlapping A/B/A opens; canceled old request cannot authorize newer session')
 modes['hold-cancel']='hold'
 for i in range(140):
  rid=p.request('session.open',{'sessionId':'cancel-'+str(i),'commandName':'lookup-ip-or-asn','argumentValues':{'ip':'hold-cancel'},'preferences':{}});drain('hold-cancel');p.send({'type':'cancel','requestId':rid,'reason':'canceled open'});assert 'error'in p.wait(rid)
 view(p.open('after-140','192.0.2.1'));p.dispose('after-140');summary.append('140 canceled opens without dispose: SDK-owned guard releases authority')
 for i in range(128):view(p.open('capacity-'+str(i),'bad\ninput'))
 assert 'error'in p.open('over-capacity','bad\ninput')
 for i in range(128):p.dispose('capacity-'+str(i))
 p.close();p=None;summary.append('128-session capacity, explicit failure at 129, disposal/EOF')
 p=Pipe([str(R/'target/debug/tezbar-ipapi-is')]);v=view(p.open('production','bad\ninput'));assert v['actions'][0]['title']=='Retry IP or ASN';assert 'error'in p.action('production','forged');p.dispose('production');p.close();p=None
 summary.append('Actual production executable initialize/recoverable invalid-input view/action failure/dispose/EOF, no network')
 (R/'evidence/pipes-results.json').write_text(json.dumps({'passed':summary,'network':'Loopback synthetic TLS producer. QA-only injected client; no API/OS call.','notRun':['Live API','real clipboard/browser/window','public response through production HTTPS client']},indent=2)+'\n');print('\n'.join(summary))
finally:
 if p is not None:p.p.kill();p.p.wait();p.err.close()
 stop.set();server.shutdown();server.server_close();thread.join()
