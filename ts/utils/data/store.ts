import { PublicKey } from "@solana/web3.js";
import { DataStoreProgram, makeInvoke } from "gmsol";

export const makeSetTokenMapInstruction = async (
    program: DataStoreProgram,
    { authority, store, tokenMap }: {
        authority: PublicKey,
        store: PublicKey,
        tokenMap: PublicKey,
    }
) => {
    return await program.methods.setTokenMap().accountsStrict({
        authority,
        store,
        tokenMap,
    }).instruction();
};

export const invokeSetTokenMap = makeInvoke(makeSetTokenMapInstruction, ["authority"]);
