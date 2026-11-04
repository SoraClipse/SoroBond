import {test} from 'node:test';
import assert from 'node:assert/strict';
import {verifyTestnetManifest,inspectManifest} from './testnet-verifier.mjs';
const H=n=>n.toString(16).padStart(64,'0');
function fixture(){
 let n=0;
 const make=(id,phases)=>({warrantyId:id,transactions:Object.fromEntries(phases.map(p=>[p,{hash:H(++n)}]))});
 const m={network:'testnet',contractId:'C'+'A'.repeat(55),token:'C'+'B'.repeat(55),scenarios:{
  expiry:make(1,['created','activated','expired']),
  mutual:make(2,['created','activated','claimed','settled']),
  arbitration:make(3,['created','activated','claimed','arbitrated'])
 },negative:{unauthorized_settlement:{hash:H(++n)},double_withdraw:{hash:H(++n)}}};
 const receipts=new Map();
 for(const scenario of Object.values(m.scenarios))for(const [phase,tx] of Object.entries(scenario.transactions)){
  const ledger=Number.parseInt(tx.hash,16);
  receipts.set(tx.hash,{contractId:m.contractId,transactionHash:tx.hash,warrantyId:scenario.warrantyId,phase,ledger,token:m.token,bondAmount:'100',buyerPayout:phase==='expired'?'0':'25',sellerReturn:phase==='expired'?'100':'75'});
 }
 const rpc={network:'testnet',checkNetwork:async()=>({passphrase:'Test SDF Network ; September 2015'}),status:async hash=>receipts.has(hash)?{status:'CONFIRMED',ledger:receipts.get(hash).ledger}:{status:'FAILED'}};
 const reader={network:'testnet',contractId:m.contractId,readReceipt:async hash=>receipts.get(hash)};
 return {m,rpc,reader,receipts};
}
test('fail closed for incorrect Stellar network',async()=>{
 const {m,rpc,reader}=fixture();
 rpc.checkNetwork=async()=>({passphrase:'Public Global Stellar Network ; September 2015'});
 await assert.rejects(verifyTestnetManifest({manifest:m,rpc,reader}),/wrong_network/);
});
test('refuse a reportedly successful unauthorized settlement',async()=>{
 const {m,rpc,reader}=fixture();
 const original=rpc.status;
 rpc.status=async hash=>hash===m.negative.unauthorized_settlement.hash?{status:'CONFIRMED',ledger:999}:original(hash);
 await assert.rejects(verifyTestnetManifest({manifest:m,rpc,reader}),/negative_not_rejected_unauthorized_settlement/);
});
test('refuse successful second withdrawal',async()=>{
 const {m,rpc,reader}=fixture();
 const original=rpc.status;
 rpc.status=async hash=>hash===m.negative.double_withdraw.hash?{status:'CONFIRMED',ledger:999}:original(hash);
 await assert.rejects(verifyTestnetManifest({manifest:m,rpc,reader}),/negative_not_rejected_double_withdraw/);
});
test('refuse altered settlement accounting and mismatched token',async()=>{
 const {m,rpc,reader,receipts}=fixture();
 const tx=m.scenarios.mutual.transactions.settled.hash;
 receipts.set(tx,{...receipts.get(tx),sellerReturn:'80'});
 await assert.rejects(verifyTestnetManifest({manifest:m,rpc,reader}),/invalid_settlement_accounting/);
 receipts.set(tx,{...receipts.get(tx),sellerReturn:'75',token:'CDIFFERENT'});
 await assert.rejects(verifyTestnetManifest({manifest:m,rpc,reader}),/wrong_asset_receipt/);
});
test('refuse duplicate transaction used in two scenarios',()=>{
 const {m}=fixture();
 m.scenarios.mutual.transactions.created.hash=m.scenarios.expiry.transactions.created.hash;
 assert.throws(()=>inspectManifest(m),/reused_transaction/);
});
