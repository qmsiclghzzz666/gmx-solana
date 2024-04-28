import { createSharedStatesSelector } from "../../utils";
import { selectMarketState } from "./select-market-state";

export const selectMarketStateTokens = createSharedStatesSelector([selectMarketState], state => state.tokens);
