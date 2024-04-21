import "./Earn.css";
import PageTitle from "@/components/PageTitle/PageTitle";
import { Trans, t } from "@lingui/macro";
import ExternalLink from "@/components/ExternalLink/ExternalLink";

export default function Earn() {
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

      {/* <div className="MarketPoolsPage-content">
        <MarketStats
          marketsTokensAPRData={marketsTokensAPRData}
          marketsTokensIncentiveAprData={marketsTokensIncentiveAprData}
          marketTokensData={depositMarketTokensData}
          marketsInfoData={marketsInfoData}
          marketInfo={marketInfo}
          marketToken={marketToken}
        />

        <div className="MarketPoolsPage-swap-box" ref={gmSwapBoxRef}>
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
        </div>
      </div> */}
    </div>
  )
}
