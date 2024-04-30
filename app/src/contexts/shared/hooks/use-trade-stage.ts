import { selectStage } from "../selectors/trade-box-selectors";
import { useSharedStatesSelector } from "./use-shared-states-selector";

export const useTradeStage = () => useSharedStatesSelector(selectStage);
