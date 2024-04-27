import { MarketState } from "../types";
import { useSharedStatesSelector } from "./use-shared-states-selector";

export const useMarketStateSelector = <T>(selector: (s: MarketState) => T) => {
  return useSharedStatesSelector(s => selector(s.market));
};
