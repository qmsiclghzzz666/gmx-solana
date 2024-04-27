import { Trans } from "@lingui/macro";
import cx from "classnames";

import "./NetFeeTooltip.scss";
import { MarketStat } from "@/contexts/shared";
import { formatRatePercentage, getMarketIndexName, getMarketPoolName } from "./utils";
import { BN } from "@coral-xyz/anchor";

const zero = new BN(0);

export function NetFeeTooltip({ marketStats }: { marketStats: MarketStat[] }) {
  return (
    <table className="NetFeeTooltip">
      <thead className="NetFeeTooltip-head">
        <tr>
          <th>
            <Trans>Pool</Trans>
          </th>
          <th className="NetFeeTooltip-cell-center">
            <Trans>Longs Net Fee / 1h</Trans>
          </th>
          <th className="NetFeeTooltip-cell-right">
            <Trans>Shorts Net Fee / 1h</Trans>
          </th>
        </tr>
      </thead>
      <tbody>
        {marketStats.map((stat) => {
          const { marketInfo: market, netFeeLong, netFeeShort } = stat;

          return (
            <tr key={market.marketTokenAddress.toBase58()}>
              <td>
                <div className="items-top flex-wrap text-white">
                  <span>{getMarketIndexName(market)}</span>
                  <span className="subtext lh-1">[{getMarketPoolName(market)}]</span>
                </div>
              </td>
              <td
                className={cx("NetFeeTooltip-cell-center", {
                  "text-green": netFeeLong.gt(zero),
                  "text-red": netFeeLong.lt(zero),
                })}
              >
                {formatRatePercentage(netFeeLong)}
              </td>
              <td
                className={cx("NetFeeTooltip-cell-right", {
                  "text-green": netFeeShort.gt(zero),
                  "text-red": netFeeShort.lt(zero),
                })}
              >
                {formatRatePercentage(netFeeShort)}
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
