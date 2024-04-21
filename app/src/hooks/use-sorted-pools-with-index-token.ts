import { MarketInfo, MarketInfos } from "@/onchain/market";
import { TokenData, Tokens } from "@/onchain/token";
import { getByKey } from "@/utils/objects";
import { groupBy } from "lodash";
import { useMemo } from "react";
import { toBN } from "gmsol";
import { convertToUsd } from "@/utils/number";
import { BN_ZERO } from "@/config/constants";

export function useSortedPoolsWithIndexToken(marketsInfoData?: MarketInfos, marketTokensData?: Tokens) {
  return useMemo(() => {
    if (!marketsInfoData || !marketTokensData) {
      return {
        markets: [],
        marketsInfo: [],
      };
    }
    // Group markets by index token address
    const groupedMarketList: { [marketAddress: string]: MarketInfo[] } = groupBy(
      Object.values(marketsInfoData),
      (market) => market[market.isSpotOnly ? "marketTokenAddress" : "indexTokenAddress"]
    );

    const allMarkets = Object.values(groupedMarketList)
      .map((markets) => {
        return markets
          .filter((market) => {
            const marketInfoData = getByKey(marketsInfoData, market.marketTokenAddress.toBase58())!;
            return !marketInfoData.isDisabled;
          })
          .map((market) => getByKey(marketTokensData, market.marketTokenAddress.toBase58())!);
      })
      .filter((markets) => markets.length > 0);

    const sortedGroups = allMarkets.sort((a, b) => {
      const totalMarketSupplyA = a.reduce((acc, market) => {
        const totalSupplyUsd = convertToUsd(market?.totalSupply, market?.decimals, market?.prices.minPrice);
        acc = acc.add(totalSupplyUsd || BN_ZERO);
        return acc;
      }, toBN(0));

      const totalMarketSupplyB = b.reduce((acc, market) => {
        const totalSupplyUsd = convertToUsd(market?.totalSupply, market?.decimals, market?.prices.minPrice);
        acc = acc.add(totalSupplyUsd || BN_ZERO);
        return acc;
      }, toBN(0));

      return totalMarketSupplyA.gt(totalMarketSupplyB) ? -1 : 1;
    });

    // Sort markets within each group by total supply
    const sortedMarkets = sortedGroups.map((markets) => {
      return markets.sort((a, b) => {
        const totalSupplyUsdA = convertToUsd(a.totalSupply, a.decimals, a.prices.minPrice)!;
        const totalSupplyUsdB = convertToUsd(b.totalSupply, b.decimals, b.prices.minPrice)!;
        return totalSupplyUsdA.gt(totalSupplyUsdB) ? -1 : 1;
      });
    });

    // Flatten the sorted markets array
    const flattenedMarkets = sortedMarkets.flat(Infinity).filter(Boolean) as TokenData[];
    return {
      markets: flattenedMarkets,
      marketsInfo: flattenedMarkets.map((market) => getByKey(marketsInfoData, market.address.toBase58())!),
    };
  }, [marketsInfoData, marketTokensData]);
}
