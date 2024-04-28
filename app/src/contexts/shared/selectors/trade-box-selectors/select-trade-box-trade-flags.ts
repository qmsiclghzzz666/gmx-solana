import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";

export const selectTradeBoxTradeFlags = createSharedStatesSelector([selectTradeBoxState], state => state.tradeFlags);
