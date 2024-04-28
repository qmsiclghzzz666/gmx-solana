import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";

export const selectTradeBoxSetTradeParams = createSharedStatesSelector([selectTradeBoxState], state => state.setTradeParams);
