import "./GmSwapBox.scss";
import { MarketInfo, MarketInfos } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { Mode, Operation } from "../utils";
import { GmForm } from "./GmForm";
import { useTokenOptionsFromStorage } from "../hooks";
import GmStateProvider from "../GmStateProvider";

type Props = {
  chainId: string,
  marketInfo: MarketInfo,
  operation: Operation;
  mode: Mode;
  tokensData?: Tokens;
  marketTokens: Tokens,
  marketInfos: MarketInfos,
  setMode: (mode: Mode) => void;
  setOperation: (operation: Operation) => void;
  onSelectMarket: (marketAddress: string) => void;
};

export function GmSwapBox({
  chainId,
  marketInfo,
  operation,
  mode,
  tokensData,
  marketTokens,
  marketInfos,
  setMode,
  setOperation,
  onSelectMarket,
}: Props) {
  const [tokenOptions, setTokenAddress] = useTokenOptionsFromStorage({
    chainId,
    marketInfo,
    operation,
    mode,
    tokensData,
  });

  return (
    <GmStateProvider
      market={marketInfo}
      operation={operation}
      firstToken={tokenOptions.firstToken}
      secondToken={tokenOptions.secondToken}
      marketTokens={marketTokens}
      marketInfos={marketInfos}
    >
      <GmForm
        genesisHash={chainId}
        operation={operation}
        mode={mode}
        tokenOptions={tokenOptions}
        setOperation={setOperation}
        setMode={setMode}
        onSelectMarket={onSelectMarket}
        onSelectFirstToken={(token) => {
          setTokenAddress(token.address, "first");
        }}
      />
    </GmStateProvider>
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
