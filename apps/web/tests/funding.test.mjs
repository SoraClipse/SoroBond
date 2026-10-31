import {test} from 'node:test';
import assert from 'node:assert/strict';
import {fundingQuote,fundWarranty,fundingDashboard} from '../src/funding.mjs';
const HASH='a'.repeat(64),TX='b'.repeat(64);
function fixture(){
 let confirmed=false,failed=false,status='PENDING',walletRejected=false;
 const network='testnet',contractId='CDUMMYTESTNETCONTRACTIDENTITY',token='CTOKENTESTNETIDENTITY';
 const params={network,contractId,token,seller:'GSeller',buyer:'GBuyer',arbitrator:'GResolver',bondAmount:'25000000',decimals:7,durationSeconds:3600,termsHash:HASH};
 const record={id:1,status:'Pending',seller:params.seller,buyer:params.buyer,arbitrator:params.arbitrator,token,bond_amount:params.bondAmount,terms_hash:HASH,expires_at:0,offer_expires_at:36000};
 const rpc={network,prepare:async x=>({network,envelope:x}),submitSigned:async()=>({status:'PENDING',hash:TX}),status:async()=>({status:failed?'FAILED':status,ledger:48})};
 const chain={network,contractId,readPinnedToken:async()=>token,readBalance:async(_,who)=>who===contractId?'25000000':'50000000',readLockedTotal:async()=> '25000000',readCreatedWarranty:async()=>confirmed?record:null,readWarranty:async()=>record,buildInvocation:async()=> 'unsigned-XDR'};
 const wallet={connect:async()=>({network,address:params.seller}),signTransaction:async()=>{if(walletRejected)throw Error('wallet_cancelled');return 'signed-XDR';}};
 return {params,rpc,chain,wallet,setConfirmed:v=>confirmed=v,setFailed:v=>failed=v,setStatus:v=>status=v,setWalletRejected:v=>walletRejected=v};
}
test('quote displays exact token amount, destination and liabilities',async()=>{
 const {params,rpc,chain}=fixture();const q=await fundingQuote({...params,rpc,chain});
 assert.equal(q.displayAmount,'2.5000000');assert.equal(q.contractId,params.contractId);assert.equal(q.lockedTotal,'25000000');assert.equal(q.funded,false);assert.equal(q.sufficient,true);
});
test('no funded UI before confirmed RPC and matching on-chain warranty',async()=>{
 const f=fixture();
 await assert.rejects(fundWarranty({...f.params,rpc:f.rpc,chain:f.chain,wallet:f.wallet,maxPolls:2}),/not_confirmed/);
 f.setStatus('CONFIRMED');
 await assert.rejects(fundWarranty({...f.params,rpc:f.rpc,chain:f.chain,wallet:f.wallet}),/onchain_state_mismatch/);
 f.setConfirmed(true);
 const r=await fundWarranty({...f.params,rpc:f.rpc,chain:f.chain,wallet:f.wallet});
 assert.equal(r.status,'CONFIRMED');assert.equal(r.state.status,'Pending');assert.equal(r.ledger,48);
});
test('wrong token, insufficient funds, wrong wallet, cancellation and RPC failure fail closed',async()=>{
 const f=fixture();
 await assert.rejects(fundingQuote({...f.params,token:'CWRONG',rpc:f.rpc,chain:f.chain}),/wrong_token/);
 await assert.rejects(fundWarranty({...f.params,bondAmount:'60000000',rpc:f.rpc,chain:f.chain,wallet:f.wallet}),/insufficient_balance/);
 f.setStatus('CONFIRMED');f.setConfirmed(true);
 f.wallet.connect=async()=>({network:'testnet',address:'GStranger'});
 await assert.rejects(fundWarranty({...f.params,rpc:f.rpc,chain:f.chain,wallet:f.wallet}),/unauthorized_actor/);
 f.wallet.connect=async()=>({network:'testnet',address:f.params.seller});
 f.setWalletRejected(true);
 await assert.rejects(fundWarranty({...f.params,rpc:f.rpc,chain:f.chain,wallet:f.wallet}),/wallet_cancelled/);
 f.setWalletRejected(false);f.setFailed(true);
 await assert.rejects(fundWarranty({...f.params,rpc:f.rpc,chain:f.chain,wallet:f.wallet}),/transaction_failed/);
});
test('dashboard requires exact pinned token and solvency',async()=>{
 const f=fixture();const args={network:f.params.network,contractId:f.params.contractId,rpc:f.rpc,chain:f.chain,id:1,decimals:7};
 assert.equal((await fundingDashboard(args)).confirmed,true);
 f.chain.readBalance=async()=> '1';await assert.rejects(fundingDashboard(args),/vault_underfunded/);
});
