import { selectSetStage } from "../selectors/trade-box-selectors";
import { useSharedStatesSelector } from "./use-shared-states-selector";

export const useSetTradeStage = () => useSharedStatesSelector(selectSetStage);
