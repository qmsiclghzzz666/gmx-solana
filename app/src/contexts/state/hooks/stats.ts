import { MarketInfo } from "@/onchain/market";
import { useStateSelector } from "./utils";
import { getUnit } from "@/utils/number";
import { BN } from "@coral-xyz/anchor";
import { IndexTokenStat, MarketStat } from "../types";
import { USD_DECIMALS } from "@/config/constants";

const info2Stat = (info: MarketInfo) => {
  const longUnit = getUnit(info.longToken.decimals);
  const shortUnit = getUnit(info.shortToken.decimals);
  const longUnitPrice = info.longToken.prices.minPrice.div(longUnit);
  const shortUnitPrice = info.shortToken.prices.minPrice.div(shortUnit);
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
    utilization: usedLiquidity.div(maxLiquidity),
  };
  return stat;
};

export const useIndexTokenStats = () => useStateSelector(state => {
  const infos = state.marketInfos;
  const stats: { [indexAddress: string]: IndexTokenStat } = {};
  for (const key in infos) {
    const info = infos[key];
    const stat = info2Stat(info);
    const indexKey = info.indexTokenAddress.toBase58();
    const indexStat = stats[indexKey] ?? {
      token: info.indexToken,
      price: info.indexToken.prices.minPrice,
      totalPoolValue: new BN(0),
      totalUtilization: new BN(0),
      totalUsedLiquidity: new BN(0),
      totalMaxLiquidity: new BN(0),
      bestNetFeeLong: getUnit(USD_DECIMALS).neg(),
      bestNetFeeShort: getUnit(USD_DECIMALS).neg(),
      marketsStats: [],
    };
    indexStat.totalPoolValue = indexStat.totalPoolValue.add(stat.poolValueUsd);
    indexStat.totalUsedLiquidity = indexStat.totalUsedLiquidity.add(stat.usedLiquidity);
    indexStat.totalMaxLiquidity = indexStat.totalMaxLiquidity.add(stat.maxLiquidity);
    indexStat.totalUtilization = indexStat.totalUsedLiquidity.div(indexStat.totalMaxLiquidity);
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
