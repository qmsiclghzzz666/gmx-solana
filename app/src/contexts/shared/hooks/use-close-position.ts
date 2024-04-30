import { useSharedStatesSelector } from "./use-shared-states-selector";
import { selectSetClosingPositionAddress } from "../selectors/position-seller-selectors";
import { Address } from "@coral-xyz/anchor";
import { useCallback } from "react";

export const useClosePosition = () => {
  const setClosingPositionAddress = useSharedStatesSelector(selectSetClosingPositionAddress);
  return useCallback((address: Address) => setClosingPositionAddress(address), [setClosingPositionAddress]);
}
