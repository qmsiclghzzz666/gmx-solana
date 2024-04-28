import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";

export const selectTradeBoxFromTokenAddress = createSharedStatesSelector([selectTradeBoxState], state => state.fromTokenAddress);
