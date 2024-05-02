import { selectMarketAddress, selectTradeBoxTradeFlags } from "@/contexts/shared/selectors/trade-box-selectors";
import "./PositionItem.scss";
import { PositionInfo, formatLeverage, formatLiquidationPrice } from "@/onchain/position";
import cx from "classnames";
import { createStructuredSelector } from "reselect";
import { selectTradeBoxCollateralTokenAddress } from "@/contexts/shared/selectors/trade-box-selectors/select-trade-box-collateral-token-address";
import { useSharedStatesSelector } from "@/contexts/shared";
import { createSharedStatesSelector } from "@/contexts/shared/utils";
import Tooltip from "../Tooltip/Tooltip";
import TokenIcon from "../TokenIcon/TokenIcon";
import StatsTooltipRow from "../StatsTooltipRow/StatsTooltipRow";
import { Trans, t } from "@lingui/macro";
import { formatUsd, getMarketIndexName, getMarketPoolName } from "../MarketsList/utils";
import { BN_ZERO } from "@/config/constants";
import { formatDeltaUsd, formatTokenAmount } from "@/utils/number";
import { useCallback } from "react";
import { TradeMode, TradeParams } from "@/onchain/trade";
import { getTradeParamsFromPosition } from "@/onchain/trade/utils";
import ExternalLink from "../ExternalLink/ExternalLink";
import { getAddressUrl } from "@/utils/explorer";
import { getPositiveOrNegativeClass } from "@/utils/ui";

export function PositionItem({
  position,
  isLarge,
  onClosePositionClick,
  onPositionClick,
  ...ops
}: {
  position: PositionInfo,
  isLarge: boolean,
  onPositionClick: (params: TradeParams) => void,
  onClosePositionClick?: (address: string) => void,
  showDebugValues?: boolean,
}) {
  const address = position.address;
  const handleClosePositionClick = useCallback(() => onClosePositionClick && onClosePositionClick(address.toBase58()), [address, onClosePositionClick]);
  const handlePositionClick = useCallback((tradeMode?: TradeMode) => {
    const params = getTradeParamsFromPosition(position) as TradeParams;
    if (tradeMode) {
      params.tradeMode = tradeMode;
    }
    onPositionClick(params);
  }, [onPositionClick, position]);
  return isLarge ? <Large position={position} onPositionClick={handlePositionClick} onClosePositionClick={handleClosePositionClick} {...ops} /> : <></>;
}

const selectIsCurrentMarket = createStructuredSelector({
  currentMarketTokenAddress: selectMarketAddress,
  currentCollateralTokenAddress: selectTradeBoxCollateralTokenAddress,
  isCurrentTradeTypeLong: createSharedStatesSelector([selectTradeBoxTradeFlags], flags => flags.isLong),
}, createSharedStatesSelector);

function Large({
  position,
  hideActions,
  onClosePositionClick,
  onPositionClick,
  showDebugValues,
}: {
  position: PositionInfo,
  hideActions?: boolean,
  onClosePositionClick?: () => void,
  onPositionClick: (tradeMode?: TradeMode) => void,
  showDebugValues?: boolean,
}) {
  const {
    currentCollateralTokenAddress,
    currentMarketTokenAddress,
    isCurrentTradeTypeLong,
  } = useSharedStatesSelector(selectIsCurrentMarket);
  const isCurrentMarket = position.marketTokenAddress.toBase58() === currentMarketTokenAddress &&
    position.collateralTokenAddress.toBase58() === currentCollateralTokenAddress &&
    position.isLong === isCurrentTradeTypeLong;

  const indexName = getMarketIndexName(position.marketInfo);
  const poolName = getMarketPoolName(position.marketInfo);
  const indexPriceDecimals = position?.marketInfo.indexToken?.priceDecimals;
  // const displayedPnl = savedShowPnlAfterFees ? p.position.pnlAfterFees : p.position.pnl;
  const displayedPnl = position.pnl;
  // const displayedPnlPercentage = savedShowPnlAfterFees ? p.position.pnlAfterFeesPercentage : p.position.pnlPercentage;
  const displayedPnlPercentage = position.pnlPercentage;

  function renderNetValue() {
    return (
      <Tooltip
        handle={formatUsd(position.netValue)}
        renderContent={() => (
          <div>
            {(position.uiFeeUsd && position.uiFeeUsd.gt(BN_ZERO))
              ? t`Net Value: Initial Collateral + PnL - Borrow Fee - Negative Funding Fee - Close Fee - UI Fee`
              : t`Net Value: Initial Collateral + PnL - Borrow Fee - Negative Funding Fee - Close Fee`}
            <br />
            <br />
            {/* <StatsTooltipRow
              label={t`Initial Collateral`}
              value={formatUsd(position.collateralUsd) || "..."}
              showDollar={false}
            /> */}
            <StatsTooltipRow
              label={t`PnL`}
              value={formatDeltaUsd(position?.pnl) || "..."}
              showDollar={false}
              className={getPositiveOrNegativeClass(position.pnl)}
            />
            {/* <StatsTooltipRow
              label={t`Accrued Borrow Fee`}
              value={formatUsd(p.position.pendingBorrowingFeesUsd?.mul(-1)) || "..."}
              showDollar={false}
              className={cx({
                "text-red": !p.position.pendingBorrowingFeesUsd.isZero(),
              })}
            /> */}
            {/* <StatsTooltipRow
              label={t`Accrued Negative Funding Fee`}
              value={formatUsd(p.position.pendingFundingFeesUsd.mul(-1)) || "..."}
              showDollar={false}
              className={cx({
                "text-red": !p.position.pendingFundingFeesUsd.isZero(),
              })}
            /> */}
            {/* <StatsTooltipRow
              label={t`Close Fee`}
              showDollar={false}
              value={formatUsd(p.position.closingFeeUsd?.mul(-1)) || "..."}
              className="text-red"
            /> */}
            {position.uiFeeUsd && position.uiFeeUsd.gt(BN_ZERO) && (
              <StatsTooltipRow
                label={t`UI Fee`}
                showDollar={false}
                value={formatUsd(position.uiFeeUsd.muln(-1))}
                className="text-red"
              />
            )}
            {/* <br />
            <StatsTooltipRow
              label={t`PnL After Fees`}
              value={formatDeltaUsd(p.position.pnlAfterFees, p.position.pnlAfterFeesPercentage)}
              showDollar={false}
              className={getPositiveOrNegativeClass(p.position.pnlAfterFees)}
            /> */}
          </div>
        )}
      />

    );
  }

  function renderCollateral() {
    return (
      <>
        <div className={cx("position-list-collateral", { isSmall: false })}>
          <Tooltip
            handle={`${formatUsd(position.remainingCollateralUsd)}`}
            renderContent={() => {
              // const fundingFeeRateUsd = getFundingFeeRateUsd(
              //   p.position.marketInfo,
              //   p.position.isLong,
              //   p.position.sizeInUsd,
              //   CHART_PERIODS["1d"]
              // );
              // const borrowingFeeRateUsd = getBorrowingFeeRateUsd(
              //   p.position.marketInfo,
              //   p.position.isLong,
              //   p.position.sizeInUsd,
              //   CHART_PERIODS["1d"]
              // );
              return (
                <>
                  {/* {p.position.hasLowCollateral && (
                    <div>
                      <Trans>
                        WARNING: This position has a low amount of collateral after deducting fees, deposit more
                        collateral to reduce the position's liquidation risk.
                      </Trans>
                      <br />
                      <br />
                    </div>
                  )} */}
                  {/* <StatsTooltipRow
                    label={t`Initial Collateral`}
                    value={
                      <>
                        <div>
                          {formatTokenAmount(
                            p.position.collateralAmount,
                            p.position.collateralToken.decimals,
                            p.position.collateralToken.symbol,
                            {
                              useCommas: true,
                            }
                          )}{" "}
                          ({formatUsd(p.position.collateralUsd)})
                        </div>
                      </>
                    }
                    showDollar={false}
                  />
                  <br /> */}
                  {/* <StatsTooltipRow
                    label={t`Accrued Borrow Fee`}
                    showDollar={false}
                    value={formatUsd(p.position.pendingBorrowingFeesUsd.mul(-1)) || "..."}
                    className={cx({
                      "text-red": !p.position.pendingBorrowingFeesUsd.isZero(),
                    })}
                  />
                  <StatsTooltipRow
                    label={t`Accrued Negative Funding Fee`}
                    showDollar={false}
                    value={formatDeltaUsd(p.position.pendingFundingFeesUsd.mul(-1)) || "..."}
                    className={cx({
                      "text-red": !p.position.pendingFundingFeesUsd.isZero(),
                    })}
                  />
                  <StatsTooltipRow
                    label={t`Accrued Positive Funding Fee`}
                    showDollar={false}
                    value={formatDeltaUsd(p.position.pendingClaimableFundingFeesUsd) || "..."}
                    className={cx({
                      "text-green": p.position.pendingClaimableFundingFeesUsd.gt(0),
                    })}
                  />
                  <br /> */}
                  {/* <StatsTooltipRow
                    showDollar={false}
                    label={t`Current Borrow Fee / Day`}
                    value={formatUsd(borrowingFeeRateUsd.mul(-1))}
                    className={cx({
                      "text-red": borrowingFeeRateUsd.gt(0),
                    })}
                  />
                  <StatsTooltipRow
                    showDollar={false}
                    label={t`Current Funding Fee / Day`}
                    value={formatDeltaUsd(fundingFeeRateUsd)}
                    className={getPositiveOrNegativeClass(fundingFeeRateUsd)}
                  />
                  <br /> */}
                  <Trans>Use the Edit Collateral icon to deposit or withdraw collateral.</Trans>
                  <br />
                  <br />
                  <Trans>
                    Negative Funding Fees are settled against the collateral automatically and will influence the
                    liquidation price. Positive Funding Fees can be claimed under Claimable Funding after realizing any
                    action on the position.
                  </Trans>
                </>
              );
            }}
          />
          {/* {!position.isOpening && !hideActions && onEditCollateralClick && (
            <span className="edit-icon" onClick={p.onEditCollateralClick}>
              <AiOutlineEdit fontSize={16} />
            </span>
          )} */}
        </div>
        <div className="Exchange-list-info-label Position-collateral-amount muted">
          {`(${formatTokenAmount(
            position.remainingCollateralAmount,
            position.collateralToken?.decimals,
            position.collateralToken?.symbol,
            {
              useCommas: true,
            }
          )})`}
        </div>
      </>
    )
  }

  function renderLiquidationPrice() {
    return (
      <Tooltip
        handle={formatLiquidationPrice(position.liquidationPrice, { displayDecimals: indexPriceDecimals }) || "..."}
        position="bottom-end"
        // handleClassName={cx("plain", {
        //   "LiqPrice-soft-warning": estimatedLiquidationHours && estimatedLiquidationHours < 24 * 7,
        //   "LiqPrice-hard-warning": estimatedLiquidationHours && estimatedLiquidationHours < 24,
        // })}
        handleClassName="plain"
        renderContent={() => (
          <>
            <Trans>Estimated Liquidation Price.</Trans>
            {/* {liqPriceWarning && <div>{liqPriceWarning}</div>} */}
            {/* {estimatedLiquidationHours ? (
              <div>
                <div>
                  {!liqPriceWarning && "Liquidation Price is influenced by Fees, Collateral value, and Price Impact."}
                </div>
                <br />
                <StatsTooltipRow
                  label={"Estimated time to Liquidation"}
                  value={formatEstimatedLiquidationTime(estimatedLiquidationHours)}
                  showDollar={false}
                />
                <br />
                <div>
                  Estimation based on current Borrow and Funding Fees rates reducing position's Collateral over time,
                  excluding any price movement.
                </div>
              </div>
            ) : (
              ""
            )} */}
          </>
        )}
      />
    );
  }

  return (
    <tr
      className={cx("Exchange-list-item", {
        "Exchange-list-item-active": isCurrentMarket,
      })}
      onClick={() => onPositionClick()}
    >
      <td>
        {/* title */}
        <div className="Exchange-list-title">
          <Tooltip
            handle={
              <>
                <TokenIcon
                  className="PositionList-token-icon"
                  symbol={position.marketInfo.indexToken.symbol}
                  displaySize={20}
                  importSize={24}
                />
                {position.marketInfo.indexToken.symbol}
              </>
            }
            position="bottom-start"
            handleClassName="plain"
            renderContent={() => (
              <div className="default-cursor">
                <StatsTooltipRow
                  label={t`Market`}
                  value={
                    <div className="items-center">
                      <span>{indexName && indexName}</span>
                      <span className="subtext lh-1">{poolName && `[${poolName}]`}</span>
                    </div>
                  }
                  showDollar={false}
                />

                <br />

                <div>
                  <Trans>
                    Click on the Position to select its market, then use the trade box to increase your Position Size,
                    or to set Take-Profit / Stop-Loss Orders.
                  </Trans>
                  <br />
                  <br />
                  <Trans>Use the &quot;Close&quot; button to reduce your Position Size.</Trans>
                </div>

                {showDebugValues && (
                  <>
                    <br />
                    <StatsTooltipRow
                      label={t`Address`}
                      value={
                        <ExternalLink href={getAddressUrl(position.address)}>
                          <div className="debug-key muted">
                            {position.address.toBase58()}
                          </div>
                        </ExternalLink>
                      }
                      showDollar={false}
                    />
                  </>
                )}
              </div>
            )}
          />
          {/* {position.pendingUpdate && <ImSpinner2 className="spin position-loading-icon" />} */}
        </div>
        <div className="Exchange-list-info-label">
          <span className="muted Position-leverage">{formatLeverage(position.leverage) || "..."}&nbsp;</span>
          <span className={cx({ positive: position.isLong, negative: !position.isLong })}>
            {position.isLong ? t`Long` : t`Short`}
          </span>
        </div>
      </td>
      <td>
        {/* netValue */}
        {position.isOpening ? (
          t`Opening...`
        ) : (
          <>
            {renderNetValue()}
            {displayedPnl && (
              <div
                // onClick={p.openSettings}
                className={cx("Exchange-list-info-label cursor-pointer Position-pnl", {
                  positive: displayedPnl.gt(BN_ZERO),
                  negative: displayedPnl.lt(BN_ZERO),
                  muted: displayedPnl.eq(BN_ZERO),
                })}
              >
                {formatDeltaUsd(displayedPnl, displayedPnlPercentage)}
              </div>
            )}
          </>
        )}
      </td>
      <td>
        {/* size */}
        {formatUsd(position.sizeInUsd)}
        {/* {renderPositionOrders()} */}
      </td>
      <td>
        {/* collateral */}
        <div>{renderCollateral()}</div>
      </td>
      <td>
        {/* entryPrice */}
        {position.isOpening
          ? t`Opening...`
          : formatUsd(position.entryPrice, {
            displayDecimals: indexPriceDecimals,
          })}
      </td>
      <td>
        {/* markPrice */}
        {formatUsd(position.markPrice, {
          displayDecimals: indexPriceDecimals,
        })}
      </td>
      <td>
        {/* liqPrice */}
        {renderLiquidationPrice()}
      </td>
      <td>
        {/* Close */}
        {!position.isOpening && !hideActions && (
          <button
            className="Exchange-list-action"
            onClick={onClosePositionClick}
          // disabled={p.position.sizeInUsd.eq(0)}
          >
            <Trans>Close</Trans>
          </button>
        )}
      </td>
    </tr>
  );
}
