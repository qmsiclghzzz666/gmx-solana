import { useSharedStatesSelector } from "@/contexts/shared";
import Modal from "../Modal/Modal";
import "./ConfirmationBox.scss";
import { selectTradeBoxTradeFlags } from "@/contexts/shared/selectors/trade-box-selectors";
import { useCallback, useMemo } from "react";
import { t } from "@lingui/macro";
import { useTradeStage } from "@/contexts/shared/hooks";
import { useSetTradeStage } from "@/contexts/shared/hooks/use-set-trade-stage";
import Button from "../Button/Button";
import LoadingDots from "../Common/LoadingDots/LoadingDots";

interface Props {
  onClose?: () => void,
}

export function ConfirmationBox({
  onClose
}: Props) {
  const { isMarket, isLimit, isSwap, isLong } = useSharedStatesSelector(selectTradeBoxTradeFlags);

  const title = useMemo(() => {
    if (isMarket) {
      if (isSwap) {
        return t`Confirm Swap`;
      }

      return isLong ? t`Confirm Long` : t`Confirm Short`;
    } else if (isLimit) {
      return t`Confirm Limit Order`;
    }
    return t`Confirm`
  }, [isLimit, isLong, isMarket, isSwap]);

  const submitButtonText = useMemo(() => {
    let text = "";
    if (isMarket) {
      if (isSwap) {
        text = t`Swap`;
      } else {
        text = isLong ? t`Long` : t`Short`;
      }
    } else if (isLimit) {
      text = t`Confirm Limit Order`;
    } else {
      text = t`Confirm`;
    }
    return text;
  }, [isLimit, isLong, isMarket, isSwap]);

  const stage = useTradeStage();
  const isVisible = useMemo(() => stage === "confirmation", [stage]);

  const setStage = useSetTradeStage();
  const handleClose = useCallback(() => {
    if (onClose) {
      onClose();
    }
    setStage("trade");
  }, [onClose, setStage]);

  const { trigger, isSending, error } = useTriggerCreateOrder();

  const handleSubmit = useCallback(() => {
    if (trigger) {
      void trigger().then(handleClose);
    }
  }, [handleClose, trigger]);

  return (
    <div className="Confirmation-box">
      <Modal
        isVisible={isVisible}
        setIsVisible={handleClose}
        label={title}
      >
        <div className="Confirmation-box-row">
          <Button
            variant="primary-action"
            className="w-full"
            type="submit"
            onClick={handleSubmit}
            disabled={isSending || Boolean(error)}
          >
            {!isSending ? error ? error : submitButtonText : <LoadingDots />}
          </Button>
        </div>
      </Modal>
    </div>
  );
}

function useTriggerCreateOrder() {
  const { isMarket, isLimit, isSwap, isLong } = useSharedStatesSelector(selectTradeBoxTradeFlags);
  if (isMarket) {
    if (isSwap) {
      return {
        trigger: undefined,
        isSending: false,
        error: t`Swap orders are not supported for now.`
      }
    } else {
      return {
        trigger: async () => { },
        isSending: true,
        error: null,
      }
    }
  } else if (isLimit) {
    return {
      trigger: undefined,
      isSending: false,
      error: t`Limit orders are not supported for now.`
    }
  } else {
    return {
      trigger: undefined,
      isSending: false,
      error: t`Unsupported order type.`,
    }
  }
}
