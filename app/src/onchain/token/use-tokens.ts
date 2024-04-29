import { useMemo } from "react";
import { Token, TokenBalances, TokenMetadatas, Tokens } from "./types";
import { PriceProvider, usePriceFromFeeds } from "./use-price-from-feeds";
import { PublicKey } from "@solana/web3.js";
import useSWR from "swr";
import { useConnection } from "@solana/wallet-adapter-react";
import { TokenAccountNotFoundError, getAccount, getAssociatedTokenAddressSync, getMint } from "@solana/spl-token";
import { toBN } from "gmsol";
import { useAnchorProvider } from "@/contexts/anchor";
import { Address, translateAddress } from "@coral-xyz/anchor";
import { isObject } from "lodash";
import { NATIVE_TOKEN_ADDRESS } from "@/config/tokens";

export const BALANCE_KEY = "token-balanses";
export const METADATA_KEY = "token-metadatas";

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

  const request = useMemo(() => {
    return {
      key: METADATA_KEY,
      tokens: tokens.map(key => key.toBase58()),
    };
  }, [tokens]);

  const { data, isLoading } = useSWR(request, async ({ tokens }) => {
    const tokenDatas: TokenMetadatas = {};

    for (const addressStr of tokens) {
      const address = translateAddress(addressStr);
      const mint = await getMint(connection.connection, translateAddress(address));
      tokenDatas[address.toBase58()] = {
        decimals: mint.decimals,
        totalSupply: toBN(mint.supply),
      };
    }

    return tokenDatas;
  });

  return {
    tokenMetadatas: data ?? {},
    isLoading,
  };
};

export const useTokenBalances = (tokens: Address[]) => {
  const provider = useAnchorProvider();

  const owner = provider?.publicKey;
  const request = useMemo(() => {
    return {
      key: BALANCE_KEY,
      tokens: tokens.map(token => token.toString()),
      owner: owner?.toBase58(),
    }
  }, [tokens, owner]);

  const { data } = useSWR(request, async ({ tokens, owner }) => {
    const tokenBalances: TokenBalances = {};

    if (owner && provider) {
      const ownerAddress = translateAddress(owner);
      for (const address of tokens) {
        const tokenAddress = translateAddress(address);
        if (tokenAddress.equals(NATIVE_TOKEN_ADDRESS)) {
          try {
            const balance = await provider.connection.getBalance(ownerAddress);
            tokenBalances[tokenAddress.toBase58()] = toBN(balance);
          } catch (error) {
            console.error("fetch balance error", error);
          }
        } else {
          const accountAddress = getAssociatedTokenAddressSync(tokenAddress, ownerAddress);
          try {
            const account = await getAccount(provider.connection, accountAddress);
            tokenBalances[address.toString()] = toBN(account.amount);
          } catch (error) {
            if ((error as TokenAccountNotFoundError).name === "TokenAccountNotFoundError") {
              tokenBalances[address.toString()] = null;
            } else {
              console.error(`fetch account balance for ${tokenAddress.toBase58()} error`, error);
            }
          }
        }
      }
    }
    return tokenBalances;
  });

  return data ?? {};
};

export const filterBalances = (value: unknown) => {
  if (isObject(value)) {
    const { key } = value as { key?: string };
    if (key === BALANCE_KEY) {
      console.debug("filtered token balances");
      return true;
    }
  }
  return false;
};

export const filterMetadatas = (value: unknown) => {
  if (isObject(value)) {
    const { key } = value as { key?: string };
    if (key === METADATA_KEY) {
      console.debug("filtered token metadatas");
      return true;
    }
  }
  return false;
};
