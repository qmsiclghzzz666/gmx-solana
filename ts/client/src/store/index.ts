import { PublicKey } from "@solana/web3.js";
import { DATA_STORE_ID } from "../program";
import { utils } from "@coral-xyz/anchor";
import { keyToSeed } from "../utils/seed";

const encodeUtf8 = utils.bytes.utf8.encode;

export const findMarketPDA = (store: PublicKey, token: PublicKey) => PublicKey.findProgramAddressSync([
    encodeUtf8("market"),
    store.toBytes(),
    keyToSeed(token.toBase58()),
], DATA_STORE_ID);
