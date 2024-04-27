import "./Earn.css";
import PageTitle from "@/components/PageTitle/PageTitle";
import { Trans, t } from "@lingui/macro";
import ExternalLink from "@/components/ExternalLink/ExternalLink";
import { GmList } from "@/components/GmList/GmList";
import { useStateSelector } from "@/contexts/shared";
import { MarketStats } from "@/components/MarketStats/MarketStats";
import { getByKey } from "@/utils/objects";
import { getTokenData } from "@/onchain/token/utils";
import { useLoaderData, useSearchParams } from "react-router-dom";
import { PublicKey } from "@solana/web3.js";
import { useCallback, useEffect, useRef } from "react";
import { GmSwapBox } from "@/components/GmSwap/GmSwapBox/GmSwapBox";
import { getGmSwapBoxAvailableModes } from "@/components/GmSwap/utils";
import { CreateDepositParams, CreateWithdrawalParams, Mode, Operation } from "@/components/GmSwap/types";
import { useGenesisHash } from "@/onchain/utils";
import { useAnchor, useExchange } from "@/contexts/anchor";
import { invokeCreateDeposit, invokeCreateWithdrawal } from "gmsol";
import { GMSOL_DEPLOYMENT } from "@/config/deployment";
import { useSWRConfig } from "swr";
import { filterBalances, filterMetadatas } from "@/onchain/token";
import { fitlerMarkets } from "@/onchain/market";
import { useTriggerInvocation } from "@/onchain/transaction";

export default function Earn() {
  const chainId = useGenesisHash();
  const exchange = useExchange();
  const { owner } = useAnchor();

  const gmSwapBoxRef = useRef<HTMLDivElement>(null);

  function buySellActionHandler() {
    gmSwapBoxRef?.current?.scrollIntoView();
    window.scrollBy(0, -25); // add some offset
  }

  const { marketInfos, tokens, marketTokens } = useStateSelector(state => {
    return {
      marketInfos: state.marketInfos,
      tokens: state.tokens,
      marketTokens: state.marketTokens,
    };
  });

  const { market, operation, mode } = useLoaderData() as {
    market: string | null,
    operation: Operation,
    mode: Mode,
  };

  const selectedMarketKey = market ?? Object.keys(marketInfos)[0];

  const marketInfo = getByKey(marketInfos, selectedMarketKey);

  const marketToken = getTokenData(
    marketTokens,
    selectedMarketKey ? new PublicKey(selectedMarketKey) : undefined,
  );

  const setSearchParams = useSearchParams()[1];

  const setSelectedMarketKey = useCallback((address: string) => {
    setSearchParams((params) => {
      params.set("market", address);
      return params;
    });
  }, [setSearchParams]);

  const setMode = useCallback((mode: Mode) => {
    setSearchParams((params) => {
      params.set("mode", mode.toLowerCase());
      return params;
    });
  }, [setSearchParams]);

  const setOperation = useCallback((operation: Operation) => {
    setSearchParams((params) => {
      params.set("operation", operation.toLowerCase());
      return params;
    });
  }, [setSearchParams]);

  // Repair mode if it is incorrect.
  useEffect(() => {
    const newAvailableModes = getGmSwapBoxAvailableModes(operation, getByKey(marketInfos, selectedMarketKey));

    if (!newAvailableModes.includes(mode)) {
      setMode(newAvailableModes[0]);
    }
  }, [marketInfos, mode, setMode, operation, selectedMarketKey]);

  const { mutate } = useSWRConfig();

  const mutateStates = useCallback(() => {
    void mutate(filterMetadatas);
    void mutate(filterBalances);
    void mutate(fitlerMarkets);
  }, [mutate]);

  const createDepositInvoker = useCallback(async (params: CreateDepositParams) => {
    const payer = exchange.provider.publicKey;
    if (payer && GMSOL_DEPLOYMENT) {
      const [signature, deposit] = await invokeCreateDeposit(exchange, {
        store: GMSOL_DEPLOYMENT?.store,
        payer,
        ...params,
      });
      console.log(`created a deposit ${deposit.toBase58()} at tx ${signature}`);
      return signature;
    } else {
      throw Error("Wallet is not connected");
    }
  }, [exchange]);

  const { trigger: triggerCreateDeposit, isSending: isCreatingDeposit } = useTriggerInvocation({
    key: "exchange-create-deposit",
    onSentMessage: t`Creating deposit...`,
    message: t`Deposit created.`,
  }, createDepositInvoker, {
    onSuccess: mutateStates
  });

  const createWithdrawalInvoker = useCallback(async (params: CreateWithdrawalParams) => {
    const payer = exchange.provider.publicKey;
    if (payer && GMSOL_DEPLOYMENT) {
      const [signature, deposit] = await invokeCreateWithdrawal(exchange, {
        store: GMSOL_DEPLOYMENT.store,
        payer,
        ...params,
      });
      console.log(`created a withdrawal ${deposit.toBase58()} at tx ${signature}`);
      return signature;
    } else {
      throw Error("Wallet is not connected");
    }
  }, [exchange]);

  const { trigger: triggerCreateWithdrawal, isSending: isCreatingWithdrawal } = useTriggerInvocation({
    key: "exchange-create-deposit",
    onSentMessage: t`Creating withdrawal...`,
    message: t`Withdrawal created.`,
  }, createWithdrawalInvoker, { onSuccess: mutateStates });

  return (
    <div className="default-container page-layout">
      <PageTitle
        title={t`Earn`}
        isTop
        subtitle={
          <div>
            <Trans>
              Buy <ExternalLink href="#">GM</ExternalLink> to earn rewards.
            </Trans>
          </div>
        }
      />

      <div className="MarketPoolsPage-content">
        <MarketStats
          // marketsTokensAPRData={marketsTokensAPRData}
          // marketsTokensIncentiveAprData={marketsTokensIncentiveAprData}
          marketTokensData={marketTokens}
          marketsInfoData={marketInfos}
          marketInfo={marketInfo}
          marketToken={marketToken}
        />

        <div className="MarketPoolsPage-swap-box" ref={gmSwapBoxRef}>
          {chainId && marketInfo ? (<GmSwapBox
            owner={owner}
            chainId={chainId}
            marketInfo={marketInfo}
            tokensData={tokens}
            marketTokens={marketTokens}
            marketInfos={marketInfos}
            onSelectMarket={setSelectedMarketKey}
            operation={operation}
            isPending={
              (operation === Operation.Deposit && isCreatingDeposit)
              || (operation === Operation.Withdrawal && isCreatingWithdrawal)
            }
            mode={mode}
            setMode={setMode}
            setOperation={setOperation}
            onCreateDeposit={(params) => void triggerCreateDeposit(params)}
            onCreateWithdrawal={(params) => void triggerCreateWithdrawal(params)}
          />) : "loading"}
        </div>
      </div>

      <div className="Tab-title-section">
        <div className="Page-title">
          <Trans>Select a Market</Trans>
        </div>
      </div>
      <GmList
        // marketsTokensAPRData={marketsTokensAPRData}
        // marketsTokensIncentiveAprData={marketsTokensIncentiveAprData}
        marketTokensData={marketTokens}
        marketsInfoData={marketInfos}
        tokensData={tokens}
        buySellActionHandler={buySellActionHandler}
        shouldScrollToTop={true}
      />

    </div>
  )
}
