import { MarketInfo } from "@/onchain/market";
import { useMarketStateSelector } from "./utils";
import { getUnit } from "@/utils/number";
import { BN } from "@coral-xyz/anchor";
import { IndexTokenStat, MarketStat } from "../types";
import { BN_ZERO, MIN_SIGNED_USD, USD_DECIMALS } from "@/config/constants";
import { getMidPrice } from "@/onchain/token/utils";

export const info2Stat = (info: MarketInfo) => {
  const longUnit = getUnit(info.longToken.decimals);
  const shortUnit = getUnit(info.shortToken.decimals);
  const longUnitPrice = getMidPrice(info.longToken.prices).div(longUnit);
  const shortUnitPrice = getMidPrice(info.shortToken.prices).div(shortUnit);
  const poolValueUsd = info.longPoolAmount.mul(longUnitPrice).add(info.shortPoolAmount.mul(shortUnitPrice));
  const usedLiquidity = new BN(0);
  const maxLiquidity = poolValueUsd;
  const stat: MarketStat = {
    marketInfo: info,
    poolValueUsd,
    usedLiquidity,
    maxLiquidity,
    netFeeLong: getUnit(USD_DECIMALS - 2),
    netFeeShort: getUnit(USD_DECIMALS - 2).neg(),
    utilization: !maxLiquidity.isZero() ? usedLiquidity.div(maxLiquidity) : BN_ZERO,
  };
  return stat;
};

export const useIndexTokenStats = () => useMarketStateSelector(state => {
  const infos = state.marketInfos;
  const stats: { [indexAddress: string]: IndexTokenStat } = {};
  for (const key in infos) {
    const info = infos[key];
    const stat = info2Stat(info);
    const indexKey = info.indexTokenAddress.toBase58();
    const indexStat = stats[indexKey] ?? {
      token: info.indexToken,
      price: info.indexToken.prices.minPrice,
      totalPoolValue: BN_ZERO,
      totalUtilization: BN_ZERO,
      totalUsedLiquidity: BN_ZERO,
      totalMaxLiquidity: BN_ZERO,
      bestNetFeeLong: MIN_SIGNED_USD,
      bestNetFeeShort: MIN_SIGNED_USD,
      marketsStats: [],
    };
    indexStat.totalPoolValue = indexStat.totalPoolValue.add(stat.poolValueUsd);
    indexStat.totalUsedLiquidity = indexStat.totalUsedLiquidity.add(stat.usedLiquidity);
    indexStat.totalMaxLiquidity = indexStat.totalMaxLiquidity.add(stat.maxLiquidity);
    indexStat.totalUtilization = !indexStat.totalMaxLiquidity.isZero() ? indexStat.totalUsedLiquidity.div(indexStat.totalMaxLiquidity) : BN_ZERO;
    if (stat.netFeeLong.gt(indexStat.bestNetFeeLong)) {
      indexStat.bestNetFeeLong = stat.netFeeLong;
    }
    if (stat.netFeeShort.gt(indexStat.bestNetFeeShort)) {
      indexStat.bestNetFeeShort = stat.netFeeShort;
    }
    indexStat.marketsStats.push(stat);
    indexStat.marketsStats.sort((a, b) => b.poolValueUsd.cmp(a.poolValueUsd));
    stats[indexKey] = indexStat;
  }

  return Object.values(stats);
});
