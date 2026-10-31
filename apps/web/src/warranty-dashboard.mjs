import {fundingQuote,fundWarranty,fundingDashboard} from '/funding.mjs';
import {claimDashboard,submitClaim,resolveClaim,withdrawExpired,uploadPrivateEvidence} from '/claims.mjs';
const get=id=>document.getElementById(id);
const value=id=>get(id).value.trim();
const text=(id,content)=>{get(id).textContent=String(content??'');};
const buttons=['claim','settle','arbitrate','withdraw'];
const context=window.sorobondApp;
let quote=null;
if(context){text('connection','Connected to '+context.network+' contract '+context.contractId+'. Verify this address against the deployment record before signing.');}
else{text('connection','No trusted Soroban integration configured. Transaction buttons are disabled until a wallet, RPC client and contract reader are installed.');}
function settings(){if(!context)throw Error('wallet_and_chain_adapter_required');if(context.network!=='testnet')throw Error('testnet_only_screen');return context;}
function id(){const n=Number(value('warranty-id'));if(!Number.isSafeInteger(n)||n<=0)throw Error('invalid_warranty_id');return n;}
function status(msg){text('status',msg);}
function disableActions(){for(const b of buttons)get(b).disabled=true;}
async function refresh(){
 const s=settings(),warrantyId=id(),who=await s.wallet.connect();
 const [fund,claim]=await Promise.all([fundingDashboard({...s,id:warrantyId,decimals:Number(value('decimals'))}),claimDashboard({...s,id:warrantyId,actor:who.address})]);
 text('contract',s.contractId);text('token',fund.token);text('bond',fund.bondAmount+' ('+fund.displayAmount+' tokens)');
 text('liabilities',fund.lockedTotal);text('bond-status',fund.status);text('deadline',claim.w.expires_at);text('resolver',claim.w.arbitrator);
 text('verdict',claim.verdict?JSON.stringify(claim.verdict,null,2):'No finalized verdict');
 disableActions();
 get('claim').disabled=!claim.capabilities.canClaim||!value('evidence-receipt');get('settle').disabled=!claim.capabilities.canOfferSettlement;
 get('arbitrate').disabled=!claim.capabilities.canArbitrate;get('withdraw').disabled=!claim.capabilities.canWithdraw;
 status('On-chain status confirmed by trusted contract reader: '+fund.status);
}
async function run(handler){
 try{status('Checking signatures and submitting. Not confirmed yet.');const r=await handler();status('CONFIRMED at ledger '+r.ledger+'; transaction '+r.hash+'. Matching contract state verified.');await refresh();}
 catch(error){status('Not confirmed: '+error.message);}
}
get('load').onclick=()=>{disableActions();refresh().catch(e=>status('State unavailable: '+e.message));};
get('quote').onclick=async()=>{
 try{quote=null;get('fund').disabled=true;const s=settings();const data=await fundingQuote({...s,seller:value('seller'),token:value('fund-token'),bondAmount:value('amount'),decimals:Number(value('decimals'))});quote=data;text('quote-details',JSON.stringify(data,null,2));get('fund').disabled=!data.sufficient;}
 catch(e){status('Quote rejected: '+e.message);}
};
get('fund').onclick=()=>run(()=>{
 const s=settings();if(!quote)throw Error('quote_required');
 return fundWarranty({...s,seller:value('seller'),buyer:value('buyer'),arbitrator:value('arbitrator'),token:value('fund-token'),bondAmount:value('amount'),decimals:Number(value('decimals')),durationSeconds:Number(value('duration')),termsHash:value('terms')});
});
get('claim').onclick=()=>run(async()=>{
 const s=settings(),w=await s.wallet.connect();
 return submitClaim({...s,id:id(),buyer:w.address,claimAmount:value('claim-amount'),evidenceHash:value('evidence-hash'),evidenceReceipt:value('evidence-receipt')});
});
get('settle').onclick=()=>run(async()=>{
 const s=settings(),w=await s.wallet.connect();
 return resolveClaim({...s,id:id(),actor:w.address,method:'settle',payout:value('payout')});
});
get('arbitrate').onclick=()=>run(async()=>{
 const s=settings(),w=await s.wallet.connect();
 return resolveClaim({...s,id:id(),actor:w.address,method:'arbitrate',payout:value('payout')});
});
get('withdraw').onclick=()=>run(async()=>{
 const s=settings(),w=await s.wallet.connect();
 return withdrawExpired({...s,id:id(),seller:w.address});
});
const query=new URLSearchParams(location.search);
if(query.has('id')){get('warranty-id').value=query.get('id');get('load').click();}

get('upload-evidence').onclick=async()=>{
 try{
  const s=settings(),w=await s.wallet.connect(),file=get('evidence-file').files[0],warrantyId=id();
  const details=await claimDashboard({...s,id:warrantyId,actor:w.address});
  if(!details.capabilities.canClaim)throw Error('claim_not_allowed');
  const stored=await uploadPrivateEvidence({file,vault:s.evidenceVault,warrantyId,actor:w.address,parties:[details.w.buyer,details.w.seller,details.w.arbitrator]});
  get('evidence-hash').value=stored.commitment;get('evidence-receipt').value=stored.receipt;
  get('claim').disabled=false;
  status('Evidence stored via trusted encrypted vault. Only its commitment can be submitted on-chain.');
 }catch(e){get('evidence-hash').value='';get('evidence-receipt').value='';get('claim').disabled=true;status('Evidence not stored: '+e.message);}
};

