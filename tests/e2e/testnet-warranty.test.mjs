import {test} from 'node:test';
import {readFile} from 'node:fs/promises';
import {pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
import {verifyTestnetManifest} from './testnet-verifier.mjs';
// Opt in with real transaction IDs and a reviewed on-chain receipt adapter.
// No manifest means SKIP, not live-Testnet success.
const manifestPath=process.env.SOROBOND_TESTNET_MANIFEST;
const adapterPath=process.env.SOROBOND_TESTNET_ADAPTER;
test('verify real Stellar Testnet expiry, mutual settlement, arbitration and rejected transactions',{
 skip:!(manifestPath&&adapterPath)&&'requires actual Testnet transaction manifest and receipt adapter',
},async()=>{
 const manifest=JSON.parse(await readFile(resolve(manifestPath),'utf8'));
 const {rpc,reader}=await import(pathToFileURL(resolve(adapterPath)).href).then(m=>m.createTestnetReaders(manifest));
 const verified=await verifyTestnetManifest({manifest,rpc,reader});
 if(verified.transactions.length!==13)throw Error('missing_evidence');
});
