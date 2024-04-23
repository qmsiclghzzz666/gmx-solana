import "./Earn.css";
import PageTitle from "@/components/PageTitle/PageTitle";
import { Trans, t } from "@lingui/macro";
import ExternalLink from "@/components/ExternalLink/ExternalLink";
import { GmList } from "@/components/GmList/GmList";
import { useStateSelector } from "@/contexts/state";
import { MarketStats } from "@/components/MarketStats/MarketStats";
import { getByKey } from "@/utils/objects";
import { getTokenData } from "@/onchain/token/utils";
import { useLoaderData, useSearchParams } from "react-router-dom";
import { PublicKey } from "@solana/web3.js";
import { useCallback, useEffect, useRef } from "react";
import { GmSwapBox } from "@/components/GmSwap/GmSwapBox/GmSwapBox";
import { CreateDepositParams, Mode, Operation, getGmSwapBoxAvailableModes } from "@/components/GmSwap/utils";
import { useGenesisHash } from "@/onchain";
import { useExchange } from "@/contexts/anchor";
import { MakeCreateDepositParams, invokeCreateDeposit } from "gmsol";
import { GMSOL_DEPLOYMENT } from "@/config/deployment";
import useSWRMutation from "swr/mutation";

export default function Earn() {
  const chainId = useGenesisHash();
  const exchange = useExchange();
  const payer = exchange.provider.publicKey;

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

  const { trigger: triggerCreateDeposit } = useSWRMutation('exchange/create-deposit', async (_key, { arg }: { arg: MakeCreateDepositParams }) => {
    try {
      const [signature, deposit] = await invokeCreateDeposit(exchange, arg, { signByProvider: true });
      console.log(`created a deposit ${deposit.toBase58()} at tx ${signature}`);
    } catch (error) {
      console.log(error);
      throw error;
    }
  });

  const handleCreateDeposit = useCallback((params: CreateDepositParams) => {
    if (payer && GMSOL_DEPLOYMENT) {
      const store = GMSOL_DEPLOYMENT.store;
      const fullParams: MakeCreateDepositParams = {
        store,
        payer,
        marketToken: params.marketToken,
        initialLongToken: params.initialLongToken,
        initialShortToken: params.initialShortToken,
        initialLongTokenAmount: params.initialLongTokenAmount,
        initialShortTokenAmount: params.initialShortTokenAmount,
      };
      void triggerCreateDeposit(fullParams);
    } else {
      console.log("not connected");
    }
  }, [payer, triggerCreateDeposit]);

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
            chainId={chainId}
            marketInfo={marketInfo}
            tokensData={tokens}
            marketTokens={marketTokens}
            marketInfos={marketInfos}
            onSelectMarket={setSelectedMarketKey}
            operation={operation}
            mode={mode}
            setMode={setMode}
            setOperation={setOperation}
            onCreateDeposit={handleCreateDeposit}
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
