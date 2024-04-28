import { TradeBox } from "@/components/TradeBox/TradeBox";
import "./Exchange.css";

import { Helmet } from "react-helmet-async";
import { usePending } from "@/contexts/pending";
import { useTradeParamsProcessor } from "@/onchain/trade/use-trade-params-processor";
import { TVChart } from "@/components/TVChart/TVChart";

export default function Exchange() {
  const { setPendingTxs } = usePending();

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
              {/* <Tab
                options={tabOptions}
                optionLabels={tabLabels}
                option={listSection}
                onChange={handleTabChange}
                type="inline"
                className="Exchange-list-tabs"
              /> */}
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
