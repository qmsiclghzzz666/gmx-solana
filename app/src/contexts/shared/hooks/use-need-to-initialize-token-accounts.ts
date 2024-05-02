import { Address, translateAddress } from "@coral-xyz/anchor";
import { useTokensData } from "./use-tokens-data";
import { useCallback, useMemo } from "react";
import { useInitializeTokenAccounts } from "@/onchain/token";
import { ConfirmOptions } from "@solana/web3.js";

export function useNeedToInitializeTokenAccounts(filter?: Address[], opts?: ConfirmOptions) {
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
  const { trigger, isSending } = useInitializeTokenAccounts(opts);
  const initialize = useCallback(async () => {
    return await trigger(needToInitializeTokenAddresses.map(address => translateAddress(address)));
  }, [needToInitializeTokenAddresses, trigger]);
  return { needToInitialize: needToInitializeTokens, isSending, initialize };
}
