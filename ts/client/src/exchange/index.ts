export * from "./instructions";

import { utils } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { EXCHANGE_ID } from "../program";

export const findControllerPDA = (store: PublicKey) => PublicKey.findProgramAddressSync([
    utils.bytes.utf8.encode("controller"),
    store.toBuffer(),
], EXCHANGE_ID);
