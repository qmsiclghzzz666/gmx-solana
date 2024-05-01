import { selectMarketAddress, selectTradeBoxTradeFlags } from "@/contexts/shared/selectors/trade-box-selectors";
import "./PositionItem.scss";
import { PositionInfo, formatLeverage } from "@/onchain/position";
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
}: {
  position: PositionInfo,
  hideActions?: boolean,
  onClosePositionClick?: () => void,
  onPositionClick: (tradeMode?: TradeMode) => void,
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
      />

    );
  }

  function renderCollateral() {
    return (
      <>
        <div className={cx("position-list-collateral", { isSmall: false })}>
          <Tooltip
            handle={`${formatUsd(position.remainingCollateralUsd)}`}
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
        handle={"-"}
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
      <td className="clickable">
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
              <div>
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

                {/* {showDebugValues && (
                <>
                  <br />
                  <StatsTooltipRow
                    label={"Key"}
                    value={<div className="debug-key muted">{p.position.contractKey}</div>}
                    showDollar={false}
                  />
                </>
              )} */}
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
