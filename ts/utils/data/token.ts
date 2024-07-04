import { utils } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { STORE_PROGRAM_ID, StoreProgram, IxWithOutput, makeInvoke, toBN } from "gmsol";
import { TIME_WINDOW } from "./constants";
import { toInteger } from "lodash";

export const CLAIMABLE_ACCOUNT_SEED = utils.bytes.utf8.encode("claimable_account");

export const findClaimableAccountPDA = (
    store: PublicKey,
    mint: PublicKey,
    user: PublicKey,
    time_key: bigint | number,
) => {
    const buf = Buffer.alloc(8);
    buf.writeBigInt64BE(BigInt(time_key));
    return PublicKey.findProgramAddressSync([
        CLAIMABLE_ACCOUNT_SEED,
        store.toBuffer(),
        mint.toBuffer(),
        user.toBuffer(),
        buf,
    ], STORE_PROGRAM_ID)
};

export const getTimeKey = (timestamp: bigint | number, window: number) => BigInt(timestamp) / BigInt(window);

export const makeUseClaimableAccountInstruction = async (
    program: StoreProgram,
    { authority, store, user, mint, amount, timestamp }: {
        authority: PublicKey,
        store: PublicKey,
        user: PublicKey,
        mint: PublicKey,
        amount?: bigint | number,
        timestamp?: bigint | number,
    }
) => {
    const current = timestamp ?? (toInteger(Date.now() / 1000));
    const time_key = getTimeKey(current, TIME_WINDOW);
    const [account] = findClaimableAccountPDA(store, mint, user, time_key);
    return [
        await program.methods.useClaimableAccount(
            toBN(current),
            toBN(amount ?? 0),
        ).accounts({
            authority,
            store,
            mint,
            user,
            account,
        }).instruction(),
        current,
    ] satisfies IxWithOutput<bigint | number> as IxWithOutput<bigint | number>;
};

export const invokeUseClaimableAccount = makeInvoke(makeUseClaimableAccountInstruction, ["authority"]);

export const makeCloseEmptyClaimableAccountInstruction = async (
    program: StoreProgram,
    { authority, store, user, mint, timestamp }: {
        authority: PublicKey,
        store: PublicKey,
        user: PublicKey,
        mint: PublicKey,
        timestamp: bigint | number,
    }
) => {
    const time_key = getTimeKey(timestamp, TIME_WINDOW);
    const [account] = findClaimableAccountPDA(store, mint, user, time_key);
    return await program.methods.closeEmptyClaimableAccount(user, toBN(timestamp)).accounts({
        authority,
        store,
        mint,
        account,
    }).instruction();
};

export const invokeCloseEmptyClaimableAccount = makeInvoke(makeCloseEmptyClaimableAccountInstruction, ["authority"]);
