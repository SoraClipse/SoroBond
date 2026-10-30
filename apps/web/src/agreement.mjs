import {createHash} from 'node:crypto';
export const PASSPHRASES={testnet:'Test SDF Network ; September 2015',public:'Public Global Stellar Network ; September 2015'};
const validText=x=>typeof x==='string'&&x.trim().length>0&&x.length<=500;
export function canonicalTerms(t){
 if(!t||!['testnet','public'].includes(t.network)||!['XLM','USDC'].includes(t.asset)||!['buyer','seller'].every(k=>validText(t[k]))||t.buyer===t.seller||!validText(t.item)||!validText(t.coverage)||!validText(t.exclusions)||!validText(t.resolver)||!/^([1-9]\d*)(?:\.\d{1,7})?$/.test(t.bondAmount??'')||!Number.isSafeInteger(t.expiresAt)||t.expiresAt<=0)throw Error('invalid_agreement');
 return JSON.stringify({network:t.network,asset:t.asset,buyer:t.buyer,seller:t.seller,item:t.item,coverage:t.coverage,exclusions:t.exclusions,resolver:t.resolver,bondAmount:t.bondAmount,expiresAt:t.expiresAt});
}
export function termsDigest(t){return createHash('sha256').update(canonicalTerms(t)).digest('hex');}
export async function coSign(t,signer,expectedDigest){const digest=termsDigest(t);if(expectedDigest!==digest)throw Error('terms_changed');if(!signer?.connected||signer.network!==t.network)throw Error('wallet_network_mismatch');if(signer.address!==t.buyer&&signer.address!==t.seller)throw Error('unauthorized_signer');const signature=await signer.sign(digest,PASSPHRASES[t.network]);if(!signature)throw Error('signature_declined');return {address:signer.address,digest,signature,network:t.network};}
export function pairSignatures(t,a,b){const digest=termsDigest(t);if(a?.digest!==digest||b?.digest!==digest||a.address===b.address||!([a.address,b.address].includes(t.buyer)&&[a.address,b.address].includes(t.seller)))throw Error('invalid_signatures');return {digest,signatures:[a,b]};}
