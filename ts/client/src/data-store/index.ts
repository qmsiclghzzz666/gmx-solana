import { Program, Provider, workspace } from "@coral-xyz/anchor";
import { DataStore } from "../idl/data_store";
import IDL from "../idl/data_store.json";

/**
 * Creates an instance of the Program with the provided IDL schema for the DataStore program.
 * 
 * @param provider Optional. The provider to be used for interacting with the blockchain network.
 *                 If no provider is specified, a default provider (if available) will be used.
 * @returns An instance of the Program configured with the DataStore's IDL.
 */
export const makeDataStoreProgram = (provider?: Provider) => new Program(IDL as DataStore, provider);
