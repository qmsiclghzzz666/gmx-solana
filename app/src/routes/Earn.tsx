import "./Earn.css";
import PageTitle from "@/components/PageTitle/PageTitle";
import { Trans, t } from "@lingui/macro";
import ExternalLink from "@/components/ExternalLink/ExternalLink";
import { GmList } from "@/components/GmList/GmList";
import { useStateSelector } from "@/contexts/state";
import { MarketStats } from "@/components/MarketStats/MarketStats";
import { getByKey } from "@/utils/objects";
import { getTokenData } from "@/onchain/token/utils";
import { useLoaderData } from "react-router-dom";
import { PublicKey } from "@solana/web3.js";

export default function Earn() {
  const { marketInfos, tokens, marketTokens } = useStateSelector(state => {
    return {
      marketInfos: state.marketInfos,
      tokens: state.tokens,
      marketTokens: state.marketTokens,
    };
  });

  const { market } = useLoaderData() as { market: string | null };

  const selectedMarketKey = market ?? Object.keys(marketInfos)[0];

  const marketInfo = getByKey(marketInfos, selectedMarketKey);

  const marketToken = getTokenData(
    marketTokens,
    selectedMarketKey ? new PublicKey(selectedMarketKey) : undefined,
  );

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

        {/* <div className="MarketPoolsPage-swap-box" ref={gmSwapBoxRef}>
          <GmSwapBox
            selectedMarketAddress={selectedMarketKey}
            markets={markets}
            marketsInfoData={marketsInfoData}
            tokensData={tokensData}
            onSelectMarket={setSelectedMarketKey}
            operation={operation}
            mode={mode}
            setMode={setMode}
            setOperation={setOperation}
          />
        </div> */}
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
        // buySellActionHandler={buySellActionHandler}
        shouldScrollToTop={true}
      />

    </div>
  )
}
