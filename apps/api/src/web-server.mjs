import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import {resolve} from 'node:path';
const port=Number(process.env.PORT||8788);
const files=new Map([
 ['/warranties/new',['../../web/public/warranties-new.html','text/html; charset=utf-8']],
 ['/warranties/fund',['../../web/public/warranty-dashboard.html','text/html; charset=utf-8']],
 ['/warranties/claims',['../../web/public/warranty-dashboard.html','text/html; charset=utf-8']],
 ['/agreement.mjs',['../../web/src/agreement.mjs','text/javascript; charset=utf-8']],
 ['/warranty-dashboard.mjs',['../../web/src/warranty-dashboard.mjs','text/javascript; charset=utf-8']],
 ['/warranty-actions.mjs',['../../web/src/warranty-actions.mjs','text/javascript; charset=utf-8']],
 ['/funding.mjs',['../../web/src/funding.mjs','text/javascript; charset=utf-8']],
 ['/claims.mjs',['../../web/src/claims.mjs','text/javascript; charset=utf-8']],
]);
const policy="default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; frame-ancestors 'none'; base-uri 'none'; object-src 'none'";
const handler=async(req,res)=>{
 const path=req.url?.split('?')[0],file=files.get(path);
 if(req.method!=='GET'||!file){res.writeHead(404);res.end();return;}
 try{
  const bytes=await readFile(new URL(file[0],import.meta.url));
  res.writeHead(200,{'content-type':file[1],'cache-control':'no-store','x-content-type-options':'nosniff','content-security-policy':policy});res.end(bytes);
 }catch{res.writeHead(500);res.end('Unavailable');}
};
export function startServer(p=port){return createServer(handler).listen(p,'127.0.0.1');}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url))startServer();
