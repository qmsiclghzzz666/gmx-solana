import { Address, translateAddress } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { StoreProgram, makeInvoke, toBN } from "gmsol";

export { findConfigPDA } from "gmsol";

export type MakeInitializeConfigParams = {
    authority: PublicKey,
    store: PublicKey,
}

export const makeInsertAmountInstruction = async (
    program: StoreProgram,
    { authority, store, key, amount }: {
        authority: PublicKey,
        store: PublicKey,
        key: string,
        amount: number | bigint,
    }
) => {
    return await program.methods.insertAmount(key, toBN(amount)).accounts({
        authority,
        store,
    }).instruction();
}

export const invokeInsertAmount = makeInvoke(makeInsertAmountInstruction, ["authority"]);

export const makeInsertFactorInstruction = async (
    program: StoreProgram,
    { authority, store, key, factor }: {
        authority: PublicKey,
        store: PublicKey,
        key: string,
        factor: number | bigint,
    }
) => {
    return await program.methods.insertFactor(key, toBN(factor)).accounts({
        authority,
        store,
    }).instruction();
}

export const invokeInsertFactor = makeInvoke(makeInsertFactorInstruction, ["authority"]);

export const makeInsertAddressInstruction = async (
    program: StoreProgram,
    { authority, store, key, address }: {
        authority: PublicKey,
        store: PublicKey,
        key: string,
        address: Address,
    }
) => {
    return await program.methods.insertAddress(key, translateAddress(address)).accounts({
        authority,
        store,
    }).instruction();
}

export const invokeInsertAddress = makeInvoke(makeInsertAddressInstruction, ["authority"]);
