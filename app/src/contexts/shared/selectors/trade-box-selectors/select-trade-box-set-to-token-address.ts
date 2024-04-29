import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";

export const selectTradeBoxSetToTokenAddress = createSharedStatesSelector([selectTradeBoxState], state => state.setToTokenAddress);
