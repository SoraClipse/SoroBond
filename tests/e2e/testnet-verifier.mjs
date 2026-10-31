// Inspect real Stellar Testnet transaction evidence. Fake/mock receipts are NOT proof.
const SHA=/^[0-9a-f]{64}$/i;
const SCENARIOS=Object.freeze({
 expiry:['created','activated','expired'],
 mutual:['created','activated','claimed','settled'],
 arbitration:['created','activated','claimed','arbitrated'],
});
const FAILURES=['unauthorized_settlement','double_withdraw'];
function insist(condition,code){if(!condition)throw Error(code);}
export function inspectManifest(m){
 insist(m&&m.network==='testnet'&&typeof m.contractId==='string'&&m.contractId.startsWith('C')&&m.contractId.length===56,'invalid_testnet_contract');
 insist(typeof m.token==='string'&&m.token.startsWith('C')&&m.token.length===56,'invalid_testnet_token');
 const seen=new Set(),ids=new Set();
 for(const [name,phases] of Object.entries(SCENARIOS)){
  const scenario=m.scenarios?.[name];
  insist(scenario&&Number.isSafeInteger(scenario.warrantyId)&&scenario.warrantyId>0,'missing_scenario_'+name);
  insist(!ids.has(scenario.warrantyId),'reused_warranty');ids.add(scenario.warrantyId);
  for(const phase of phases){
   const entry=scenario.transactions?.[phase];
   insist(entry&&SHA.test(entry.hash??''),'missing_transaction_'+name+'_'+phase);
   insist(!seen.has(entry.hash.toLowerCase()),'reused_transaction');seen.add(entry.hash.toLowerCase());
  }
 }
 for(const name of FAILURES){
  const proof=m.negative?.[name];
  insist(proof&&SHA.test(proof.hash??''),'missing_negative_'+name);
  insist(!seen.has(proof.hash.toLowerCase()),'reused_transaction');seen.add(proof.hash.toLowerCase());
 }
 return {transactions:seen.size,warranties:ids.size};
}
export async function verifyTestnetManifest({manifest,rpc,reader}){
 const stats=inspectManifest(manifest);
 insist(rpc?.network==='testnet'&&reader?.network==='testnet'&&reader?.contractId===manifest.contractId,'untrusted_environment');
 insist(typeof rpc.checkNetwork==='function'&&typeof rpc.status==='function'&&typeof reader.readReceipt==='function','missing_network_reader');
 const actual=await rpc.checkNetwork();
 insist(actual?.passphrase==='Test SDF Network ; September 2015','wrong_network');
 const entries=[];
 for(const [name,phases] of Object.entries(SCENARIOS)){
  const scenario=manifest.scenarios[name];let lastLedger=0;
  for(const phase of phases){
   const proof=scenario.transactions[phase],tx=await rpc.status(proof.hash);
   insist(tx.status==='CONFIRMED'&&Number.isSafeInteger(tx.ledger)&&tx.ledger>lastLedger,'unconfirmed_or_out_of_order_'+phase);
   lastLedger=tx.ledger;
   // Reader must resolve actual Soroban RPC events, contract identity and token.
   const receipt=await reader.readReceipt(proof.hash);
   insist(receipt?.contractId===manifest.contractId&&receipt?.transactionHash?.toLowerCase()===proof.hash.toLowerCase()&&receipt?.warrantyId===scenario.warrantyId&&receipt?.phase===phase&&receipt?.ledger===tx.ledger,'invalid_event_receipt_'+phase);
   insist(receipt.token===manifest.token,'wrong_asset_receipt');
   if(phase==='expired')insist(receipt.buyerPayout==='0'&&receipt.sellerReturn===receipt.bondAmount,'invalid_expiry_accounting');
   if(phase==='settled'||phase==='arbitrated')insist(/^(0|[1-9][0-9]*)$/.test(receipt.buyerPayout??'')&&BigInt(receipt.buyerPayout)+BigInt(receipt.sellerReturn)===BigInt(receipt.bondAmount),'invalid_settlement_accounting');
   entries.push({scenario:name,phase,hash:proof.hash,ledger:tx.ledger});
  }
 }
 for(const name of FAILURES){
  const proof=manifest.negative[name],tx=await rpc.status(proof.hash);
  insist(tx.status==='FAILED','negative_not_rejected_'+name);
  entries.push({scenario:'negative',phase:name,hash:proof.hash,status:'FAILED'});
 }
 return {verified:true,network:'testnet',contractId:manifest.contractId,stats,transactions:entries};
}
