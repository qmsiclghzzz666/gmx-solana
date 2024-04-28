import "./GmSwapBox.scss";
import { MarketInfo, MarketInfos } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { CreateDepositParams, CreateWithdrawalParams, Mode, Operation } from "../types";
import { GmForm } from "./GmForm";
import { useTokenOptionsFromStorage } from "../hooks";
import GmStateProvider from "../GmStateProvider";
import { getTokenData } from "@/onchain/token/utils";
import { NATIVE_TOKEN_ADDRESS } from "@/config/tokens";
import { PublicKey } from "@solana/web3.js";

type Props = {
  owner: PublicKey | undefined,
  chainId: string,
  marketInfo: MarketInfo,
  operation: Operation;
  mode: Mode;
  tokensData?: Tokens;
  marketTokens: Tokens,
  marketInfos: MarketInfos,
  isPending: boolean,
  setMode: (mode: Mode) => void;
  setOperation: (operation: Operation) => void;
  onSelectMarket: (marketAddress: string) => void;
  onCreateDeposit: (params: CreateDepositParams) => void;
  onCreateWithdrawal: (params: CreateWithdrawalParams) => void;
};

export function GmSwapBox({
  owner,
  chainId,
  marketInfo,
  operation,
  mode,
  tokensData,
  marketTokens,
  marketInfos,
  isPending,
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
        owner={owner}
        tokensData={tokensData}
        genesisHash={chainId}
        tokenOptions={tokenOptions}
        isPending={isPending}
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
