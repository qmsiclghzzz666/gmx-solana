import { DEFAULT_PUSH_ORACLE_PROGRAM_ID } from "@pythnetwork/pyth-solana-receiver/lib/address";
import { PublicKey } from "@solana/web3.js";

export const findPythPriceFeedPDA = (shardId: number, priceFeedId: Buffer) => {
    const shardBuffer = Buffer.alloc(2);
    shardBuffer.writeUint16LE(shardId, 0);
    return PublicKey.findProgramAddressSync([
        shardBuffer,
        priceFeedId,
    ], DEFAULT_PUSH_ORACLE_PROGRAM_ID)
};
