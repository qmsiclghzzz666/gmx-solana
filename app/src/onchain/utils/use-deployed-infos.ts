import { GMSOL_DEPLOYMENT } from "@/config/deployment";
import { useDeployedMarkets } from "../market/use-deployed-markets";
import { useMemo } from "react";
import { TokenMap, Tokens, convertToTokenAmount, useTokenBalances, useTokenMetadatas, useTokensWithPrices } from "../token";
import { MarketInfos, getPoolUsdWithoutPnl } from "../market";
import { getMarketIndexName, getMarketPoolName } from "@/components/MarketsList/utils";
import { info2Stat } from "@/contexts/shared";
import { BN_ZERO, ONE_USD } from "@/config/constants";
import { convertToUsd, getBasisPoints, getUnit } from "@/utils/number";
import { NATIVE_TOKEN_ADDRESS } from "@/config/tokens";
import { PositionInfos, getEntryPrice, getLeverage, getPositionNetValue, getPositionPnlUsd, usePositions } from "../position";
import { getMarkPrice } from "@/utils/price";
import { getByKey } from "@/utils/objects";

export const useDeployedInfos = () => {
  const { markets, isLoading: isMarketLoading } = useDeployedMarkets();

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
    const nativeKey = NATIVE_TOKEN_ADDRESS.toBase58();
    if (!(nativeKey in tokenMap)) {
      const config = GMSOL_DEPLOYMENT?.tokens[nativeKey];
      if (config) {
        tokenMap[nativeKey] = config;
      }
    }
    return tokenMap;
  }, [markets]);

  const tokens = useTokensWithPrices({ tokens: tokenMap });
  const tokenBalances = useTokenBalances(Object.keys(tokenMap));

  const marketTokenAddresses = useMemo(() => {
    return Object.values(markets).map(market => market.marketTokenAddress);
  }, [markets]);

  const { tokenMetadatas: marketTokenMetadatas, isLoading: isMarketTokenLoading } = useTokenMetadatas(marketTokenAddresses);
  const marketTokenBalances = useTokenBalances(marketTokenAddresses);

  const { positions, isLoading: isPositionsLoading } = usePositions(GMSOL_DEPLOYMENT ? { store: GMSOL_DEPLOYMENT.store, markets: Object.values(markets) } : undefined);

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
        const info = {
          ...market,
          name: `${indexName}[${poolName}]`,
          indexToken,
          longToken,
          shortToken,
        };

        infos[key] = {
          ...info,
          poolValueMax: getPoolUsdWithoutPnl(info, true, "maxPrice").add(getPoolUsdWithoutPnl(info, false, "maxPrice")),
        };

        const marketToken = marketTokenMetadatas ? marketTokenMetadatas[key] : undefined;

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
            },
            balance: marketTokenBalances[key],
          }
        }
      }
    }

    for (const key in tokens) {
      tokens[key].balance = tokenBalances[key];
    }

    const positionInfos: PositionInfos = {};
    let isPositionInfosLoading = false;
    for (const key in positions) {
      const position = positions[key];
      if (position.sizeInUsd.isZero()) continue;
      const info = infos[position.marketTokenAddress.toBase58()];
      const collateralToken = getByKey(tokens, position.collateralTokenAddress.toBase58());
      const collateralMinPrice = collateralToken?.prices.minPrice;
      if (info && collateralToken) {
        const collateralUsd = convertToUsd(position.collateralAmount, collateralToken?.decimals, collateralMinPrice);
        const remainingCollateralUsd = collateralUsd;
        const remainingCollateralAmount = convertToTokenAmount(
          remainingCollateralUsd,
          collateralToken.decimals,
          collateralMinPrice
        )!;
        const markPrice = getMarkPrice({ prices: info.indexToken.prices, isLong: position.isLong, isIncrease: false });
        const pnl = getPositionPnlUsd({
          marketInfo: info,
          sizeInUsd: position.sizeInUsd,
          sizeInTokens: position.sizeInTokens,
          markPrice,
          isLong: position.isLong,
        });
        const pnlPercentage =
          collateralUsd && !collateralUsd.eq(BN_ZERO) ? getBasisPoints(pnl, collateralUsd) : BN_ZERO;
        const leverage = getLeverage({
          sizeInUsd: position.sizeInUsd,
          collateralUsd: collateralUsd ?? BN_ZERO,
          // pnl: showPnlInLeverage ? pnl : undefined,
          pnl,
          pendingBorrowingFeesUsd: BN_ZERO,
          pendingFundingFeesUsd: BN_ZERO,
        });
        positionInfos[key] = {
          ...position,
          marketInfo: info,
          collateralToken,
          markPrice,
          entryPrice: getEntryPrice({ sizeInTokens: position.sizeInTokens, sizeInUsd: position.sizeInUsd, indexToken: info.indexToken }),
          remainingCollateralUsd,
          remainingCollateralAmount,
          netValue: getPositionNetValue({
            collateralUsd: collateralUsd ?? BN_ZERO,
            pnl,
            pendingBorrowingFeesUsd: BN_ZERO,
            pendingFundingFeesUsd: BN_ZERO,
            closingFeeUsd: BN_ZERO,
            uiFeeUsd: BN_ZERO,
          }),
          leverage,
          pnl,
          pnlPercentage,
        };
      } else {
        isPositionInfosLoading = true;
      }
    }

    return {
      marketInfos: infos,
      tokens: tokens,
      marketTokens,
      positionInfos,
      isPositionsLoading: isPositionsLoading || isPositionInfosLoading,
      isMarketLoading,
      isMarketTokenLoading,
    };
  }, [tokens, isPositionsLoading, isMarketLoading, isMarketTokenLoading, markets, marketTokenMetadatas, marketTokenBalances, tokenBalances, positions]);
};
