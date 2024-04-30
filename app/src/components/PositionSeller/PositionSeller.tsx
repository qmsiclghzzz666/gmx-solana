import { useCallback, useMemo } from "react";
import Modal from "../Modal/Modal";
import "./PositionSeller.scss";
import { Trans, t } from "@lingui/macro";
import BuyInputSection from "../BuyInputSection/BuyInputSection";
import Button from "../Button/Button";
import { useClearClosingPosition, useClosingPosition } from "@/contexts/shared";
import { BN_ZERO, USD_DECIMALS } from "@/config/constants";
import { formatUsd } from "../MarketsList/utils";
import { formatAmountFree, parseValue } from "@/utils/number";
import { useDebounceValue } from "usehooks-ts";
import { DEBOUNCE_MS } from "@/config/ui";

export function PositionSeller() {
  const position = useClosingPosition();
  const [closeUsdInputValue, setCloseUsdInputValue] = useDebounceValue("", DEBOUNCE_MS);
  const resetInputs = useCallback(() => {
    setCloseUsdInputValue("");
  }, [setCloseUsdInputValue]);
  const clearClosingPosition = useClearClosingPosition();
  const handleClose = useCallback(() => {
    clearClosingPosition();
    resetInputs();
  }, [clearClosingPosition, resetInputs]);
  const handleSubmit = useCallback(() => {

  }, []);

  const isVisible = Boolean(position);
  const maxCloseSize = position?.sizeInUsd ?? BN_ZERO;
  const closeSizeUsd = useMemo(() => parseValue(closeUsdInputValue || "0", USD_DECIMALS)!, [closeUsdInputValue]);

  return (
    <div className="PositionEditor PositionSeller">
      <Modal
        className="PositionSeller-modal"
        isVisible={isVisible}
        setIsVisible={handleClose}
        label={
          <Trans>
            Close {position?.isLong ? t`Long` : t`Short`} {position?.marketInfo.indexToken?.symbol}
          </Trans>
        }
      >
        {position && (
          <>
            <div className="relative">
              <BuyInputSection
                topLeftLabel={t`Close`}
                topRightLabel={t`Max`}
                topRightValue={formatUsd(maxCloseSize)}
                inputValue={closeUsdInputValue}
                onInputValueChange={(e) => setCloseUsdInputValue(e.target.value)}
                showMaxButton={maxCloseSize?.gt(BN_ZERO) && !closeSizeUsd?.eq(maxCloseSize)}
                onClickMax={() => setCloseUsdInputValue(formatAmountFree(maxCloseSize, USD_DECIMALS))}
                showPercentSelector
                onPercentChange={(percentage) => {
                  const formattedAmount = formatAmountFree(maxCloseSize.muln(percentage).divn(100), USD_DECIMALS, 2);
                  setCloseUsdInputValue(formattedAmount);
                }}
              >
                USD
              </BuyInputSection>
            </div>
            <div className="Exchange-swap-button-container">
              <Button
                className="w-full"
                variant="primary-action"
                // disabled={Boolean(error) && !shouldDisableValidationForTesting}
                onClick={handleSubmit}
              // buttonRef={submitButtonRef}
              >
                {/* {error ||
                  (isTrigger
                    ? t`Create ${getTriggerNameByOrderType(decreaseAmounts?.triggerOrderType)} Order`
                    : t`Close`)} */}
                {t`Close`}
              </Button>
            </div>
          </>
        )}
      </Modal>
    </div>
  );
}
