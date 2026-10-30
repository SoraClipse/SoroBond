import {readFile,writeFile,rename,mkdir} from 'node:fs/promises';
import {dirname} from 'node:path';
const HASH=/^[0-9a-fA-F]{64}$/;
export class WarrantyIndexer {
 constructor({contractId,network,store,finality=2}){if(!contractId||!['testnet','public'].includes(network)||!store||!Number.isInteger(finality)||finality<1)throw Error('invalid_config');this.contractId=contractId;this.network=network;this.store=store;this.finality=finality;this.checkpoint=0;this.events=new Map();}
 async load(){try{const data=JSON.parse(await readFile(this.store,'utf8'));if(data.network!==this.network||data.contractId!==this.contractId)throw Error('store_identity_mismatch');this.checkpoint=data.checkpoint;this.events=new Map(data.events);}catch(e){if(e.code!=='ENOENT')throw e;}}
 async save(){await mkdir(dirname(this.store),{recursive:true});const temp=this.store+'.tmp';await writeFile(temp,JSON.stringify({network:this.network,contractId:this.contractId,checkpoint:this.checkpoint,events:[...this.events]}));await rename(temp,this.store);}
 apply({latestLedger,events}){if(!Number.isSafeInteger(latestLedger)||latestLedger<0||!Array.isArray(events))throw Error('invalid_batch');const upper=latestLedger-this.finality;if(upper<this.checkpoint)return;
  const ordered=[...events].sort((a,b)=>a.ledger-b.ledger||a.index-b.index);
  for(const e of ordered){if(e.contractId!==this.contractId||e.network!==this.network)continue;if(!Number.isSafeInteger(e.ledger)||e.ledger<=0||e.ledger>upper||!Number.isSafeInteger(e.index)||e.index<0||!HASH.test(e.txHash??''))continue;
   if(!['WarrantyCreated'].includes(e.type)||!Number.isSafeInteger(e.warrantyId)||e.warrantyId<1)continue;
   if(e.ledger<this.checkpoint)continue;
   this.events.set(e.ledger+':'+e.txHash.toLowerCase()+':'+e.index,{ledger:e.ledger,id:e.warrantyId,type:e.type,txHash:e.txHash.toLowerCase()});
  }
  // A gap cannot be silently marked complete: caller supplies contiguous covered ledger range.
  this.checkpoint=upper;
 }
 rollback(firstRevertedLedger){if(!Number.isSafeInteger(firstRevertedLedger)||firstRevertedLedger<1)throw Error('invalid_rollback');for(const [key,value] of this.events)if(value.ledger>=firstRevertedLedger)this.events.delete(key);this.checkpoint=Math.min(this.checkpoint,firstRevertedLedger-1);}
 getWarranty(id){if(!Number.isSafeInteger(id)||id<1)throw Error('invalid_id');return [...this.events.values()].filter(e=>e.id===id).sort((a,b)=>a.ledger-b.ledger);}
 async sync({latestLedger,readRange}){const end=latestLedger-this.finality;if(end<=this.checkpoint)return;const from=this.checkpoint+1;const events=await readRange(from,end);if(!Array.isArray(events))throw Error('provider_unavailable');this.apply({latestLedger,events});await this.save();}
}
