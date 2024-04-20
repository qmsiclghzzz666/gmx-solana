import { useMemo } from "react";
import { Token, Tokens } from "./types";
import { PriceProvider, usePriceFromFeeds } from "./use-price-from-feeds";
import { PublicKey } from "@solana/web3.js";

export interface TokenMap {
  [address: string]: Token,
}

export const useTokens = ({
  provider = "pyth",
  tokens,
}: {
  provider?: PriceProvider,
  tokens: TokenMap,
}) => {
  const feeds = useMemo(() => {
    return Object
      .keys(tokens)
      .map(address => tokens[address].feedAddress)
      .filter(address => !(address === undefined)) as PublicKey[];
  }, [tokens]);
  const prices = usePriceFromFeeds({
    provider,
    feeds,
  });

  return useMemo(() => {
    const tokenDatas: Tokens = {};
    for (const address in tokens) {
      const token = tokens[address];
      if (token.feedAddress) {
        const tokenPrices = prices[token.feedAddress.toBase58()];
        if (tokenPrices) {
          tokenDatas[address] = {
            ...token,
            prices: tokenPrices,
          };
        }
      }
    }
    return tokenDatas;
  }, [tokens, prices]);
};
