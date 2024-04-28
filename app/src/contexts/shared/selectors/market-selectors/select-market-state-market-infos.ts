import { createSharedStatesSelector } from "../../utils";
import { selectMarketState } from "./select-market-state";

export const selectMarketStateMarketInfos = createSharedStatesSelector(selectMarketState, state => state.marketInfos);
