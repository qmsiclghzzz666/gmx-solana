import { AdvancedRealTimeChart } from "react-ts-tradingview-widgets";

export function TVChartContainer({ symbol }: { symbol: string }) {
  return (
    <div className="ExchangeChart-container">
      <AdvancedRealTimeChart theme="dark" autosize symbol={`PYTH:${symbol}USD`} />
    </div>
  );
}
