import * as anchor from "@coral-xyz/anchor";
import { DataStore } from "../target/types/data_store";
import { sha256 } from "js-sha256";

export const dataStore = anchor.workspace.DataStore as anchor.Program<DataStore>;

export const ADDRESS_SEED = anchor.utils.bytes.utf8.encode("address");

export const keyToSeed = (key: string) => anchor.utils.bytes.hex.decode(sha256(key));

export const createAddressPDA = (key: string) => anchor.web3.PublicKey.findProgramAddressSync([
    ADDRESS_SEED,
    keyToSeed(key),
], dataStore.programId);
