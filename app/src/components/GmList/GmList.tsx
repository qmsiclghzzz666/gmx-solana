import { useMemo } from "react";
import { Trans, t } from "@lingui/macro";
import { MarketInfos, MarketTokenAPRs } from "@/onchain/market";
import { TokenData, Tokens } from "@/onchain/token";
import { useMedia } from "react-use";
import icon_solana from "@/img/ic_solana_24.svg";
import Tooltip from "../Tooltip/Tooltip";
import PageTitle from "../PageTitle/PageTitle";
import { useSortedPoolsWithIndexToken } from "@/hooks";
import { getByKey } from "@/utils/objects";
import { getMidPrice, getTokenData } from "@/onchain/token/utils";
import { convertToUsd, formatTokenAmount } from "@/utils/number";
import { getNormalizedTokenSymbol } from "@/utils/tokens";
import { formatUsd, getMarketIndexName, getMarketPoolName } from "../MarketsList/utils";
import Button from "../Button/Button";
import TokenIcon from "../TokenIcon/TokenIcon";

import "./GmList.scss";
import { useAnchor } from "@/contexts/anchor";
import { getTotalGmInfo } from "@/onchain/market/utils";
import { GmTokensBalanceInfo, GmTokensTotalBalanceInfo } from "../GmTokensBalanceInfo/GmTokensBalanceInfo";
import { BN } from "@coral-xyz/anchor";

type Props = {
  hideTitle?: boolean;
  marketsInfoData?: MarketInfos;
  tokensData?: Tokens;
  marketTokensData?: Tokens;
  marketsTokensAPRData?: MarketTokenAPRs;
  marketsTokensIncentiveAprData?: MarketTokenAPRs;
  shouldScrollToTop?: boolean;
  buySellActionHandler?: () => void;
};

interface TotalBalance {
  balance: BN;
  balanceUsd: BN;
}

export function GmList({
  hideTitle,
  marketTokensData,
  marketsInfoData,
  tokensData,
  marketsTokensAPRData,
  marketsTokensIncentiveAprData,
  shouldScrollToTop,
  buySellActionHandler,
}: Props) {
  const isMobile = useMedia("(max-width: 1100px)");

  // const { chainId } = useChainId();
  const { active } = useAnchor();
  // const currentIcons = getIcons(chainId);
  // const userEarnings = useUserEarnings(chainId);

  // const daysConsidered = useDaysConsideredInMarketsApr();
  const daysConsidered = 1;

  const { markets } = useSortedPoolsWithIndexToken(marketsInfoData, marketTokensData);
  // const isLpIncentiveActive = useIncentiveStats()?.lp?.isActive ?? false;

  const userTotalGmInfo = useMemo(() => {
    if (active) {
      return getTotalGmInfo(marketTokensData);
    }
  }, [marketTokensData, active]);

  return (
    <div className="GMList">
      {!isMobile && <DesktopList
        hideTitle={hideTitle}
        daysConsidered={daysConsidered}
        sortedMarketsByIndexToken={markets}
        marketsInfoData={marketsInfoData}
        tokensData={tokensData}
        marketsTokensAPRData={marketsTokensAPRData}
        marketsTokensIncentiveAprData={marketsTokensIncentiveAprData}
        shouldScrollToTop={shouldScrollToTop}
        buySellActionHandler={buySellActionHandler}
        userTotalGmInfo={userTotalGmInfo}
      />}
      {isMobile && <MobileList
        hideTitle={hideTitle}
        sortedMarketsByIndexToken={markets}
        marketsInfoData={marketsInfoData}
        tokensData={tokensData}
        marketsTokensAPRData={marketsTokensAPRData}
        marketsTokensIncentiveAprData={marketsTokensIncentiveAprData}
        shouldScrollToTop={shouldScrollToTop}
        buySellActionHandler={buySellActionHandler}
        daysConsidered={daysConsidered}
        userTotalGmInfo={userTotalGmInfo}
      />}
    </div>
  );
}

function DesktopList({
  hideTitle,
  daysConsidered,
  sortedMarketsByIndexToken,
  marketsInfoData,
  tokensData,
  shouldScrollToTop,
  userTotalGmInfo,
}: {
  sortedMarketsByIndexToken: TokenData[],
  daysConsidered: number,
  userTotalGmInfo?: TotalBalance,
} & Props) {
  return (
    <div className="token-table-wrapper App-card">
      {!hideTitle && (
        <>
          <div className="App-card-title">
            <Trans>GM Pools</Trans>
            <img src={icon_solana} width="16" alt="Network Icon" />
          </div>
          <div className="App-card-divider"></div>
        </>
      )}

      <table className="token-table">
        <thead>
          <tr>
            <th>
              <Trans>MARKET</Trans>
            </th>
            <th>
              <Trans>PRICE</Trans>
            </th>
            <th>
              <Trans>TOTAL SUPPLY</Trans>
            </th>
            <th>
              <Tooltip
                handle={<Trans>BUYABLE</Trans>}
                className="text-none"
                position="bottom-end"
                renderContent={() => (
                  <p className="text-white">
                    <Trans>Available amount to deposit into the specific GM pool.</Trans>
                  </p>
                )}
              />
            </th>
            <th>
              <GmTokensTotalBalanceInfo
                balance={userTotalGmInfo?.balance}
                balanceUsd={userTotalGmInfo?.balanceUsd}
                // userEarnings={userEarnings}
                label={t`WALLET`}
              />
            </th>
            <th>
              <Tooltip
                handle={t`APR`}
                className="text-none"
                position="bottom-end"
                renderContent={() => (
                  <p className="text-white">
                    <Trans>
                      <p>
                        APR is based on the Fees collected for the past {daysConsidered} days. It is an estimate as
                        actual Fees are auto-compounded into the pool in real-time.
                      </p>
                    </Trans>
                  </p>
                )}
              />
            </th>

            <th></th>
          </tr>
        </thead>
        <tbody>
          {sortedMarketsByIndexToken.length ? (
            sortedMarketsByIndexToken.map((token) => {
              const market = getByKey(marketsInfoData, token?.address.toBase58())!;

              const indexToken = getTokenData(tokensData, market?.indexTokenAddress, "native");
              const longToken = getTokenData(tokensData, market?.longTokenAddress);
              const shortToken = getTokenData(tokensData, market?.shortTokenAddress);
              // const mintableInfo = market && token ? getMintableMarketTokens(market, token) : undefined;

              // const apr = getByKey(marketsTokensAPRData, token?.address.toBase58());
              // const incentiveApr = getByKey(marketsTokensIncentiveAprData, token?.address.toBase58());
              // const marketEarnings = getByKey(userEarnings?.byMarketAddress, token?.address);

              if (!token || !indexToken || !longToken || !shortToken) {
                return null;
              }

              const totalSupply = token?.totalSupply;
              const price = token?.prices ? getMidPrice(token.prices) : undefined;
              const totalSupplyUsd = convertToUsd(totalSupply, token?.decimals, price);
              const tokenIconName = market.isSpotOnly
                ? getNormalizedTokenSymbol(longToken.symbol) + getNormalizedTokenSymbol(shortToken.symbol)
                : getNormalizedTokenSymbol(indexToken.symbol);

              return (
                <tr key={token.address.toBase58()}>
                  <td>
                    <div className="App-card-title-info">
                      <div className="App-card-title-info-icon">
                        <TokenIcon symbol={tokenIconName} displaySize={40} importSize={40} />
                      </div>

                      <div className="App-card-title-info-text">
                        <div className="App-card-info-title">
                          {getMarketIndexName({ indexToken, isSpotOnly: market?.isSpotOnly })}
                          <div className="Asset-dropdown-container">
                            {/* <GmAssetDropdown
                              token={token}
                              marketsInfoData={marketsInfoData}
                              tokensData={tokensData}
                            /> */}
                          </div>
                        </div>
                        <div className="App-card-info-subtitle">
                          [{getMarketPoolName({ longToken, shortToken })}]
                        </div>
                      </div>
                    </div>
                  </td>
                  <td>
                    {formatUsd(price, {
                      displayDecimals: 3,
                    })}
                  </td>

                  <td className="GmList-last-column">
                    {formatTokenAmount(totalSupply, token.decimals, "GM", {
                      useCommas: true,
                      displayDecimals: 2,
                    })}
                    <br />({formatUsd(totalSupplyUsd)})
                  </td>
                  <td className="GmList-last-column">
                    Unlimited
                    {/* <MintableAmount
                      mintableInfo={mintableInfo}
                      market={market}
                      token={token}
                      longToken={longToken}
                      shortToken={shortToken}
                    /> */}
                  </td>

                  <td>
                    <GmTokensBalanceInfo
                      token={token}
                      daysConsidered={daysConsidered}
                      oneLine={false}
                    // earnedRecently={marketEarnings?.recent}
                    // earnedTotal={marketEarnings?.total}
                    />
                  </td>

                  <td>
                    {/* <AprInfo apr={apr} incentiveApr={incentiveApr} isIncentiveActive={isLpIncentiveActive} /> */}
                    Unavailable
                  </td>

                  <td className="GmList-actions">
                    <Button
                      className="GmList-action"
                      variant="secondary"
                      to={`/earn/?market=${market.marketTokenAddress.toBase58()}&operation=deposit&scroll=${shouldScrollToTop ? "1" : "0"
                        }`}
                    >
                      <Trans>Buy</Trans>
                    </Button>
                    <Button
                      className="GmList-action GmList-last-action"
                      variant="secondary"
                      to={`/earn/?market=${market.marketTokenAddress.toBase58()}&operation=withdrawal&scroll=${shouldScrollToTop ? "1" : "0"
                        }`}
                    >
                      <Trans>Sell</Trans>
                    </Button>
                  </td>
                </tr>
              );
            })
          ) : (
            <></>
          )}
        </tbody>
      </table>
    </div>
  );
}

function MobileList(
  {
    hideTitle,
    sortedMarketsByIndexToken,
    marketsInfoData,
    tokensData,
    buySellActionHandler,
    daysConsidered,
    userTotalGmInfo,
  }: {
    hideTitle?: boolean,
    sortedMarketsByIndexToken: TokenData[],
    daysConsidered: number,
    userTotalGmInfo?: TotalBalance,
  } & Props
) {
  return (
    <>
      {!hideTitle && <PageTitle title={t`GM Pools`} />}

      <div className="token-grid">
        {sortedMarketsByIndexToken.map((token) => {
          // const apr = marketsTokensAPRData?.[token.address];
          // const incentiveApr = marketsTokensIncentiveAprData?.[token.address];
          // const marketEarnings = getByKey(userEarnings?.byMarketAddress, token?.address);

          const totalSupply = token?.totalSupply;
          const price = token?.prices ? getMidPrice(token.prices) : undefined;
          const totalSupplyUsd = convertToUsd(totalSupply, token?.decimals, price);
          const market = getByKey(marketsInfoData, token?.address.toBase58());
          const indexToken = getTokenData(tokensData, market?.indexTokenAddress, "native");
          const longToken = getTokenData(tokensData, market?.longTokenAddress);
          const shortToken = getTokenData(tokensData, market?.shortTokenAddress);
          // const mintableInfo = market && token ? getMintableMarketTokens(market, token) : undefined;

          if (!indexToken || !longToken || !shortToken || !market) {
            return null;
          }
          const indexName = market && getMarketIndexName(market);
          const poolName = market && getMarketPoolName(market);
          const tokenIconName = market.isSpotOnly
            ? getNormalizedTokenSymbol(longToken.symbol) + getNormalizedTokenSymbol(shortToken.symbol)
            : getNormalizedTokenSymbol(indexToken.symbol);

          return (
            <div className="App-card" key={token.address.toBase58()}>
              <div className="App-card-title">
                <div className="mobile-token-card">
                  <TokenIcon symbol={tokenIconName} displaySize={20} importSize={40} />
                  <div className="token-symbol-text">
                    <div className="items-center">
                      <span>{indexName && indexName}</span>
                      <span className="subtext">{poolName && `[${poolName}]`}</span>
                    </div>
                  </div>
                  <div>
                    {/* <GmAssetDropdown
                      token={token}
                      tokensData={tokensData}
                      marketsInfoData={marketsInfoData}
                      position={index % 2 !== 0 ? "left" : "right"}
                    /> */}
                  </div>
                </div>
              </div>
              <div className="App-card-divider"></div>
              <div className="App-card-content">
                <div className="App-card-row">
                  <div className="label">
                    <Trans>Price</Trans>
                  </div>
                  <div>
                    {formatUsd(price, {
                      displayDecimals: 3,
                    })}
                  </div>
                </div>
                <div className="App-card-row">
                  <div className="label">
                    <Trans>Total Supply</Trans>
                  </div>
                  <div>
                    {" "}
                    {formatTokenAmount(totalSupply, token.decimals, "GM", {
                      useCommas: true,
                      displayDecimals: 2,
                    })}{" "}
                    ({formatUsd(totalSupplyUsd)})
                  </div>
                </div>
                <div className="App-card-row">
                  <div className="label">
                    <Tooltip
                      handle={<Trans>Buyable</Trans>}
                      className="text-none"
                      position="bottom-start"
                      renderContent={() => (
                        <p className="text-white">
                          <Trans>Available amount to deposit into the specific GM pool.</Trans>
                        </p>
                      )}
                    />
                  </div>
                  <div>
                    Unlimited
                    {/* <MintableAmount
                      mintableInfo={mintableInfo}
                      market={market}
                      token={token}
                      longToken={longToken}
                      shortToken={shortToken}
                    /> */}
                  </div>
                </div>
                <div className="App-card-row">
                  <div className="label">
                    <GmTokensTotalBalanceInfo
                      balance={userTotalGmInfo?.balance}
                      balanceUsd={userTotalGmInfo?.balanceUsd}
                      // userEarnings={userEarnings}
                      tooltipPosition="bottom-start"
                      label={t`Wallet`}
                    />
                  </div>
                  <div>
                    <GmTokensBalanceInfo
                      token={token}
                      daysConsidered={daysConsidered}
                      oneLine
                    // earnedRecently={marketEarnings?.recent}
                    // earnedTotal={marketEarnings?.total}
                    />
                  </div>
                </div>
                <div className="App-card-row">
                  <div className="label">
                    <Trans>APR</Trans>
                  </div>
                  <div>
                    {/* <AprInfo apr={apr} incentiveApr={incentiveApr} isIncentiveActive={isLpIncentiveActive} /> */}
                    Unavailable
                  </div>
                </div>

                <div className="App-card-divider"></div>
                <div className="App-card-buttons m-0" onClick={buySellActionHandler}>
                  <Button
                    variant="secondary"
                    to={`/earn/?market=${market.marketTokenAddress.toBase58()}&operation=deposit&scroll=0`}
                  >
                    <Trans>Buy</Trans>
                  </Button>
                  <Button
                    variant="secondary"
                    to={`/earn/?market=${market.marketTokenAddress.toBase58()}&operation=withdrawal&scroll=0`}
                  >
                    <Trans>Sell</Trans>
                  </Button>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </>
  );
}

// function MintableAmount({ mintableInfo, market, token, longToken, shortToken }) {
//   const longTokenMaxValue = useMemo(
//     () => [
//       formatTokenAmount(mintableInfo?.longDepositCapacityAmount, longToken.decimals, longToken.symbol, {
//         useCommas: true,
//       }),
//       `(${formatTokenAmount(market.longPoolAmount, longToken.decimals, "", {
//         useCommas: true,
//         displayDecimals: 0,
//       })} / ${formatTokenAmount(getMaxPoolAmountForDeposit(market, true), longToken.decimals, longToken.symbol, {
//         useCommas: true,
//         displayDecimals: 0,
//       })})`,
//     ],
//     [longToken.decimals, longToken.symbol, market, mintableInfo?.longDepositCapacityAmount]
//   );
//   const shortTokenMaxValue = useMemo(
//     () => [
//       formatTokenAmount(mintableInfo?.shortDepositCapacityAmount, shortToken.decimals, shortToken.symbol, {
//         useCommas: true,
//       }),
//       `(${formatTokenAmount(market.shortPoolAmount, shortToken.decimals, "", {
//         useCommas: true,
//         displayDecimals: 0,
//       })} / ${formatTokenAmount(getMaxPoolAmountForDeposit(market, false), shortToken.decimals, shortToken.symbol, {
//         useCommas: true,
//         displayDecimals: 0,
//       })})`,
//     ],
//     [market, mintableInfo?.shortDepositCapacityAmount, shortToken.decimals, shortToken.symbol]
//   );
//   return (
//     <Tooltip
//       maxAllowedWidth={350}
//       handle={
//         <>
//           {formatTokenAmount(mintableInfo?.mintableAmount, token.decimals, "GM", {
//             useCommas: true,
//             displayDecimals: 0,
//           })}
//           <br />(
//           {formatUsd(mintableInfo?.mintableUsd, {
//             displayDecimals: 0,
//           })}
//           )
//         </>
//       }
//       className="text-none"
//       position="bottom-end"
//       renderContent={() => (
//         <>
//           <p className="text-white">
//             <Trans>
//               {longToken.symbol} and {shortToken.symbol} can be used to buy GM tokens for this market up to the
//               specified buying caps.
//             </Trans>
//           </p>
//           <br />
//           <StatsTooltipRow label={`Max ${longToken.symbol}`} value={longTokenMaxValue} />
//           <StatsTooltipRow label={`Max ${shortToken.symbol}`} value={shortTokenMaxValue} />
//         </>
//       )}
//     />
//   );
// }
