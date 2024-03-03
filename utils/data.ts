import * as anchor from "@coral-xyz/anchor";
import { DataStore } from "../target/types/data_store";
import { keyToSeed } from "./seed";

export const dataStore = anchor.workspace.DataStore as anchor.Program<DataStore>;

export const ADDRESS_SEED = anchor.utils.bytes.utf8.encode("address");

export const createAddressPDA = (key: string) => anchor.web3.PublicKey.findProgramAddressSync([
    ADDRESS_SEED,
    keyToSeed(key),
], dataStore.programId);
