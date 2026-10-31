import {test} from 'node:test';
import assert from 'node:assert/strict';
import {inspectManifest,verifyTestnetManifest} from './testnet-verifier.mjs';
const H=n=>n.toString(16).padStart(64,'0');let serial=1;
const makeScenario=(id,phases)=>({warrantyId:id,transactions:Object.fromEntries(phases.map(p=>[p,{hash:H(serial++)}]))});
const fixture=()=>{serial=1;return {
 network:'testnet',contractId:'C'+'A'.repeat(55),token:'C'+'B'.repeat(55),
 scenarios:{
  expiry:makeScenario(1,['created','activated','expired']),
  mutual:makeScenario(2,['created','activated','claimed','settled']),
  arbitration:makeScenario(3,['created','activated','claimed','arbitrated'])
 },
 negative:{unauthorized_settlement:{hash:H(serial++)},double_withdraw:{hash:H(serial++)}}
};};
test('reject missing transaction hashes or reused evidence',()=>{
 const m=fixture();
 assert.equal(inspectManifest(m).warranties,3);assert.equal(inspectManifest(m).transactions,13);
 m.scenarios.expiry.transactions.created.hash='not-a-hash';
 assert.throws(()=>inspectManifest(m),/missing_transaction/);
 const x=fixture();x.negative.double_withdraw.hash=x.negative.unauthorized_settlement.hash;
 assert.throws(()=>inspectManifest(x),/reused_transaction/);
});
test('mock receipt unit tests are not live Testnet deployment proof',async()=>{
 const m=fixture(),all={};let i=0;
 for(const [name,sc] of Object.entries(m.scenarios))for(const [phase,p] of Object.entries(sc.transactions)){
  const ledger=++i;all[p.hash]={ledger,phase,id:sc.warrantyId,name};
 }
 const rpc={network:'testnet',checkNetwork:async()=>({passphrase:'Test SDF Network ; September 2015'}),status:async hash=>all[hash]?{status:'CONFIRMED',ledger:all[hash].ledger}:{status:'FAILED'}};
 const reader={network:'testnet',contractId:m.contractId,readReceipt:async hash=>{
  const v=all[hash];
  return {contractId:m.contractId,transactionHash:hash,phase:v.phase,warrantyId:v.id,ledger:v.ledger,token:m.token,bondAmount:'100',buyerPayout:v.phase==='expired'?'0':'25',sellerReturn:v.phase==='expired'?'100':'75'};
 }};
 assert.equal((await verifyTestnetManifest({manifest:m,rpc,reader})).transactions.length,13);
 reader.readReceipt=async()=>({contractId:'CWRONG'});
 await assert.rejects(verifyTestnetManifest({manifest:m,rpc,reader}),/invalid_event_receipt/);
});
