import { GMSOL_DEPLOYMENT } from "@/config/deployment";
import { useDeployedMarkets } from "./use-deployed-markets";
import { useMemo } from "react";
import { TokenMap, useTokens } from "../token";
import { MarketInfos } from "../market";

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

  const tokens = useTokens({ tokens: tokenMap });

  return useMemo(() => {
    const infos: MarketInfos = {};
    for (const key in markets) {
      const market = markets[key];
      const indexToken = tokens[market.indexTokenAddress.toBase58()];
      const longToken = tokens[market.longTokenAddress.toBase58()];
      const shortToken = tokens[market.shortTokenAddress.toBase58()];

      if (indexToken && longToken && shortToken) {
        infos[key] = {
          ...market,
          indexToken,
          longToken,
          shortToken,
        };
      }
    }
    return infos;
  }, [markets, tokens]);
};
