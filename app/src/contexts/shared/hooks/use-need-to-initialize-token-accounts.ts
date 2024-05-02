import { Address, translateAddress } from "@coral-xyz/anchor";
import { useTokensData } from "./use-tokens-data";
import { useCallback, useMemo } from "react";
import { useInitializeTokenAccounts } from "@/onchain/token";
import { ConfirmOptions } from "@solana/web3.js";
import { useMarketTokens } from "./use-market-tokens";
import { useMarketInfos } from "./use-market-infos";

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

  const marketTokens = useMarketTokens();
  const marketInfos = useMarketInfos();
  const { needToInitializeMarketTokenAddresses, needToInitializeMarketTokens } = useMemo(() => {
    const needToInitializeMarketTokenAddresses = Object.keys(marketTokens).filter(address => {
      if (!translatedFilter) return true;
      return translatedFilter.has(address) && marketTokens[address].balance === null;
    });
    const needToInitializeMarketTokens = needToInitializeMarketTokenAddresses.map(address => marketInfos[address]);
    return { needToInitializeMarketTokenAddresses, needToInitializeMarketTokens }
  }, [marketInfos, marketTokens, translatedFilter]);

  const { trigger, isSending } = useInitializeTokenAccounts(opts);
  const initialize = useCallback(async () => {
    return await trigger(needToInitializeTokenAddresses.concat(needToInitializeMarketTokenAddresses).map(address => translateAddress(address)));
  }, [needToInitializeMarketTokenAddresses, needToInitializeTokenAddresses, trigger]);
  const needToInitialize = (needToInitializeTokens.length + needToInitializeMarketTokens.length) > 0;
  return { needToInitialize, needToInitializeTokens, needToInitializeMarketTokens, isSending, initialize };
}
