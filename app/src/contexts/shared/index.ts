export * from "./types";
export * from "./SharedStatesProvider";
export * from "./hooks";

import { createContext } from "use-context-selector";
import { SharedStates } from "./types";

export const SharedStatesCtx = createContext<SharedStates | null>(null);
