import "./GmSwapBox.scss";
import { MarketInfo, MarketInfos } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { CreateDepositParams, CreateWithdrawalParams, Mode, Operation } from "../types";
import { GmForm } from "./GmForm";
import { useTokenOptionsFromStorage } from "../hooks";
import GmStateProvider from "../GmStateProvider";
import { getTokenData } from "@/onchain/token/utils";
import { NATIVE_TOKEN_ADDRESS } from "@/config/tokens";

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
  onCreateDeposit: (params: CreateDepositParams) => void;
  onCreateWithdrawal: (params: CreateWithdrawalParams) => void;
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
  onCreateDeposit,
  onCreateWithdrawal,
}: Props) {
  const [tokenOptions, setTokenAddress] = useTokenOptionsFromStorage({
    chainId,
    marketInfo,
    operation,
    mode,
    tokensData,
  });

  const nativeToken = getTokenData(tokensData, NATIVE_TOKEN_ADDRESS);

  return (
    <GmStateProvider
      market={marketInfo}
      operation={operation}
      mode={mode}
      firstToken={tokenOptions.firstToken}
      secondToken={tokenOptions.secondToken}
      nativeToken={nativeToken}
      marketTokens={marketTokens}
      marketInfos={marketInfos}
    >
      <GmForm
        genesisHash={chainId}
        tokenOptions={tokenOptions}
        setOperation={setOperation}
        setMode={setMode}
        onSelectMarket={onSelectMarket}
        onSelectFirstToken={(token) => {
          setTokenAddress(token.address, "first");
        }}
        onCreateDeposit={onCreateDeposit}
        onCreateWithdrawal={onCreateWithdrawal}
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
