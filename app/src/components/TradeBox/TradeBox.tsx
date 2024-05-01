import "./TradeBox.scss";
import longImg from "@/img/long.svg";
import shortImg from "@/img/short.svg";
import swapImg from "@/img/swap.svg";


import { PendingTxsSetter } from "@/contexts/pending/types";
import { TradeForm } from "./TradeForm";
import Tab from "../Tab/Tab";
import { TradeMode, TradeType } from "@/onchain/trade";
import { t } from "@lingui/macro";
import { useTradeBoxStateSelector } from "@/contexts/shared/hooks/use-trade-box-state-selector";
import { useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { ConfirmationBox } from "./ConfirmationBox";
import { useSharedStatesSelector } from "@/contexts/shared";
import { selectSetFromTokenInputValue, selectSetToTokenInputValue, selectTradeBoxTradeFlags } from "@/contexts/shared/selectors/trade-box-selectors";

interface Prop {
  setPendingTxs: PendingTxsSetter,
}

const tradeTypeIcons = {
  [TradeType.Long]: longImg,
  [TradeType.Short]: shortImg,
  [TradeType.Swap]: swapImg,
};

const tradeTypeLabels = {
  [TradeType.Long]: t`Long`,
  [TradeType.Short]: t`Short`,
  [TradeType.Swap]: t`Swap`,
};

const tradeModeLabels = {
  [TradeMode.Market]: t`Market`,
  [TradeMode.Limit]: t`Limit`,
  [TradeMode.Trigger]: t`TP/SL`,
};

export function TradeBox({
  setPendingTxs,
}: Prop) {
  void setPendingTxs;

  const { isSwap } = useSharedStatesSelector(selectTradeBoxTradeFlags);

  const {
    tradeType,
    tradeMode,
    availalbleTradeModes,
    setTradeMode,
  } = useTradeBoxStateSelector(s => s);

  const setFromTokenInputValue = useSharedStatesSelector(selectSetFromTokenInputValue);
  const setToTokenInputValue = useSharedStatesSelector(selectSetToTokenInputValue);

  const resetInputs = useCallback(() => {
    setFromTokenInputValue("");
    setToTokenInputValue("");
  }, [setFromTokenInputValue, setToTokenInputValue]);

  const navigate = useNavigate();
  const handleChangeTradeType = useCallback((tradeType: TradeType) => {
    if ((tradeType === TradeType.Swap) !== isSwap) {
      resetInputs();
    }
    navigate(`/trade/${tradeType.toLocaleLowerCase()}`);
  }, [isSwap, navigate, resetInputs]);

  const handleSelectTradeMode = useCallback((tradeMode: TradeMode) => {
    setTradeMode(tradeMode);
  }, [setTradeMode]);

  return (
    <>
      <div className="App-box SwapBox">
        <Tab
          icons={tradeTypeIcons}
          options={Object.values(TradeType)}
          optionLabels={tradeTypeLabels}
          option={tradeType}
          onChange={handleChangeTradeType}
          className="SwapBox-option-tabs"
        />
        <Tab
          options={availalbleTradeModes}
          optionLabels={tradeModeLabels}
          className="SwapBox-asset-options-tabs"
          type="inline"
          option={tradeMode}
          onChange={handleSelectTradeMode}
        />
        <TradeForm />
      </div>

      {/* {isSwap && <SwapCard maxLiquidityUsd={swapOutLiquidity} fromToken={fromToken} toToken={toToken} />}
      <div className="Exchange-swap-info-group">
        {isPosition && <MarketCard isLong={isLong} marketInfo={marketInfo} allowedSlippage={allowedSlippage} />}
      </div> */}

      <ConfirmationBox onClose={resetInputs} />
    </>
  );
}
