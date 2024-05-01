import { Address, translateAddress } from "@coral-xyz/anchor";
import { useTokensData } from "./use-tokens-data";
import { useCallback, useMemo } from "react";
import { useInitializeTokenAccounts } from "@/onchain/token";

export function useNeedToInitializeTokenAccounts(filter?: Address[]) {
  const translatedFilter = useMemo(() => new Set(filter?.map(address => address.toString())), [filter]);
  const tokens = useTokensData();
  const { needToInitializeTokenAddresses, needToInitializeTokens } = useMemo(() => {
    const needToInitializeTokenAddresses = Object.keys(tokens).filter(address => {
      if (!translatedFilter) return true;
      return translatedFilter.has(address) && tokens[address].balance === null;
    });
    const needToInitializeTokens = needToInitializeTokenAddresses.map(address => tokens[address]);
    return { needToInitializeTokenAddresses, needToInitializeTokens }
  }, [tokens, translatedFilter]);
  const { trigger, isSending } = useInitializeTokenAccounts();
  const initialize = useCallback(async () => {
    return await trigger(needToInitializeTokenAddresses.map(address => translateAddress(address)));
  }, [needToInitializeTokenAddresses, trigger]);
  return { needToInitialize: needToInitializeTokens, isSending, initialize };
}
