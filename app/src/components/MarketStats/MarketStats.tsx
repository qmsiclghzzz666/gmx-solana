import { Trans, t } from "@lingui/macro";
import "./MarketStats.scss";
import { MarketInfo, MarketInfos, MarketTokenAPRs } from "@/onchain/market";
import { TokenData, Tokens } from "@/onchain/token";
import { convertToUsd, formatTokenAmountWithUsd } from "@/utils/number";
import { formatUsd, getMarketIndexName, getMarketPoolName } from "../MarketsList/utils";
import Tooltip from "../Tooltip/Tooltip";
import { CardRow } from "../CardRow/CardRow";
import StatsTooltipRow from "../StatsTooltipRow/StatsTooltipRow";
import { BN_ZERO, GM_DECIMALS } from "@/config/constants";
import { getPoolUsdWithoutPnl, getSellableMarketToken } from "@/onchain/market/utils";
import { convertToTokenAmount, getMaxMintableUsd, getMidPrice } from "@/onchain/token/utils";
import MarketTokenSelector from "../MarketTokenSelector/MarketTokenSelector";
import { AprInfo } from "../AprInfo/AprInfo";

type Props = {
  marketsInfoData?: MarketInfos;
  marketTokensData?: Tokens;
  marketInfo?: MarketInfo;
  marketToken?: TokenData;
  marketsTokensAPRData?: MarketTokenAPRs;
  marketsTokensIncentiveAprData?: MarketTokenAPRs;
};

export function MarketStats(p: Props) {
  const {
    marketInfo,
    marketToken,
    marketsTokensAPRData,
    marketsInfoData,
    marketTokensData,
    marketsTokensIncentiveAprData,
  } = p;
  const marketPrice = marketToken ? getMidPrice(marketToken.prices) : undefined;
  const marketBalance = marketToken?.balance ?? BN_ZERO;
  const marketBalanceUsd = convertToUsd(marketBalance, marketToken?.decimals, marketPrice);

  const marketTotalSupply = marketToken?.totalSupply;
  const marketTotalSupplyUsd = convertToUsd(marketTotalSupply, marketToken?.decimals, marketPrice);

  const { longToken, shortToken, longPoolAmount, shortPoolAmount } = marketInfo || {};

  // const mintableInfo = marketInfo && marketToken ? getMintableMarketTokens(marketInfo, marketToken) : undefined;
  const sellableInfo = marketInfo && marketToken ? getSellableMarketToken(marketInfo, marketToken) : undefined;

  const maxLongSellableTokenAmount = convertToTokenAmount(
    sellableInfo?.maxLongSellableUsd,
    longToken?.decimals,
    longToken?.prices.minPrice
  );

  const maxShortSellableTokenAmount = convertToTokenAmount(
    sellableInfo?.maxShortSellableUsd,
    shortToken?.decimals,
    shortToken?.prices.minPrice
  );

  const longPoolAmountUsd = marketInfo ? getPoolUsdWithoutPnl(marketInfo, true, "midPrice") : undefined;
  const shortPoolAmountUsd = marketInfo ? getPoolUsdWithoutPnl(marketInfo, false, "midPrice") : undefined;

  // const apr = getByKey(marketsTokensAPRData, marketInfo?.marketTokenAddress);
  // const incentiveApr = getByKey(marketsTokensIncentiveAprData, marketInfo?.marketTokenAddress);
  // const isLpIncentiveActive = useIncentiveStats()?.lp?.isActive ?? false;
  const indexName = marketInfo && getMarketIndexName(marketInfo);
  const poolName = marketInfo && getMarketPoolName(marketInfo);

  // const bridgingOprionsForToken = getBridgingOptionsForToken(longToken?.symbol);
  // const shouldShowMoreInfo = Boolean(bridgingOprionsForToken);
  const shouldShowMoreInfo = false;

  // const maxLongTokenValue = useMemo(
  //   () => [
  //     formatTokenAmount(
  //       mintableInfo?.longDepositCapacityAmount,
  //       marketInfo?.longToken.decimals,
  //       marketInfo?.longToken.symbol,
  //       {
  //         useCommas: true,
  //       }
  //     ),
  //     `(${formatTokenAmount(marketInfo?.longPoolAmount, marketInfo?.longToken.decimals, undefined, {
  //       displayDecimals: 0,
  //       useCommas: true,
  //     })} / ${formatTokenAmount(
  //       marketInfo?.maxLongPoolAmount,
  //       marketInfo?.longToken.decimals,
  //       marketInfo?.longToken.symbol,
  //       { displayDecimals: 0, useCommas: true }
  //     )})`,
  //   ],
  //   [
  //     marketInfo?.longPoolAmount,
  //     marketInfo?.longToken.decimals,
  //     marketInfo?.longToken.symbol,
  //     marketInfo?.maxLongPoolAmount,
  //     mintableInfo?.longDepositCapacityAmount,
  //   ]
  // );

  // const maxShortTokenValue = useMemo(
  //   () => [
  //     formatTokenAmount(
  //       mintableInfo?.shortDepositCapacityAmount,
  //       marketInfo?.shortToken.decimals,
  //       marketInfo?.shortToken.symbol,
  //       {
  //         useCommas: true,
  //       }
  //     ),
  //     `(${formatTokenAmount(marketInfo?.shortPoolAmount, marketInfo?.shortToken.decimals, undefined, {
  //       displayDecimals: 0,
  //       useCommas: true,
  //     })} / ${formatTokenAmount(
  //       marketInfo?.maxShortPoolAmount,
  //       marketInfo?.shortToken.decimals,
  //       marketInfo?.shortToken.symbol,
  //       { displayDecimals: 0, useCommas: true }
  //     )})`,
  //   ],
  //   [
  //     marketInfo?.maxShortPoolAmount,
  //     marketInfo?.shortPoolAmount,
  //     marketInfo?.shortToken.decimals,
  //     marketInfo?.shortToken.symbol,
  //     mintableInfo?.shortDepositCapacityAmount,
  //   ]
  // );

  return (
    <div className="App-card MarketStats-card">
      <MarketTokenSelector
        marketTokensData={marketTokensData}
        marketsInfoData={marketsInfoData}
        marketsTokensAPRData={marketsTokensAPRData}
        marketsTokensIncentiveAprData={marketsTokensIncentiveAprData}
        currentMarketInfo={marketInfo}
      />
      <div className="App-card-divider" />
      <div className="App-card-content">
        <CardRow
          label={t`Market`}
          value={
            indexName && poolName ? (
              <div className="items-top">
                <span>{indexName}</span>
                <span className="subtext gm-market-name">[{poolName}]</span>
              </div>
            ) : (
              "..."
            )
          }
        />
        <CardRow
          label={t`Price`}
          value={
            <Tooltip
              handle={
                formatUsd(marketPrice, {
                  displayDecimals: 3,
                }) || "..."
              }
              position="bottom-end"
              renderContent={() => {
                return (
                  <div>
                    <Trans>GM Token pricing includes positions&apos; Pending PnL, Impact Pool Amount and Borrow Fees.</Trans>
                  </div>
                );
              }}
            />
          }
        />

        <CardRow
          label={t`Wallet`}
          value={formatTokenAmountWithUsd(
            marketBalance || BN_ZERO,
            marketBalanceUsd || BN_ZERO,
            "GM",
            marketToken?.decimals ?? GM_DECIMALS
          )}
        />

        <CardRow
          label={t`APR`}
          value={<AprInfo apr={BN_ZERO} />}
        />

        <CardRow
          label={t`Total Supply`}
          value={
            marketTotalSupply && marketTotalSupplyUsd
              ? formatTokenAmountWithUsd(marketTotalSupply, marketTotalSupplyUsd, "GM", marketToken?.decimals, {
                displayDecimals: 0,
              })
              : "..."
          }
        />

        <CardRow
          label={t`Buyable`}
          value={
            marketTotalSupplyUsd && marketToken ? (
              <Tooltip
                maxAllowedWidth={350}
                handle={formatTokenAmountWithUsd(
                  marketToken.maxMintable,
                  getMaxMintableUsd(marketToken),
                  "GM",
                  marketToken?.decimals,
                  {
                    displayDecimals: 0,
                  }
                )}
                position="bottom-end"
                renderContent={() => {
                  return (
                    <div>
                      {marketInfo?.isSingle ? (
                        <Trans>
                          {marketInfo?.longToken.symbol} can be used to buy GM for this market up to the specified
                          buying caps.
                        </Trans>
                      ) : (
                        <Trans>
                          {marketInfo?.longToken.symbol} and {marketInfo?.shortToken.symbol} can be used to buy GM for
                          this market up to the specified buying caps.
                        </Trans>
                      )}
                      {/* 
                      <br />
                      <br />

                      <StatsTooltipRow
                        label={t`Max ${marketInfo?.longToken.symbol}`}
                        value={maxLongTokenValue}
                        showDollar={false}
                      />

                      <br />

                      {!marketInfo?.isSameCollaterals && (
                        <StatsTooltipRow
                          label={t`Max ${marketInfo?.shortToken.symbol}`}
                          value={maxShortTokenValue}
                          showDollar={false}
                        />
                      )} */}
                    </div>
                  );
                }}
              />
            ) : (
              "..."
            )
          }
        />

        <CardRow
          label={t`Sellable`}
          value={
            <Tooltip
              maxAllowedWidth={300}
              handle={formatTokenAmountWithUsd(
                sellableInfo?.totalAmount,
                sellableInfo?.totalUsd,
                "GM",
                marketToken?.decimals,
                {
                  displayDecimals: 0,
                }
              )}
              position="bottom-end"
              renderContent={() => (
                <div>
                  <Trans>
                    GM can be sold for {longToken?.symbol} and {shortToken?.symbol} for this market up to the specified
                    selling caps. The remaining tokens in the pool are reserved for currently open Positions.
                  </Trans>
                  <br />
                  <br />
                  <StatsTooltipRow
                    label={t`Max ${marketInfo?.longToken.symbol}`}
                    value={formatTokenAmountWithUsd(
                      maxLongSellableTokenAmount,
                      sellableInfo?.maxLongSellableUsd,
                      longToken?.symbol,
                      longToken?.decimals
                    )}
                    showDollar={false}
                  />
                  <StatsTooltipRow
                    label={t`Max ${marketInfo?.shortToken.symbol}`}
                    value={formatTokenAmountWithUsd(
                      maxShortSellableTokenAmount,
                      sellableInfo?.maxShortSellableUsd,
                      shortToken?.symbol,
                      shortToken?.decimals
                    )}
                    showDollar={false}
                  />
                </div>
              )}
            />
          }
        />

        <div className="App-card-divider" />
        {marketInfo?.isSingle ? (
          <>
            <CardRow label={t`Collateral`} value={longToken?.symbol || "..."} />
            <CardRow
              label={t`Pool Amount`}
              value={formatTokenAmountWithUsd(
                longPoolAmount?.add(shortPoolAmount ?? BN_ZERO),
                longPoolAmountUsd?.add(shortPoolAmountUsd ?? BN_ZERO),
                longToken?.symbol,
                longToken?.decimals
              )}
            />
            {shouldShowMoreInfo && (
              // <CardRow
              //   label={t`Read more`}
              //   value={<BridgingInfo chainId={chainId} tokenSymbol={longToken?.symbol} />}
              // />
              <></>
            )}
          </>
        ) : (
          <>
            <CardRow label={t`Long Collateral`} value={longToken?.symbol || "..."} />
            <CardRow
              label={t`Pool Amount`}
              value={formatTokenAmountWithUsd(
                longPoolAmount,
                longPoolAmountUsd,
                longToken?.symbol,
                longToken?.decimals
              )}
            />
            {shouldShowMoreInfo && (
              // <CardRow
              //   label={t`Read more`}
              //   value={<BridgingInfo chainId={chainId} tokenSymbol={longToken?.symbol} />}
              // />
              <></>
            )}
            <div className="App-card-divider" />
            <CardRow label={t`Short Collateral`} value={shortToken?.symbol || "..."} />
            <CardRow
              label={t`Pool Amount`}
              value={formatTokenAmountWithUsd(
                shortPoolAmount,
                shortPoolAmountUsd,
                shortToken?.symbol,
                shortToken?.decimals
              )}
            />
          </>
        )}
      </div>
    </div>
  );
}
