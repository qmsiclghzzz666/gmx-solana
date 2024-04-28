import { createSelector } from "reselect";
import { SharedStates } from "./types";

export const createSharedStatesSelector = createSelector.withTypes<SharedStates>();
