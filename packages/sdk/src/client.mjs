import {createHash} from 'node:crypto';
export const PASSPHRASES=Object.freeze({testnet:'Test SDF Network ; September 2015',public:'Public Global Stellar Network ; September 2015'});
export class RpcError extends Error {constructor(code){super(code);this.code=code;}}
const HASH=/^[0-9a-fA-F]{64}$/;
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
export class SorobanClient{
 constructor({network='testnet',endpoint,fetcher=fetch,timeout=8000,retries=1}={}){
  if(!PASSPHRASES[network])throw new RpcError('unsupported_network');
  const expected=network==='testnet'?'https://soroban-testnet.stellar.org':'https://soroban.stellar.org';
  if(endpoint!==undefined && endpoint!==expected)throw new RpcError('untrusted_endpoint');
  this.network=network;this.endpoint=expected;this.fetcher=fetcher;this.timeout=timeout;this.retries=retries;this.checked=false;this.sequence=0;
 }
 async rpc(method,params={},retry=true){
  if(!Number.isInteger(this.retries)||this.retries<0||this.retries>3||this.timeout<100||this.timeout>20000)throw new RpcError('invalid_settings');
  for(let attempt=0;attempt<=(retry?this.retries:0);attempt++){
   try{
    const response=await this.fetcher(this.endpoint,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({jsonrpc:'2.0',id:++this.sequence,method,params}),signal:AbortSignal.timeout(this.timeout),redirect:'error'});
    if(!response.ok)throw new RpcError('rpc_unavailable');
    const raw=await response.text();if(raw.length>262144)throw new RpcError('oversized_response');
    const data=JSON.parse(raw);if(data.error)throw new RpcError('rpc_rejected');
    if(!data.result||typeof data.result!=='object')throw new RpcError('malformed_response');
    return data.result;
   }catch(error){if(error instanceof RpcError&&['rpc_rejected','malformed_response','oversized_response'].includes(error.code))throw error;
    if(attempt===(retry?this.retries:0))throw new RpcError('rpc_unavailable');await sleep(50*(attempt+1));
   }
  }
 }
 async checkNetwork(){const response=await this.rpc('getNetwork');if(response.passphrase!==PASSPHRASES[this.network])throw new RpcError('wrong_network');this.checked=true;return response;}
 async simulate(envelope){if(!this.checked)await this.checkNetwork();if(typeof envelope!=='string'||!envelope||envelope.length>131072)throw new RpcError('invalid_envelope');const result=await this.rpc('simulateTransaction',{transaction:envelope});
  if(result.error)throw new RpcError('simulation_failed');return result;}
 async prepare(envelope){const simulation=await this.simulate(envelope);if(!simulation.transactionData||simulation.minResourceFee===undefined)throw new RpcError('missing_fee_estimate');return {simulation,network:this.network,passphrase:PASSPHRASES[this.network],envelope};}
 async submitSigned(signedXdr,signerConfirmedNetwork){if(signerConfirmedNetwork!==this.network)throw new RpcError('wallet_network_mismatch');if(!this.checked)await this.checkNetwork();if(typeof signedXdr!=='string'||!signedXdr||signedXdr.length>131072)throw new RpcError('invalid_envelope');const result=await this.rpc('sendTransaction',{transaction:signedXdr},false);if(!HASH.test(result.hash??''))throw new RpcError('invalid_transaction_hash');return {hash:result.hash.toLowerCase(),status:result.status==='PENDING'?'PENDING':result.status==='ERROR'?'FAILED':'SUBMITTED'};}
 async status(hash){if(!HASH.test(hash??''))throw new RpcError('invalid_transaction_hash');const result=await this.rpc('getTransaction',{hash:hash.toLowerCase()});const status=result.status;
  if(status==='NOT_FOUND'||status==='PENDING')return {status:'PENDING',hash:hash.toLowerCase()};
  if(status==='FAILED')return {status:'FAILED',hash:hash.toLowerCase()};
  if(status==='SUCCESS'&&Number.isSafeInteger(result.ledger)&&result.ledger>0)return {status:'CONFIRMED',hash:hash.toLowerCase(),ledger:result.ledger};
  throw new RpcError('invalid_transaction_status');
 }
}
