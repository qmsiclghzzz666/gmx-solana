import { TradeBox } from "@/components/TradeBox/TradeBox";
import "./Exchange.css";

import { Helmet } from "react-helmet-async";
import { usePending } from "@/contexts/pending";
import { useTradeParamsProcessor } from "@/onchain/trade/use-trade-params-processor";
import { TVChart } from "@/components/TVChart/TVChart";
import Tab from "@/components/Tab/Tab";
import { useCallback, useMemo } from "react";
import { t } from "@lingui/macro";
import { useLocalStorageSerializeKey } from "@/utils/localStorage";
import { getSyntheticsListSectionKey } from "@/config/localStorage";
import { useChainId } from "@/contexts/shared";
import { PositionList } from "@/components/PositionList/PositionList";

enum ListSection {
  Positions = "Positions",
  // Orders = "Orders",
  // Trades = "Trades",
  // Claims = "Claims",
}

export default function Exchange() {
  const { setPendingTxs } = usePending();
  const chainId = useChainId();

  const [listSection, setListSection] = useLocalStorageSerializeKey(
    getSyntheticsListSectionKey(chainId ?? ""),
    ListSection.Positions
  );

  const positionsCount: number = 0;

  const tabLabels = useMemo(
    () => ({
      [ListSection.Positions]: t`Positions${positionsCount ? ` (${positionsCount})` : ""}`,
      // [ListSection.Orders]: renderOrdersTabTitle(),
      // [ListSection.Trades]: t`Trades`,
      // [ListSection.Claims]: totalClaimables > 0 ? t`Claims (${totalClaimables})` : t`Claims`,
    }),
    []
  );
  const tabOptions = useMemo(() => Object.keys(ListSection).map(section => section as ListSection), []);

  const handleTabChange = useCallback((section: ListSection) => setListSection(section), [setListSection]);
  const handlePositionListOrdersClick = useCallback(() => { }, []);
  const handleSettlePositionFeesClick = useCallback(() => { }, []);
  const handleSelectPositionClick = useCallback(() => { }, []);
  const hanldeClosePositionClick = useCallback(() => { }, []);
  const openSettings = useCallback(() => { }, []);

  useTradeParamsProcessor();

  return (
    <div className="default-container Exchange page-layout">
      <Helmet>
        <style type="text/css">
          {`
            :root {
              --main-bg-color: #08091b;
             {
         `}
        </style>
      </Helmet>
      <div className="Exchange-content">
        <div className="Exchange-left">
          <TVChart />
          <div className="Exchange-list large">
            <div className="Exchange-list-tab-container">
              <Tab
                options={tabOptions}
                optionLabels={tabLabels}
                option={listSection}
                onChange={handleTabChange}
                type="inline"
                className="Exchange-list-tabs"
              />
              {/* <div className="align-right Exchange-should-show-position-lines">
                {listSection === ListSection.Orders && selectedOrdersKeysArr.length > 0 && (
                  <button
                    className="muted font-base cancel-order-btn"
                    disabled={isCancelOrdersProcessig}
                    type="button"
                    onClick={onCancelOrdersClick}
                  >
                    <Plural value={selectedOrdersKeysArr.length} one="Cancel order" other="Cancel # orders" />
                  </button>
                )}
                <Checkbox
                  isChecked={shouldShowPositionLines}
                  setIsChecked={setShouldShowPositionLines}
                  className={cx("muted chart-positions", { active: shouldShowPositionLines })}
                >
                  <span>
                    <Trans>Chart positions</Trans>
                  </span>
                </Checkbox>
              </div> */}
            </div>
            {listSection === ListSection.Positions && (
              <PositionList
                onOrdersClick={handlePositionListOrdersClick}
                onSettlePositionFeesClick={handleSettlePositionFeesClick}
                onSelectPositionClick={handleSelectPositionClick}
                onClosePositionClick={hanldeClosePositionClick}
                openSettings={openSettings}
              />
            )}
          </div>
        </div>

        <div className="Exchange-right">
          <div className="Exchange-swap-box">
            <TradeBox
              // allowedSlippage={allowedSlippage!}
              // isHigherSlippageAllowed={isHigherSlippageAllowed}
              // setIsHigherSlippageAllowed={setIsHigherSlippageAllowed}
              setPendingTxs={setPendingTxs}
            />
          </div>
        </div>

        <div className="Exchange-lists small">

        </div>
      </div>
    </div>
  )
}
