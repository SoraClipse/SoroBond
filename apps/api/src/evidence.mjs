import {createHash,randomBytes,createCipheriv,createDecipheriv} from 'node:crypto';
export const MAX_EVIDENCE_BYTES=1024*1024;
export class EvidenceVault {
 constructor(key){if(!Buffer.isBuffer(key)||key.length!==32)throw Error('invalid_encryption_key');this.key=key;this.records=new Map();}
 put({warrantyId,plaintext,parties,expiresAt,now=Date.now()}){
  if(!Number.isSafeInteger(warrantyId)||warrantyId<1||!Buffer.isBuffer(plaintext)||plaintext.length<1||plaintext.length>MAX_EVIDENCE_BYTES)throw Error('invalid_evidence');
  if(!Array.isArray(parties)||parties.length<2||parties.some(p=>typeof p!=='string'||!p)||!Number.isSafeInteger(expiresAt)||expiresAt<=now)throw Error('invalid_access_policy');
  const iv=randomBytes(12),cipher=createCipheriv('aes-256-gcm',this.key,iv);const sealed=Buffer.concat([cipher.update(plaintext),cipher.final()]);const commitment=createHash('sha256').update(Buffer.from(String(warrantyId)+':')).update(plaintext).digest('hex');
  const id=randomBytes(20).toString('hex');this.records.set(id,{warrantyId,parties:[...new Set(parties)],expiresAt,commitment,iv,tag:cipher.getAuthTag(),sealed});return {id,commitment};
 }
 get({id,actor,warrantyId,expectedCommitment,now=Date.now()}){const r=this.records.get(id);
  if(!r||!r.parties.includes(actor)||r.expiresAt<=now||r.warrantyId!==warrantyId)throw Error('not_found');
  if(expectedCommitment!==r.commitment)throw Error('commitment_mismatch');
  const d=createDecipheriv('aes-256-gcm',this.key,r.iv);d.setAuthTag(r.tag);const clear=Buffer.concat([d.update(r.sealed),d.final()]);
  if(createHash('sha256').update(Buffer.from(String(warrantyId)+':')).update(clear).digest('hex')!==r.commitment)throw Error('commitment_mismatch');return clear;
 }
 purge(now=Date.now()){for(const [id,r] of this.records)if(r.expiresAt<=now)this.records.delete(id);}
}
