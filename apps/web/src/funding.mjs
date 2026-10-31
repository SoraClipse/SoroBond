import {amount,formatAmount,invariant,renderStatus,submitAndVerify,validHash,verifyContext} from './warranty-actions.mjs';

export async function fundingQuote({rpc,chain,network,contractId,seller,token,bondAmount,decimals}){
 verifyContext({rpc,chain,network,contractId});
 invariant(typeof seller==='string'&&seller.length>0,'invalid_seller');
 invariant(typeof token==='string'&&token.length>0,'invalid_token');
 const bond=amount(bondAmount);
 invariant(typeof chain.readPinnedToken==='function'&&typeof chain.readBalance==='function'&&typeof chain.readLockedTotal==='function','chain_reader_missing');
 const pinned=await chain.readPinnedToken();
 invariant(pinned===token,'wrong_token');
 const [balance,liability]=await Promise.all([chain.readBalance(token,seller),chain.readLockedTotal()]);
 const available=amount(balance,{positive:false}),locked=amount(liability,{positive:false});
 return {token,contractId,seller,network,bondAmount:bond.toString(),displayAmount:formatAmount(bond,decimals),sellerBalance:available.toString(),lockedTotal:locked.toString(),sufficient:available>=bond,funded:false};
}
export async function fundWarranty({rpc,chain,wallet,network,contractId,seller,buyer,arbitrator,token,bondAmount,decimals,durationSeconds,termsHash,...options}){
 const q=await fundingQuote({rpc,chain,network,contractId,seller,token,bondAmount,decimals});
 invariant(q.sufficient,'insufficient_balance');
 invariant(typeof buyer==='string'&&buyer!==seller&&typeof arbitrator==='string'&&arbitrator!==seller&&arbitrator!==buyer,'invalid_participants');
 invariant(Number.isSafeInteger(durationSeconds)&&durationSeconds>=3600&&durationSeconds<=90*86400,'invalid_duration');
 const digest=validHash(termsHash);
 return submitAndVerify({rpc,chain,wallet,network,contractId,actor:seller,method:'create',args:[seller,buyer,arbitrator,q.bondAmount,durationSeconds,digest],maxPolls:options.maxPolls,wait:options.wait,
  validate:async()=>invariant(await chain.readPinnedToken()===token,'wrong_token'),
  after:async({hash})=>{
   invariant(typeof chain.readCreatedWarranty==='function','receipt_reader_missing');
   const w=await chain.readCreatedWarranty(hash);
   invariant(w&&w.status==='Pending'&&w.seller===seller&&w.buyer===buyer&&w.arbitrator===arbitrator&&w.token===token&&String(w.bond_amount)===q.bondAmount&&w.terms_hash?.toLowerCase()===digest,'onchain_state_mismatch');
   return renderStatus(w);
  }});
}
export async function fundingDashboard({rpc,chain,network,contractId,id,decimals}){
 verifyContext({rpc,chain,network,contractId});
 invariant(Number.isSafeInteger(id)&&id>0,'invalid_warranty_id');
 const w=await chain.readWarranty(id);
 invariant(w&&w.id===id,'warranty_not_found');
 const token=await chain.readPinnedToken();
 invariant(w.token===token,'wrong_token');
 const liability=await chain.readLockedTotal();
 const balance=await chain.readBalance(token,contractId);
 invariant(amount(balance,{positive:false})>=amount(liability,{positive:false}),'vault_underfunded');
 return {...renderStatus(w),displayAmount:formatAmount(w.bond_amount,decimals),lockedTotal:String(liability),vaultBalance:String(balance),confirmed:true};
}
