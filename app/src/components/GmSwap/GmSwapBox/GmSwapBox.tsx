import "./GmSwapBox.scss";
import { Market, MarketInfo, MarketInfos } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { Mode, Operation } from "./utils";
import { getByKey } from "@/utils/objects";
import Inner from "./Inner";
import { getTokenData } from "@/onchain/token/utils";

type Props = {
  selectedMarketAddress?: string;
  markets: Market[];
  marketsInfoData?: MarketInfos;
  tokensData?: Tokens;
  onSelectMarket: (marketAddress: string) => void;
  operation: Operation;
  mode: Mode;
  setMode: (mode: Mode) => void;
  setOperation: (operation: Operation) => void;
};

const getTokenOptions = (marketInfo?: MarketInfo) => {
  if (!marketInfo) {
    return [];
  }

  const { longToken, shortToken } = marketInfo;

  if (!longToken || !shortToken) return [];

  const options = [longToken];

  if (!marketInfo.isSingle) {
    options.push(shortToken);
  }

  return options;
};

export function GmSwapBox({
  operation,
  mode,
  setMode,
  setOperation,
  onSelectMarket,
  marketsInfoData,
  tokensData,
  selectedMarketAddress: marketAddress,
}: Props) {
  const marketInfo = getByKey(marketsInfoData, marketAddress);
  const tokenOptions = getTokenOptions(marketInfo);
  return (
    <Inner
      operation={operation}
      mode={mode}
      setOperation={setOperation}
      setMode={setMode}
      marketInfo={marketInfo}
      tokensData={tokensData}
      tokenOptions={tokenOptions}
      onSelectMarket={onSelectMarket}
    />
  );
}

// function showMarketToast(market: MarketInfo) {
//   if (!market) return;
//   const indexName = getMarketIndexName(market);
//   const poolName = getMarketPoolName(market);
//   helperToast.success(
//     <Trans>
//       <div className="inline-flex">
//         GM:&nbsp;<span>{indexName}</span>
//         <span className="subtext gm-toast">[{poolName}]</span>
//       </div>{" "}
//       <span>selected in order form</span>
//     </Trans>
//   );
// }
