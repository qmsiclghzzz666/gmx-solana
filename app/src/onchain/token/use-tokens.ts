import { useMemo, useRef } from "react";
import { Token, TokenMetadatas, Tokens } from "./types";
import { PriceProvider, usePriceFromFeeds } from "./use-price-from-feeds";
import { PublicKey } from "@solana/web3.js";
import useSWR from "swr";
import { useConnection } from "@solana/wallet-adapter-react";
import { getMint } from "@solana/spl-token";
import { toBN } from "gmsol";

export interface TokenMap {
  [address: string]: Token,
}

export const useTokensWithPrices = ({
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

export const useTokenMetadatas = (tokens: PublicKey[]) => {
  const connection = useConnection();
  const cache = useRef<TokenMetadatas>({});

  const request = useMemo(() => {
    return {
      key: "token-metadatas",
      tokens,
    };
  }, [tokens]);

  const { data, isLoading } = useSWR(request, async ({ tokens }) => {
    const tokenDatas: TokenMetadatas = {};

    for (const address of tokens) {
      const mint = await getMint(connection.connection, address);
      tokenDatas[address.toBase58()] = {
        decimals: mint.decimals,
        totalSupply: toBN(mint.supply),
      };
    }

    return tokenDatas;
  });

  return useMemo(() => {
    if (!isLoading && data) {
      cache.current = data;
    }
    return cache.current;
  }, [data, isLoading]);
};
