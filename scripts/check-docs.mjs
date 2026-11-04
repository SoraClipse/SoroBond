import {readFileSync,existsSync} from 'node:fs';
import {resolve,dirname,relative,extname} from 'node:path';
const files=['README.md','CONTRIBUTING.md','SECURITY.md','docs/operations/deploy.md','docs/verification/testnet.md'];
const root=process.cwd();
const broken=[];
for(const file of files){
 const content=readFileSync(file,'utf8');
 for(const match of content.matchAll(/\[[^\]]+\]\(([^)]+)\)/g)){
  const target=match[1].split('#')[0].split('?')[0];
  if(!target || /^(?:https?:|mailto:)/.test(target))continue;
  const absolute=resolve(dirname(resolve(root,file)),decodeURIComponent(target));
  if(!absolute.startsWith(root+'/') && absolute!==root)broken.push(`${file}: link escapes repository: ${target}`);
  else if(!existsSync(absolute))broken.push(`${file}: missing ${target}`);
 }
}
if(broken.length)throw new Error(broken.join('\n'));
console.log('Local documentation links OK');
