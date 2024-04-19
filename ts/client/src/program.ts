import { Program, Provider } from "@coral-xyz/anchor";

import { DataStore } from "./idl/data_store";
import { Exchange } from "./idl/exchange";

import DataStoreIDL from "./idl/data_store.json";
import ExchangeIDL from "./idl/exchange.json";
import { PublicKey } from "@solana/web3.js";

/**
 * Creates an instance of the Program with the provided IDL schema for the DataStore program.
 * 
 * @param provider Optional. The Anchor provider to be used for interacting with the program.
 *                 If no provider is specified, a default provider (if available) will be used.
 * @returns An instance of the Program configured with the DataStore's IDL.
 */
export const makeDataStoreProgram = (provider?: Provider) => new Program(DataStoreIDL as DataStore, provider);

/**
 * Creates an instance of the Program with the provided IDL schema for the Exchange program.
 * 
 * @param provider Optional. The Anchor provider to be used for interacting with the program.
 *                 If no provider is specified, a default provider (if available) will be used.
 * @returns An instance of the Program configured with the Exchange's IDL.
 */
export const makeExchangeProgram = (provider?: Provider) => new Program(ExchangeIDL as Exchange, provider);

export const DATA_STORE_ID: PublicKey = new PublicKey(DataStoreIDL.address);
export const EXCHANGE_ID: PublicKey = new PublicKey(ExchangeIDL.address);
