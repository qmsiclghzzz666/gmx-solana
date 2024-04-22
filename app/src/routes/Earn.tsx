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
import { Mode, Operation, getGmSwapBoxAvailableModes } from "@/components/GmSwap/GmSwapBox/utils";

export default function Earn() {
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
          <GmSwapBox
            selectedMarketAddress={selectedMarketKey}
            markets={[]}
            marketsInfoData={marketInfos}
            tokensData={tokens}
            onSelectMarket={setSelectedMarketKey}
            operation={operation}
            mode={mode}
            setMode={setMode}
            setOperation={setOperation}
          />
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
