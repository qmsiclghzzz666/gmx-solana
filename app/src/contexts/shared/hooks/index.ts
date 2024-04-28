import { selectMarketInfos } from "../selectors/market-selectors";
import { useSharedStatesSelector } from "./use-shared-states-selector";

export const useMarketInfos = () => useSharedStatesSelector(selectMarketInfos);

export * from "./use-index-token-stats";
export * from "./use-shared-states-selector";
export * from "./use-market-state-selector";
