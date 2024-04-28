import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";

export const selectTradeBoxSetFromTokenAddress = createSharedStatesSelector([selectTradeBoxState], state => state.setFromTokenAddress);
