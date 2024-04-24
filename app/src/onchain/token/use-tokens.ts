import { useMemo, useRef } from "react";
import { Token, TokenBalances, TokenMetadatas, Tokens } from "./types";
import { PriceProvider, usePriceFromFeeds } from "./use-price-from-feeds";
import { PublicKey } from "@solana/web3.js";
import useSWR from "swr";
import { useConnection } from "@solana/wallet-adapter-react";
import { TokenAccountNotFoundError, getAccount, getAssociatedTokenAddressSync, getMint } from "@solana/spl-token";
import { toBN } from "gmsol";
import { useAnchorProvider } from "@/contexts/anchor";

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

export const useTokenBalances = (tokens: PublicKey[]) => {
  const provider = useAnchorProvider();
  const cache = useRef<TokenBalances>({});

  const owner = provider?.publicKey;
  const request = useMemo(() => {
    return {
      key: "token-balances",
      tokens,
      owner,
    }
  }, [tokens, owner]);

  const { data, isLoading } = useSWR(request, async ({ tokens, owner }) => {
    const tokenBalances: TokenBalances = {};

    if (owner && provider) {
      for (const address of tokens) {
        const accountAddress = getAssociatedTokenAddressSync(address, owner);
        try {
          const account = await getAccount(provider.connection, accountAddress);
          tokenBalances[address.toBase58()] = toBN(account.amount);
        } catch (error) {
          if ((error as TokenAccountNotFoundError).name === "TokenAccountNotFoundError") {
            tokenBalances[address.toBase58()] = null;
          } else {
            console.error("fetch account balance error", error);
          }
        }
      }
    }
    return tokenBalances;
  });

  return useMemo(() => {
    if (!isLoading && data) {
      cache.current = data;
    }
    return cache.current;
  }, [data, isLoading]);
};
