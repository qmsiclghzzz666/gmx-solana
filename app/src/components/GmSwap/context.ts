import React, { Dispatch } from "react";
import { createContext } from "use-context-selector";
import { Action, GmState } from "./types";

export const GmStateContext = createContext<GmState | null>(null);
export const GmStateDispatchContext = React.createContext<Dispatch<Action> | null>(null);
