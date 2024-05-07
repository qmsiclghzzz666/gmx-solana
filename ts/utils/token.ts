import { AnchorProvider, translateAddress } from "@coral-xyz/anchor";
import { createAssociatedTokenAccount, createMint, mintTo } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { provider as DefaultProvider, isDevNet } from "./endpoint";

export const BTC_TOKEN_MINT = translateAddress(isDevNet ? "Hb5pJ53KeUPCkUvaDZm7Y7WafEjuP1xjD4owaXksJ86R" : "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh");
export const SOL_TOKEN_MINT = translateAddress("So11111111111111111111111111111111111111112");

export const BTC_FEED = translateAddress(isDevNet ? "6PxBx93S8x3tno1TsFZwT5VqP8drrRCbCXygEXYNkFJe" : "Cv4T27XbjVoKUYwP72NQQanvZeA7W4YF9L4EnYT9kx5o");
export const SOL_FEED = translateAddress(isDevNet ? "99B2bTijsU6f1GCT73HmdR7HCFFjGMBcPZY6jZ96ynrR" : "CH31Xns5z3M1cTAbKW34jcxPPciazARpijcHj9rxtemt");
export const USDC_FEED = translateAddress(isDevNet ? "2EmfL3MqL3YHABudGNmajjCpR13NNEn9Y4LWxbDm6SwR" : "GzGuoKXE8Unn7Vcg1DtomwD27tL4bVUpSK2M1yk6Xfz5");

export const BTC_FEED_PYTH = translateAddress("4cSM2e6rvbGQUFiJbqytoVMi5GgghSMr8LwVrT9VPSPo");
export const SOL_FEED_PYTH = translateAddress("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
export const USDC_FEED_PYTH = translateAddress("Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX");
export const BTC_FEED_ID = "0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43";
export const SOL_FEED_ID = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
export const USDC_FEED_ID = "0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";

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
