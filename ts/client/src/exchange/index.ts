export * from "./instructions";

import { utils } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { EXCHANGE_PROGRAM_ID } from "../program";

export const findControllerPDA = (store: PublicKey) => PublicKey.findProgramAddressSync([
    utils.bytes.utf8.encode("controller"),
    store.toBuffer(),
], EXCHANGE_PROGRAM_ID);

export enum PriceProvider {
    Pyth = 0,
    Chainlink = 1,
    PythLegacy = 2,
}
