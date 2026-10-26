// Original pinned TSX + actual locked @raycast/utils bundle in a closed VM.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict'),crypto=require('node:crypto');
const root=path.resolve(__dirname,'..'), v1=path.join(root,'.qa/v1');
const ts=require(path.join(v1,'node_modules/typescript'));
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/cases.json'),'utf8'));
const deny=new Proxy(function(){throw Error('Unmocked producer');},{get(_t,k){if(k==='__esModule')return false;throw Error('Unmocked producer '+String(k));},apply(){throw Error('Unmocked producer');}});
let slots=[],cursor=0,effects=[],dirty=false,cleanups=[];
function depsEq(a,b){return a&&b&&a.length===b.length&&a.every((v,i)=>Object.is(v,b[i]));}
const React={Fragment:'Fragment',createElement:(type,props,...children)=>({type,props:{...props,children:children.length===1?children[0]:children}}),
 useRef(x){let i=cursor++;return slots[i]??(slots[i]={current:x});},
 useState(x){let i=cursor++;if(!slots[i])slots[i]={value:typeof x==='function'?x():x};if(!slots[i].set)slots[i].set=v=>{slots[i].value=typeof v==='function'?v(slots[i].value):v;dirty=true;};return[slots[i].value,slots[i].set];},
 useMemo(fn,deps){let i=cursor++;if(!slots[i]||!depsEq(slots[i].deps,deps))slots[i]={value:fn(),deps};return slots[i].value;},
 useCallback(fn,deps){return this.useMemo(()=>fn,deps);},
 useEffect(fn,deps){let i=cursor++;if(!slots[i]||!depsEq(slots[i].deps,deps)){effects.push(()=>{slots[i]?.cleanup?.();const cleanup=fn();slots[i]={deps,cleanup};});}},
 useSyncExternalStore(sub,get){cursor++;return get();}};
React.useCallback=React.useCallback.bind(React);
const caches=new Map(),toasts=[],calls=[];
class Cache {constructor({namespace}={}){this.n=namespace;this.subscribe=()=>()=>{};}get=(k)=>caches.get(this.n+':'+k);set=(k,v)=>{caches.set(this.n+':'+k,v);dirty=true;};}
const comp=n=>n;
const Detail=comp('Detail'); // strings need attached metadata: use function components as tokens
function D(){}D.Metadata=function(){};D.Metadata.Label='Detail.Metadata.Label';D.Metadata.Link='Detail.Metadata.Link';D.Metadata.Separator='Detail.Metadata.Separator';D.Metadata.TagList=function(){};D.Metadata.TagList.Item='Detail.Metadata.TagList.Item';
function A(){}A.CopyToClipboard='Action.CopyToClipboard';
const api={Detail:D,Action:A,ActionPanel:'ActionPanel',Icon:{Redo:'Redo',Check:'Check',Xmark:'Xmark',Map:'Map'},Cache,environment:{assetsPath:'/fixture/assets',isDevelopment:true,launchType:'user',commandMode:'view'},LaunchType:{Background:'background'},Toast:{Style:{Failure:'failure'}},showToast:async o=>{toasts.push(o);return{hide(){}};},Clipboard:{copy:async text=>calls.push({copy:text})},open:async target=>calls.push({open:target})};
let producer;let now=1700000000000;class Clock extends Date{constructor(...a){super(...(a.length?a:[now]));}static now(){return now;}}
const sandbox={console:{error:()=>{},warn:()=>{},log:()=>{}},AbortController,Response,Request,URL,URLSearchParams,Date:Clock,TextEncoder,TextDecoder,Buffer:{isBuffer:Buffer.isBuffer},setTimeout:deny,process:undefined,fetch:async (u,o)=>{calls.push({fetch:String(u)});assert.ok(o.signal);const r=await producer(String(u));return{ok:r.ok,statusText:r.statusText,headers:r.headers,json:async()=>parseVM(await r.text()),text:()=>r.text()};}};
const context=vm.createContext(sandbox,{codeGeneration:{strings:false,wasm:false}});
const parseVM=new vm.Script("JSON.parse").runInContext(context);
const modules={'node:events':class DeniedEventEmitter{constructor(){throw Error('Unmocked EventEmitter construction');}},'react/jsx-runtime':{jsx:(t,p)=>({type:t,props:p}),jsxs:(t,p)=>({type:t,props:p}),Fragment:React.Fragment},react:React,'@raycast/api':api,'node:crypto':{createHash:crypto.createHash},'node:path':{join:path.posix.join},'node:fs':{readFileSync:(p,enc)=>{assert.equal(p,'/fixture/package.json');assert.equal(enc,'utf8');return fs.readFileSync(path.join(v1,'package.json'),'utf8');}}};
function evaluate(code,name,custom={}){const module={exports:{}};const req=n=>{if(n in custom)return custom[n];if(n in modules)return modules[n];if(n==='dequal/lite')return evaluate(fs.readFileSync(path.join(v1,'node_modules/dequal/lite/index.js'),'utf8'),'dequal');if(n.startsWith('node:'))return deny;throw Error('Denied module '+n);};context.__module=module;context.__require=req;new vm.Script('(function(module,exports,require){'+code+'\n})(__module,__module.exports,__require)',{filename:name}).runInContext(context,{timeout:2000});return module.exports;}
const utils=evaluate(fs.readFileSync(path.join(v1,'node_modules/@raycast/utils/dist/main.js'),'utf8'),'locked-utils');
const code=ts.transpileModule(fs.readFileSync(path.join(v1,'src/lookup-ip-or-asn.tsx'),'utf8'),{compilerOptions:{jsx:ts.JsxEmit.React,module:ts.ModuleKind.CommonJS,esModuleInterop:true}}).outputText;
// TS classic JSX emits React.createElement; injection supplies hooks/element tokens only.
context.React=React;
const Command=evaluate(code,'pinned-command',{'@raycast/utils':utils,'./types':{}}).default;
function expand(n){if(n===null||n===undefined||typeof n==='boolean')return null;if(Array.isArray(n))return n.flatMap(x=>{let e=expand(x);return Array.isArray(e)?e:e?[e]:[];});if(typeof n!=='object')return n;if(typeof n.type==='function'&&![D,D.Metadata,D.Metadata.TagList,A].includes(n.type))return expand(n.type(n.props));return {...n,props:{...n.props,children:expand(n.props.children),...(n.props.metadata?{metadata:expand(n.props.metadata)}:{}),...(n.props.actions?{actions:expand(n.props.actions)}:{})}};}
function all(n,type){if(!n)return[];if(Array.isArray(n))return n.flatMap(x=>all(x,type));if(typeof n!=='object')return[];return [...(n.type===type?[n]:[]),...all(n.props?.children,type)];}
async function render(input){let tree;for(let round=0;round<40;round++){dirty=false;cursor=0;effects=[];tree=expand(Command({arguments:{ip:input}}));for(const e of effects)e();for(let i=0;i<12;i++)await Promise.resolve();if(!dirty)return tree;}throw Error('Hook settle limit');}
function unmount(){for(const s of slots)s?.cleanup?.();slots=[];cursor=0;}
const result=[];
(async()=>{
 for(const c of cases){unmount();calls.length=0;toasts.length=0;const wire=JSON.stringify(c.data);producer=async()=>new Response(wire,{status:200,headers:{'content-type':'application/json'}});
  const tree=await render(c.input);assert.equal(tree.props.isLoading,false);assert.deepEqual(calls.filter(c=>c.fetch).map(c=>c.fetch),['https://api.ipapi.is?q='+c.input]);
  const acts=all(tree.props.actions,'Action.CopyToClipboard');assert.equal(acts.length,1);assert.equal(acts[0].props.content,JSON.stringify(JSON.parse(wire)));
  let metadata=all(tree.props.metadata,D.Metadata)[0].props.children;
  function nodes(ns){return(ns||[]).flatMap(n=>{if(!n||typeof n!=='object')return[];if(n.type===React.Fragment)return nodes(n.props.children);const {children,...p}=n.props;delete p.key;
   if(n.type===D.Metadata.TagList)return[{type:'Detail.Metadata.TagList',props:p,children:nodes(children)}];
   return[{type:n.type,props:p}];});}
  metadata=nodes(Array.isArray(metadata)?metadata:[metadata]);
  // Explicit finite adaptations: inert links, boolean text, renderer flag fallback, sanitized errors.
  for(const n of metadata){if(n.type==='Detail.Metadata.Link'){n.type='Detail.Metadata.Label';delete n.props.target;}
   if(n.props.icon==='Check'||n.props.icon==='Xmark')n.props.text=n.props.icon==='Check'?'Yes':'No';
   if(n.props.icon?.source?.startsWith('https://flagsapi.com/'))n.props.icon='Map';
   if(n.props.title==='Error')n.props.text='The service reported an error. Revalidate to retry.';}
  let copy=JSON.parse(wire);if('error'in copy)copy.error='The service reported an error. Revalidate to retry.';
  const heading=tree.props.markdown.split('Error:')[0].replace(c.input, [...c.input].map(c=>/[a-zA-Z0-9 .:_-]/.test(c)?c:'&#'+c.codePointAt(0)+';').join(''));
  result.push({name:c.name,input:c.input,wire,metadata,copy:JSON.stringify(copy),heading});
  // Execute original revalidate callback, not a self-reported action marker.
  const redo=all(tree.props.actions,A)[0];assert.equal(redo.props.title,'Revalidate IP or ASN');await redo.props.onAction();await render(c.input);assert.equal(calls.filter(c=>c.fetch).length,2);
  if('error'in c.data)assert.ok(toasts.length>=1);
 }
 // Locked hook HTTP failure -> Retry toast -> actual re-fetch, with synthetic producer only.
 unmount();calls.length=0;toasts.length=0;producer=async()=>new Response('never display raw body',{status:429,statusText:'Synthetic rate limit'});await render('bad');assert.equal(toasts[0].title,'Failed to fetch latest data');assert.equal(toasts[0].primaryAction.title,'Retry');producer=async()=>new Response('{"ip":"192.0.2.9","elapsed_ms":0}',{headers:{'content-type':'application/json'}});toasts[0].primaryAction.onAction({hide(){}});await render('bad');assert.equal(calls.filter(c=>c.fetch).length,2);
 fs.writeFileSync(path.join(root,'fixtures/oracle.json'),JSON.stringify(result,null,2)+'\n');
 console.log('Original TSX + locked useFetch/useCachedPromise/usePromise: '+result.length+' finite fixtures, copy, metadata, actions and 429 retry passed.');
})().catch(e=>{console.error(e);process.exitCode=1;});
