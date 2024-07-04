import { Program, Provider } from "@coral-xyz/anchor";

import { GmsolStore } from "./idl/gmsol_store";
import { GmsolExchange } from "./idl/gmsol_exchange";

import GmsolStoreIDL from "./idl/gmsol_store.json";
import GmsolExchangeIDL from "./idl/gmsol_exchange.json";
import { PublicKey } from "@solana/web3.js";

export type StoreProgram = Program<GmsolStore>;
export type ExchangeProgram = Program<GmsolExchange>;

/**
 * Creates an instance of the Program with the provided IDL schema for the GmsolStore program.
 * 
 * @param provider Optional. The Anchor provider to be used for interacting with the program.
 *                 If no provider is specified, a default provider (if available) will be used.
 * @returns An instance of the Program configured with the GmsolStore's IDL.
 */
export const makeStoreProgram = (provider?: Provider) => new Program(GmsolStoreIDL as GmsolStore, provider);

/**
 * Creates an instance of the Program with the provided IDL schema for the GmsolExchange program.
 * 
 * @param provider Optional. The Anchor provider to be used for interacting with the program.
 *                 If no provider is specified, a default provider (if available) will be used.
 * @returns An instance of the Program configured with the GmsolExchange's IDL.
 */
export const makeExchangeProgram = (provider?: Provider) => new Program(GmsolExchangeIDL as GmsolExchange, provider);

export const STORE_PROGRAM_ID: PublicKey = new PublicKey(GmsolStoreIDL.address);
export const EXCHANGE_PROGRAM_ID: PublicKey = new PublicKey(GmsolExchangeIDL.address);
