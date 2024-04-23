import "./GmSwapBox.scss";
import { Market, MarketInfos } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { Mode, Operation } from "../utils";
import { getByKey } from "@/utils/objects";
import Inner from "./Inner";
import { useGenesisHash } from "@/onchain";

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
  const genesisHash = useGenesisHash();
  const marketInfo = getByKey(marketsInfoData, marketAddress);
  return (
    genesisHash && marketInfo ?
      <Inner
        genesisHash={genesisHash}
        operation={operation}
        mode={mode}
        setOperation={setOperation}
        setMode={setMode}
        marketInfo={marketInfo}
        tokensData={tokensData}
        onSelectMarket={onSelectMarket}
      />
      : <div>loading</div>
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
