import {test} from 'node:test';
import assert from 'node:assert/strict';
import {claimCapabilities,submitClaim,resolveClaim,withdrawExpired,uploadPrivateEvidence} from '../src/claims.mjs';
const HASH='a'.repeat(64),TX='b'.repeat(64);
function fixture(){
 const network='testnet',contractId='CDUMMYTESTNETCONTRACTIDENTITY';let now=50;
 const warranty={id:1,buyer:'GBuyer',seller:'GSeller',arbitrator:'GResolver',status:'Active',bond_amount:'250',expires_at:100};let claim=null,verdict=null;
 const rpc={network,prepare:async x=>({network,envelope:x}),submitSigned:async()=>({status:'PENDING',hash:TX}),status:async()=>({status:'CONFIRMED',ledger:999})};
 const chain={network,contractId,readWarranty:async()=>warranty,readClaim:async()=>claim,ledgerTimestamp:async()=>now,readVerdict:async()=>verdict,buildInvocation:async()=> 'unsigned-XDR'};
 const wallet={connect:async()=>({network,address:'GBuyer'}),signTransaction:async()=> 'signed-XDR',signJointTransaction:async()=> 'cosigned-XDR'};
 const p={rpc,chain,wallet,network,contractId,id:1};
 return {p,warranty,wallet,setNow:v=>now=v,setClaim:v=>claim=v,setVerdict:v=>verdict=v};
}
test('role-gated claim at inclusive deadline and immutable verdict',()=>{
 const w={id:1,status:'Active',buyer:'GBuyer',seller:'GSeller',arbitrator:'GResolver',expires_at:100};
 const can=(actor,ledgerTime,claim)=>claimCapabilities({warranty:w,claim,actor,ledgerTime});
 assert.equal(can('GBuyer',100,null).canClaim,true);
 assert.equal(can('GSeller',100,null).canClaim,false);
 assert.equal(can('GBuyer',101,null).canClaim,false);
 w.status='Claimed';
 assert.equal(can('GSeller',200000,{amount:'5'}).canWithdraw,false);
 assert.equal(can('GResolver',120,{amount:'5'}).canArbitrate,true);
 w.status='Settled';assert.equal(can('GResolver',120,{amount:'5'}).verdictImmutable,true);
});
test('claim rejects missing private evidence, unauthorized buyer, late and excessive amounts',async()=>{
 const f=fixture(),p=f.p;
 await assert.rejects(submitClaim({...p,buyer:'GBuyer',claimAmount:'40',evidenceHash:HASH}),/private_evidence_not_stored/);
 await assert.rejects(submitClaim({...p,buyer:'GSeller',claimAmount:'40',evidenceHash:HASH,evidenceReceipt:'sealed-1234567'}),/claim_not_allowed/);
 await assert.rejects(submitClaim({...p,buyer:'GBuyer',claimAmount:'251',evidenceHash:HASH,evidenceReceipt:'sealed-1234567'}),/claim_exceeds_bond/);
 f.setNow(101);
 await assert.rejects(submitClaim({...p,buyer:'GBuyer',claimAmount:'40',evidenceHash:HASH,evidenceReceipt:'sealed-1234567'}),/claim_not_allowed/);
});
test('successful RPC without a matching on-chain claim does not confirm',async()=>{
 const f=fixture();
 await assert.rejects(submitClaim({...f.p,buyer:'GBuyer',claimAmount:'40',evidenceHash:HASH,evidenceReceipt:'sealed-1234567'}),/onchain_claim_missing/);
});
test('arbitration is resolver-only; settlement requires both signatures and an immutable verdict',async()=>{
 const f=fixture();f.warranty.status='Claimed';f.setClaim({amount:'100'});
 await assert.rejects(resolveClaim({...f.p,actor:'GBuyer',method:'arbitrate',payout:'20'}),/resolution_not_allowed/);
 await assert.rejects(resolveClaim({...f.p,actor:'GResolver',method:'arbitrate',payout:'101'}),/payout_exceeds_claim/);
 f.wallet.connect=async()=>({network:'testnet',address:'GResolver'});
 f.setVerdict({buyer_payout:'20'});
 f.wallet.signTransaction=async()=>{f.warranty.status='Settled';return 'signed-XDR';};
 const r=await resolveClaim({...f.p,actor:'GResolver',method:'arbitrate',payout:'20'});
 assert.equal(r.state.verdict.buyer_payout,'20');
 const q=fixture();q.warranty.status='Claimed';q.setClaim({amount:'100'});
 delete q.wallet.signJointTransaction;
 await assert.rejects(resolveClaim({...q.p,actor:'GBuyer',method:'settle',payout:'20'}),/joint_authorization_required/);
});
test('seller withdrawal cannot bypass expiry plus grace or failed on-chain state',async()=>{
 const f=fixture();f.setNow(100+86400);
 await assert.rejects(withdrawExpired({...f.p,seller:'GSeller'}),/withdrawal_not_allowed/);
 f.setNow(100+86401);
 f.wallet.connect=async()=>({network:'testnet',address:'GSeller'});
 await assert.rejects(withdrawExpired({...f.p,seller:'GSeller'}),/onchain_expiry_missing/);
});

test('private evidence upload requires client-side encryption and a matching warranty-bound commitment',async()=>{
 const file={arrayBuffer:async()=>new TextEncoder().encode('private receipt').buffer};
 await assert.rejects(uploadPrivateEvidence({file,warrantyId:1,actor:'GBuyer'}),/secure_evidence_vault_unavailable/);
 const vault={clientSideEncryption:true,encryptAndStore:async()=>({receipt:'encrypted-receipt-1234',commitment:'bad'})};
 await assert.rejects(uploadPrivateEvidence({file,vault,warrantyId:1,actor:'GBuyer'}),/evidence_storage_unverified/);
 vault.encryptAndStore=async({bytes,warrantyId})=>{
  const prefix=new TextEncoder().encode(String(warrantyId)+':'),b=new Uint8Array(prefix.length+bytes.length);b.set(prefix);b.set(bytes,prefix.length);
  const hash=new Uint8Array(await crypto.subtle.digest('SHA-256',b));
  return {receipt:'encrypted-receipt-1234',commitment:Array.from(hash,x=>x.toString(16).padStart(2,'0')).join('')};
 };
 const x=await uploadPrivateEvidence({file,vault,warrantyId:1,actor:'GBuyer'});
 assert.match(x.commitment,/^[0-9a-f]{64}$/);assert.equal(x.receipt,'encrypted-receipt-1234');
});
