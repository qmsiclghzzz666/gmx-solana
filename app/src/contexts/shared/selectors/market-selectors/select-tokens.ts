import { createSharedStatesSelector } from "../../utils";
import { selectMarketState } from "./select-market-state";

export const selectTokens = createSharedStatesSelector([selectMarketState], state => state.tokens);

