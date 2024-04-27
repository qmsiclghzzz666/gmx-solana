import { useNavigate, useParams } from "react-router-dom";
import { TradeParams, TradeType } from "./types";
import { getMatchingValueFromObject } from "@/utils/objects";
import { useEffect, useRef } from "react";
import { useTradeBoxStateSelector } from "@/contexts/shared/hooks/use-trade-box-state-selector";
import { isMatch } from "lodash";

export const useTradeParamsEffect = () => {
  const savedTradeMode = useTradeBoxStateSelector(s => s.tradeMode);
  const savedTradeType = useTradeBoxStateSelector(s => s.tradeType);
  const setTradeParams = useTradeBoxStateSelector(s => s.setTradeParams);

  const prevParams = useRef<TradeParams>({
    tradeType: savedTradeType,
    tradeMode: savedTradeMode,
  });

  const { tradeType } = useParams();
  const navigate = useNavigate();

  useEffect(() => {
    const params: TradeParams = {};

    if (tradeType) {
      const validTradeType = getMatchingValueFromObject(TradeType, tradeType);
      if (validTradeType) {
        params.tradeType = validTradeType as TradeType;
      } else {
        navigate("/trade");
      }
    }

    if (!isMatch(prevParams.current, params)) {
      prevParams.current = params;
      setTradeParams(params);
    }
  }, [navigate, setTradeParams, tradeType]);
};
