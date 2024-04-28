import { selectMarketStateTokens } from "../selectors/market-selectors";
import { useSharedStatesSelector } from "./use-shared-states-selector";

export const useTokensData = () => useSharedStatesSelector(selectMarketStateTokens);
