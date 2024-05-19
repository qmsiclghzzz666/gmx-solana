import { utils } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { dataStore } from "./program";
import { DataStoreProgram, makeInvoke, toBN } from "gmsol";

// Config seed.
export const CONFIG_SEED = utils.bytes.utf8.encode("config");

export const findConfigPDA = (store: PublicKey) => PublicKey.findProgramAddressSync([
    CONFIG_SEED,
    store.toBytes(),
], dataStore.programId);

export type MakeInitializeConfigParams = {
    authority: PublicKey,
    store: PublicKey,
}

export const makeInitializeConfigInstruction = async (
    program: DataStoreProgram,
    { authority, store }: MakeInitializeConfigParams,
) => {
    return await program.methods.initializeConfig().accounts({
        authority,
        store,
    }).instruction();
}

export const invokeInitializeConfig = makeInvoke(makeInitializeConfigInstruction, ["authority"]);

export const makeInsertAmountInstruction = async (
    program: DataStoreProgram,
    { authority, store, key, amount, insertNew }: {
        authority: PublicKey,
        store: PublicKey,
        key: string,
        amount: number | bigint,
        insertNew?: boolean,
    }
) => {
    return await program.methods.insertAmount(key, toBN(amount), insertNew).accounts({
        authority,
        store,
    }).instruction();
}

export const invokeInsertAmount = makeInvoke(makeInsertAmountInstruction, ["authority"]);

export const makeInsertFactorInstruction = async (
    program: DataStoreProgram,
    { authority, store, key, factor, insertNew }: {
        authority: PublicKey,
        store: PublicKey,
        key: string,
        factor: number | bigint,
        insertNew?: boolean,
    }
) => {
    return await program.methods.insertFactor(key, toBN(factor), insertNew).accounts({
        authority,
        store,
    }).instruction();
}

export const invokeInsertFactor = makeInvoke(makeInsertFactorInstruction, ["authority"]);
