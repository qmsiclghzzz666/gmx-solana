import { selectMarketStateMarketTokens } from "../selectors/market-selectors";
import { useSharedStatesSelector } from "./use-shared-states-selector";

export function useMarketTokens() {
  return useSharedStatesSelector(selectMarketStateMarketTokens);
}
