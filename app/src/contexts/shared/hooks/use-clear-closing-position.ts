import { useCallback } from "react";
import { selectSetClosingPositionAddress } from "../selectors/position-seller-selectors";
import { useSharedStatesSelector } from "./use-shared-states-selector";

export const useClearClosingPosition = () => {
  const setClosingPositionAddress = useSharedStatesSelector(selectSetClosingPositionAddress);
  return useCallback(() => setClosingPositionAddress(null), [setClosingPositionAddress]);
};
