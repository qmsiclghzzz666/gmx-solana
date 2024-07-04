import { PublicKey } from "@solana/web3.js";
import { StoreProgram, makeInvoke } from "gmsol";

export const makeSetTokenMapInstruction = async (
    program: StoreProgram,
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
