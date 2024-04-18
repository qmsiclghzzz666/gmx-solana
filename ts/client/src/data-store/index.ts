import { Program, workspace } from "@coral-xyz/anchor";
import { DataStore } from "../idl/data_store";

export const dataStore = workspace.DataStore as Program<DataStore>;
