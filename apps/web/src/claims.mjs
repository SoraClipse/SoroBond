import {amount,CLAIM_GRACE_SECONDS,invariant,submitAndVerify,validHash,verifyContext} from './warranty-actions.mjs';
export function claimCapabilities({warranty,claim,actor,ledgerTime}){
 const w=warranty;
 invariant(w&&Number.isSafeInteger(w.id)&&w.id>0&&['Pending','Active','Claimed','Settled','Expired'].includes(w.status),'invalid_warranty_state');
 invariant(Number.isSafeInteger(ledgerTime)&&ledgerTime>=0,'invalid_ledger_time');
 const buyer=actor===w.buyer,seller=actor===w.seller,resolver=actor===w.arbitrator;
 const active=w.status==='Active',disputed=w.status==='Claimed',terminal=['Settled','Expired'].includes(w.status);
 const expires=Number(w.expires_at);
 invariant(Number.isSafeInteger(expires)&&expires>=0,'invalid_deadline');
 return {isBuyer:buyer,isSeller:seller,isResolver:resolver,canClaim:buyer&&active&&ledgerTime<=expires&&!claim,canRespond:seller&&disputed,canOfferSettlement:(buyer||seller)&&disputed,canArbitrate:resolver&&disputed,canWithdraw:seller&&active&&!claim&&ledgerTime>expires+CLAIM_GRACE_SECONDS,terminal,deadline:expires,claimAmount:claim?.amount??null,evidenceHash:claim?.evidence_hash??null,verdictImmutable:terminal};
}
async function load({network,contractId,rpc,chain,id,actor}){
 verifyContext({network,contractId,rpc,chain});
 invariant(Number.isSafeInteger(id)&&id>0,'invalid_warranty_id');
 invariant(typeof chain.readWarranty==='function'&&typeof chain.readClaim==='function'&&typeof chain.ledgerTimestamp==='function','chain_reader_missing');
 const [w,claim,now]=await Promise.all([chain.readWarranty(id),chain.readClaim(id),chain.ledgerTimestamp()]);
 invariant(w&&w.id===id,'warranty_not_found');
 const capabilities=claimCapabilities({warranty:w,claim,actor,ledgerTime:now});
 return {w,claim,now,capabilities};
}
export async function claimDashboard(params){
 const current=await load(params);
 const verdict=['Settled','Expired'].includes(current.w.status)&&typeof params.chain.readVerdict==='function'?await params.chain.readVerdict(params.id):null;
 return {...current,verdict,privacy:'Private evidence must be encrypted off-chain. Only the SHA-256 commitment is published on-chain. The stored verdict cannot be changed.'};
}
export async function submitClaim(params){
 const {network,contractId,rpc,chain,wallet,id,buyer,claimAmount,evidenceHash}=params;
 const snapshot=await load({...params,actor:buyer});
 invariant(snapshot.capabilities.canClaim,'claim_not_allowed');
 const max=amount(snapshot.w.bond_amount),requested=amount(claimAmount);
 invariant(requested<=max,'claim_exceeds_bond');
 const evidence=validHash(evidenceHash);
 invariant(typeof params.evidenceReceipt==='string'&&params.evidenceReceipt.length>=12,'private_evidence_not_stored');
 return submitAndVerify({rpc,chain,wallet,network,contractId,actor:buyer,method:'submit_claim',args:[id,buyer,requested.toString(),evidence],maxPolls:params.maxPolls,wait:params.wait,
  after:async()=>{const [w,c]=await Promise.all([chain.readWarranty(id),chain.readClaim(id)]);invariant(w?.status==='Claimed'&&String(c?.amount)===requested.toString()&&c.evidence_hash?.toLowerCase()===evidence,'onchain_claim_missing');return {warranty:w,claim:c};}});
}
export async function resolveClaim(params){
 const {network,contractId,rpc,chain,wallet,id,actor,payout,method}=params;
 invariant(['settle','arbitrate'].includes(method),'invalid_resolution');
 const {w,claim,capabilities}=await load({...params,actor});
 invariant((capabilities.canArbitrate&&method==='arbitrate')||(capabilities.canOfferSettlement&&method==='settle'),'resolution_not_allowed');
 const requested=amount(payout,{positive:false});
 invariant(claim&&requested<=amount(claim.amount),'payout_exceeds_claim');
 if(method==='settle')invariant(typeof wallet?.signJointTransaction==='function','joint_authorization_required');
 // Seller and buyer must both authorize the same exact Soroban invocation.
 const coWallet=method==='settle'?{
  connect:()=>wallet.connect(),
  signTransaction:(prepared,details)=>wallet.signJointTransaction(prepared,{...details,requiredSigners:[w.buyer,w.seller]})
 }:wallet;
 return submitAndVerify({rpc,chain,wallet:coWallet,network,contractId,actor,method,args:[id,requested.toString()],maxPolls:params.maxPolls,wait:params.wait,
  after:async()=>{
   const updated=await chain.readWarranty(id);
   invariant(updated?.status==='Settled','onchain_verdict_missing');
   invariant(typeof chain.readVerdict==='function','verdict_reader_missing');
   const verdict=await chain.readVerdict(id);
   invariant(verdict?.buyer_payout!==undefined&&String(verdict.buyer_payout)===requested.toString(),'verdict_mismatch');
   return {warranty:updated,verdict};
  }});
}
export async function withdrawExpired(params){
 const {network,contractId,rpc,chain,wallet,id,seller}=params;
 const snapshot=await load({...params,actor:seller});
 invariant(snapshot.capabilities.canWithdraw,'withdrawal_not_allowed');
 return submitAndVerify({rpc,chain,wallet,network,contractId,actor:seller,method:'withdraw_expired',args:[id],maxPolls:params.maxPolls,wait:params.wait,after:async()=>{
  const w=await chain.readWarranty(id);invariant(w?.status==='Expired','onchain_expiry_missing');return w;
 }});
}

/**
 * Hash a local evidence file with the vault's warranty-bound SHA-256 scheme,
 * then require a trusted client-side-encryption adapter to persist it privately.
 * The file bytes NEVER appear in a contract transaction.
 */
export async function uploadPrivateEvidence({file,vault,warrantyId,actor,parties}){
 invariant(Number.isSafeInteger(warrantyId)&&warrantyId>0,'invalid_warranty_id');
 invariant(file&&typeof file.arrayBuffer==='function','evidence_file_required');
 invariant(vault?.clientSideEncryption===true&&typeof vault?.encryptAndStore==='function','secure_evidence_vault_unavailable');
 const bytes=new Uint8Array(await file.arrayBuffer());
 invariant(bytes.length>0&&bytes.length<=1024*1024,'invalid_evidence_size');
 const prefix=new TextEncoder().encode(String(warrantyId)+':');
 const input=new Uint8Array(prefix.length+bytes.length);
 input.set(prefix);input.set(bytes,prefix.length);
 const digest=new Uint8Array(await crypto.subtle.digest('SHA-256',input));
 const commitment=Array.from(digest,b=>b.toString(16).padStart(2,'0')).join('');
 const result=await vault.encryptAndStore({warrantyId,actor,parties,bytes});
 // Never accept arbitrary evidence IDs as proof that the vault stored it.
 invariant(result?.commitment===commitment&&typeof result?.receipt==='string'&&result.receipt.length>=12,'evidence_storage_unverified');
 return {commitment,receipt:result.receipt};
}

