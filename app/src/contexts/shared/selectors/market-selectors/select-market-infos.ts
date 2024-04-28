import { createSharedStatesSelector } from "../../utils";
import { selectMarketState } from "./select-market-state";

export const selectMarketInfos = createSharedStatesSelector(selectMarketState, state => state.marketInfos);
