export * from "./types";
export * from "./StateProvider";
export * from "./hooks";

import { createContext } from "use-context-selector";
import { State } from "./types";

export const StateCtx = createContext<State | null>(null);
