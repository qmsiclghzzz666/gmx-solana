import "./TVChart.scss";

import { ChartTokenSelector } from "../ChartTokenSelector/ChartTokenSelector";
import { formatUsd } from "../MarketsList/utils";
import { useSharedStatesSelector } from "@/contexts/shared";
import { selectAvailableChartTokens, selectChartToken } from "@/contexts/shared/selectors/chart-selectors";
import { TVChartContainer } from "./TVChartContainer";

export function TVChart() {
  const chartToken = useSharedStatesSelector(selectChartToken);
  const availableTokens = useSharedStatesSelector(selectAvailableChartTokens);
  const tokenOptions = availableTokens;

  return (
    <div className="ExchangeChart tv">
      <div className="ExchangeChart-header">
        <div className="ExchangeChart-info">
          <div className="ExchangeChart-top-inner">
            <ChartTokenSelector selectedToken={chartToken} options={tokenOptions} />
            <div className="Chart-min-max-price">
              <div className="ExchangeChart-main-price">
                {formatUsd(chartToken?.prices?.maxPrice, {
                  displayDecimals: chartToken?.priceDecimals,
                }) || "..."}
              </div>
              <div className="ExchangeChart-info-label">
                {formatUsd(chartToken?.prices?.minPrice, {
                  displayDecimals: chartToken?.priceDecimals,
                }) || "..."}
              </div>
            </div>

            {/* <div className="Chart-24h-change">
              <div className="ExchangeChart-info-label">24h Change</div>
              <div
                className={cx({
                  positive: dayPriceDelta?.deltaPercentage && dayPriceDelta?.deltaPercentage > 0,
                  negative: dayPriceDelta?.deltaPercentage && dayPriceDelta?.deltaPercentage < 0,
                })}
              >
                {dayPriceDelta?.deltaPercentageStr || "-"}
              </div>
            </div>
            <div className="ExchangeChart-additional-info">
              <div className="ExchangeChart-info-label">24h High</div>
              <div>
                {dayPriceDelta?.high
                  ? numberWithCommas(dayPriceDelta.high.toFixed(chartToken?.priceDecimals || 2))
                  : "-"}
              </div>
            </div>
            <div className="ExchangeChart-additional-info Chart-24h-low">
              <div className="ExchangeChart-info-label">24h Low</div>
              <div>
                {dayPriceDelta?.low
                  ? numberWithCommas(dayPriceDelta?.low.toFixed(chartToken?.priceDecimals || 2))
                  : "-"}
              </div>
            </div> */}
          </div>
        </div>
        {/* <div className="ExchangeChart-info VersionSwitch-wrapper">
          <VersionSwitch />
        </div> */}
      </div>
      <div className="ExchangeChart-bottom App-box App-box-border">
        {chartToken && (
          <TVChartContainer
            symbol={chartToken.symbol}
          // chartLines={chartLines}
          // symbol={chartToken.symbol}
          // chainId={chainId}
          // onSelectToken={onSelectChartToken}
          // dataProvider={dataProvider}
          // period={period}
          // setPeriod={setPeriod}
          // chartToken={chartTokenProp}
          // supportedResolutions={SUPPORTED_RESOLUTIONS_V2}
          />
        )}
      </div>
    </div>
  );
}
