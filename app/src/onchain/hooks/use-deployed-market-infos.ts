import { GMSOL_DEPLOYMENT } from "@/config/deployment";
import { useDeployedMarkets } from "./use-deployed-markets";
import { useMemo } from "react";
import { TokenMap, Tokens, useTokenMetadatas, useTokensWithPrices } from "../token";
import { MarketInfos } from "../market";
import { getMarketIndexName, getMarketPoolName } from "@/components/MarketsList/utils";
import { info2Stat } from "@/contexts/state";
import { ONE_USD } from "@/config/constants";
import { getUnit } from "@/utils/number";

export const useDeployedMarketInfos = () => {
  const markets = useDeployedMarkets();

  const tokenMap = useMemo(() => {
    const tokenMap: TokenMap = {};
    for (const key in markets) {
      const market = markets[key];
      [market.indexTokenAddress, market.longTokenAddress, market.shortTokenAddress].forEach(address => {
        const key = address.toBase58();
        const config = GMSOL_DEPLOYMENT?.tokens[key];
        if (config) {
          tokenMap[key] = config;
        }
      });
    }
    return tokenMap;
  }, [markets]);

  const tokens = useTokensWithPrices({ tokens: tokenMap });

  const marketTokenAddresses = useMemo(() => {
    return Object.values(markets).map(market => market.marketTokenAddress);
  }, [markets]);

  const marketTokenMetadatas = useTokenMetadatas(marketTokenAddresses);

  return useMemo(() => {
    const infos: MarketInfos = {};
    const marketTokens: Tokens = {};
    for (const key in markets) {
      const market = markets[key];
      const indexToken = tokens[market.indexTokenAddress.toBase58()];
      const longToken = tokens[market.longTokenAddress.toBase58()];
      const shortToken = tokens[market.shortTokenAddress.toBase58()];

      if (indexToken && longToken && shortToken) {
        const indexName = getMarketIndexName({
          indexToken,
          isSpotOnly: market.isSpotOnly,
        });
        const poolName = getMarketPoolName({
          longToken, shortToken
        });
        infos[key] = {
          ...market,
          name: `${indexName}[${poolName}]`,
          indexToken,
          longToken,
          shortToken,
        };

        const marketToken = marketTokenMetadatas[key];

        if (marketToken) {
          const stat = info2Stat(infos[key]);
          const unit = getUnit(marketToken.decimals);
          const price = marketToken.totalSupply && !marketToken.totalSupply.isZero() ? stat.poolValueUsd.mul(unit).div(marketToken.totalSupply) : ONE_USD;
          marketTokens[key] = {
            symbol: `GM`,
            address: market.marketTokenAddress,
            ...marketToken,
            prices: {
              minPrice: price,
              maxPrice: price,
            }
          }
        }
      }
    }
    return {
      marketInfos: infos,
      tokens: tokens,
      marketTokens,
    };
  }, [markets, tokens, marketTokenMetadatas]);
};
