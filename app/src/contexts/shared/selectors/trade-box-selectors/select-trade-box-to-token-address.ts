import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";

export const selectTradeBoxToTokenAddress = createSharedStatesSelector([selectTradeBoxState], state => state.toTokenAddress);
