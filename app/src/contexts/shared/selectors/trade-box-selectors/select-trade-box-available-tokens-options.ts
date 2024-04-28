import { createSharedStatesSelector } from "../../utils";
import { selectTradeBoxState } from "./select-trade-box-state";

export const selectTradeBoxAvailableTokensOptions = createSharedStatesSelector([selectTradeBoxState], state => state.avaiableTokensOptions);
