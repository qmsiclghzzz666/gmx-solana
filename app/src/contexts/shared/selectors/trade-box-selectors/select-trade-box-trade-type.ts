import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";

export const selectTradeBoxTradeType = createSharedStatesSelector([selectTradeBoxState], state => state.tradeType);
