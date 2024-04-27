import { useSharedStatesSelector } from "./use-shared-states-selector";

export function useChainId() {
  return useSharedStatesSelector(s => s.chainId);
}
