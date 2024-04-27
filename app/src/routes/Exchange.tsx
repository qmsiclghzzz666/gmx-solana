import { TradeBox } from "@/components/TradeBox/TradeBox";
import "./Exchange.css";

import { Helmet } from "react-helmet-async";
import { usePending } from "@/contexts/pending";
import { useTradeParamsEffect } from "@/onchain/trade/use-trade-params-effect";

export default function Exchange() {
  const { setPendingTxs } = usePending();

  useTradeParamsEffect();

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
