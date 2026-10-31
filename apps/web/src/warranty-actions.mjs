// Network-bound transaction coordinator. A UI must never mark a transaction
// funded until RPC confirms SUCCESS and the exact contract state is verified.
const HEX64=/^[0-9a-f]{64}$/i;
const NONNEG=/^(0|[1-9][0-9]*)$/;
export const CLAIM_GRACE_SECONDS=86400;
export function invariant(ok,code){if(!ok)throw new Error(code);}
export function amount(value,{positive=true}={}){
 const s=String(value);
 invariant(NONNEG.test(s),'invalid_amount');
 const n=BigInt(s);
 invariant(!positive||n>0n,'invalid_amount');
 invariant(n<=(2n**127n)-1n,'amount_overflow');
 return n;
}
export function formatAmount(value,decimals){
 invariant(Number.isSafeInteger(decimals)&&decimals>=0&&decimals<=18,'invalid_decimals');
 const n=amount(value,{positive:false}),div=10n**BigInt(decimals);
 return decimals===0?n.toString():(n/div)+'.'+(n%div).toString().padStart(decimals,'0');
}
export function validHash(value){invariant(typeof value==='string'&&HEX64.test(value),'invalid_hash');return value.toLowerCase();}
export function verifyContext({network,contractId,rpc,chain}){
 invariant(network==='testnet'||network==='public','invalid_network');
 invariant(rpc?.network===network && typeof rpc?.prepare==='function' && typeof rpc?.submitSigned==='function' && typeof rpc?.status==='function','rpc_not_configured');
 invariant(typeof contractId==='string'&&contractId.length>=10,'invalid_contract');
 invariant(chain?.network===network && chain?.contractId===contractId,'wrong_contract_or_network');
}
export async function submitAndVerify({network,contractId,rpc,chain,wallet,method,args,actor,validate,after,maxPolls=12,wait=async()=>{}}){
 verifyContext({network,contractId,rpc,chain});
 invariant(typeof wallet?.connect==='function'&&typeof wallet?.signTransaction==='function','wallet_not_connected');
 invariant(typeof chain?.buildInvocation==='function','transaction_builder_missing');
 invariant(Number.isSafeInteger(maxPolls)&&maxPolls>0&&maxPolls<=60,'invalid_poll_count');
 const identity=await wallet.connect();
 invariant(identity?.network===network,'wallet_network_mismatch');
 invariant(identity?.address===actor,'unauthorized_actor');
 if(validate)await validate(identity);
 // The adapter constructs a genuine Soroban invoke (signed authorization,
 // resource footprint and fees); this module never handles private keys.
 const unsigned=await chain.buildInvocation({contractId,network,method,args,actor});
 invariant(typeof unsigned==='string'&&unsigned.length>0,'unsigned_transaction_missing');
 const prepared=await rpc.prepare(unsigned);
 invariant(prepared.network===network,'wallet_network_mismatch');
 const signed=await wallet.signTransaction(prepared,{contractId,network,method,args});
 invariant(typeof signed==='string'&&signed.length>0,'wallet_rejected');
 const submission=await rpc.submitSigned(signed,network);
 invariant(submission.status!=='FAILED','transaction_rejected');
 validHash(submission.hash);
 let confirmed;
 for(let i=0;i<maxPolls;i++){
  const current=await rpc.status(submission.hash);
  if(current.status==='FAILED')throw Error('transaction_failed');
  if(current.status==='CONFIRMED'){confirmed=current;break;}
  if(i<maxPolls-1)await wait();
 }
 invariant(confirmed&&Number.isSafeInteger(confirmed.ledger)&&confirmed.ledger>0,'not_confirmed');
 invariant(typeof after==='function','verification_missing');
 const state=await after({hash:submission.hash,ledger:confirmed.ledger});
 invariant(state!==false&&state!==null&&state!==undefined,'onchain_state_mismatch');
 return {hash:submission.hash,ledger:confirmed.ledger,state,status:'CONFIRMED'};
}
export function renderStatus(w){
 invariant(w&&typeof w==='object'&&['Pending','Active','Claimed','Settled','Expired'].includes(w.status),'invalid_warranty_state');
 return {id:w.id,status:w.status,token:w.token,bondAmount:String(w.bond_amount),seller:w.seller,buyer:w.buyer,arbitrator:w.arbitrator,expiresAt:w.expires_at,offerExpiresAt:w.offer_expires_at};
}
