import { AnchorProvider, translateAddress } from "@coral-xyz/anchor";
import { createAssociatedTokenAccount, createMint, mintTo } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { provider as DefaultProvider, isDevNet } from "./endpoint";

export const BTC_TOKEN_MINT = translateAddress(isDevNet ? "Hb5pJ53KeUPCkUvaDZm7Y7WafEjuP1xjD4owaXksJ86R" : "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh");
export const BTC_FEED = translateAddress(isDevNet ? "6PxBx93S8x3tno1TsFZwT5VqP8drrRCbCXygEXYNkFJe" : "Cv4T27XbjVoKUYwP72NQQanvZeA7W4YF9L4EnYT9kx5o");
export const SOL_TOKEN_MINT = translateAddress("So11111111111111111111111111111111111111112");
export const SOL_FEED = translateAddress(isDevNet ? "99B2bTijsU6f1GCT73HmdR7HCFFjGMBcPZY6jZ96ynrR" : "CH31Xns5z3M1cTAbKW34jcxPPciazARpijcHj9rxtemt");
export const USDC_FEED = translateAddress(isDevNet ? "2EmfL3MqL3YHABudGNmajjCpR13NNEn9Y4LWxbDm6SwR" : "GzGuoKXE8Unn7Vcg1DtomwD27tL4bVUpSK2M1yk6Xfz5");

const useOrGetProvider = (provider?: AnchorProvider) => {
    return provider ?? DefaultProvider
}

export class SignedToken {
    signer: Keypair;
    mint: PublicKey;
    decimals: number;

    constructor(signer: Keypair, mint: PublicKey, decimals: number) {
        this.signer = signer;
        this.mint = mint;
        this.decimals = decimals;
    }

    async createTokenAccount(owner: PublicKey, provider?: AnchorProvider) {
        return createAssociatedTokenAccount(useOrGetProvider(provider).connection, this.signer, this.mint, owner);
    }

    async mintTo(destination: PublicKey, amount: number | bigint, provider?: AnchorProvider) {
        return await mintTo(useOrGetProvider(provider).connection, this.signer, this.mint, destination, this.signer, amount);
    }
}

export const createSignedToken = async (signer: Keypair, decimals: number, provider?: AnchorProvider) => {
    const mint = await createMint(useOrGetProvider(provider).connection, signer, signer.publicKey, signer.publicKey, decimals);
    return new SignedToken(signer, mint, decimals);
};
