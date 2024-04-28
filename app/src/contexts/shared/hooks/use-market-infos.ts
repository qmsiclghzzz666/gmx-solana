import { selectMarketStateMarketInfos } from "../selectors/market-selectors";
import { useSharedStatesSelector } from "./use-shared-states-selector";

export const useMarketInfos = () => useSharedStatesSelector(selectMarketStateMarketInfos);
