import { DataStore } from "../../target/types/data_store";
import { anchor } from "../endpoint";

export const dataStore = anchor.workspace.DataStore as anchor.Program<DataStore>;
