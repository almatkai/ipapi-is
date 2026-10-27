// Execute actual pinned DetailRuntime JSX callbacks before/after the UNAPPLIED wrapper.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const R=path.resolve(__dirname,'..'),ts=require(path.join(R,'.qa/v1/node_modules/typescript'));
const calls=[];const React={createElement:(type,props,...children)=>({type,props:{...props,children}})};
function execute(file){
 const src=fs.readFileSync(path.join(R,'evidence/host',file),'utf8');const callback=fs.readFileSync(path.join(R,'evidence/host/primary-callback.ts'),'utf8');
 const code=ts.transpileModule(src+'\n'+callback+'\nexports.callback=onRunPrimaryAction;',{compilerOptions:{module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.React}}).outputText;
 const exports={};const ctx=vm.createContext({exports,React,primaryAction:{id:'revision:copy'},onInvokeAction:(...args)=>calls.push(args),markdownFromNode:r=>r.props.markdown,parseImageOnlyMarkdown:()=>null,Markdown:'Markdown'});
 new vm.Script(code).runInContext(ctx,{timeout:1000});
 const tree=exports.DetailRuntime({root:{props:{markdown:'# IP: 192.0.2.1'}},title:'Lookup IP or ASN',onBack:()=>{},onRunPrimaryAction:exports.callback,onOpenActions:()=>{}});
 function find(n){if(!n||typeof n!=='object')return null;if(n.type==='button'&&n.props.children.includes('Enter'))return n;for(const c of n.props?.children||[]){const v=find(c);if(v)return v;}return null;}
 const event={type:'click',currentTarget:{}};find(tree).props.onClick(event);return event;
}
const event=execute('DetailRuntime-before.tsx');assert.equal(calls.pop()[0],event);
execute('DetailRuntime-after.tsx');assert.equal(calls.pop()[0],'revision:copy');
console.log('Actual pinned DetailRuntime + primary callback: reproduced MouseEvent-as-action-ID before, string action ID after wrapper.');
