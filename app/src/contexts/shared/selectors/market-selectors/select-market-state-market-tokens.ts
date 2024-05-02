import { createSharedStatesSelector } from "../../utils";
import { selectMarketState } from "./select-market-state";

export const selectMarketStateMarketTokens = createSharedStatesSelector([selectMarketState], state => state.marketTokens);
