import { useCallback } from "react";
import Modal from "../Modal/Modal";
import "./PositionSeller.scss";
import { Trans, t } from "@lingui/macro";
import BuyInputSection from "../BuyInputSection/BuyInputSection";
import Button from "../Button/Button";
import { useClearClosingPosition, useClosingPosition } from "@/contexts/shared";

export function PositionSeller() {
  const position = useClosingPosition();
  const isVisible = Boolean(position);

  const handleClose = useClearClosingPosition();

  const handleSubmit = useCallback(() => {

  }, []);

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
              // topRightValue={formatUsd(maxCloseSize)}
              // inputValue={closeUsdInputValue}
              // onInputValueChange={(e) => setCloseUsdInputValue(e.target.value)}
              // showMaxButton={maxCloseSize?.gt(0) && !closeSizeUsd?.eq(maxCloseSize)}
              // onClickMax={() => setCloseUsdInputValueRaw(formatAmountFree(maxCloseSize, USD_DECIMALS))}
              // showPercentSelector={true}
              // onPercentChange={(percentage) => {
              //   const formattedAmount = formatAmountFree(maxCloseSize.mul(percentage).div(100), USD_DECIMALS, 2);
              //   setCloseUsdInputValueRaw(formattedAmount);
              // }}
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
