import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";

export const selectTradeBoxTradeMode = createSharedStatesSelector([selectTradeBoxState], state => state.tradeMode);
