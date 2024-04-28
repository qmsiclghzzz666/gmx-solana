import { BN_ZERO, DEFAULT_RENT_EXEMPT_FEE_FOR_ZERO, ESTIMATED_EXECUTION_FEE } from "@/config/constants";
import { useRentExemptionAmount } from "@/onchain/utils";
import { TokenData, useWrapNativeToken } from "@/onchain/token";
import { convertToUsd, formatAmountFree, formatTokenAmount, parseValue } from "@/utils/number";
import { useCallback, useMemo, useState } from "react";
import Modal from "../Modal/Modal";
import { t } from "@lingui/macro";
import BuyInputSection from "../BuyInputSection/BuyInputSection";
import { formatUsd } from "../MarketsList/utils";
import TokenWithIcon from "../TokenIcon/TokenWithIcon";
import Button from "../Button/Button";
import LoadingDots from "../Common/LoadingDots/LoadingDots";

export function WrapNativeTokenModal({
  isVisible,
  nativeToken,
  onSubmitted,
  onClose
}: {
  isVisible: boolean,
  nativeToken: TokenData,
  onSubmitted: () => void,
  onClose: () => void,
}) {
  const [inputValue, setInputValue] = useState("");

  const handleSubmitted = useCallback(() => {
    setInputValue("");
    onSubmitted();
  }, [onSubmitted, setInputValue]);

  const { trigger: wrapNativeToken, isSending } = useWrapNativeToken(handleSubmitted);

  const rentExemptionFeeAmount = useRentExemptionAmount(0);

  const minResidualAmount = useMemo(() => {
    return ESTIMATED_EXECUTION_FEE.muln(2).add(rentExemptionFeeAmount ?? DEFAULT_RENT_EXEMPT_FEE_FOR_ZERO);
  }, [rentExemptionFeeAmount]);

  const { nativeTokenAmount, nativeTokenUsd } = useMemo(() => {
    const nativeTokenAmount = parseValue(inputValue, nativeToken.decimals) ?? BN_ZERO;
    const nativeTokenUsd = convertToUsd(nativeTokenAmount, nativeToken.decimals, nativeToken.prices.minPrice);
    return {
      nativeTokenAmount,
      nativeTokenUsd,
    }
  }, [inputValue, nativeToken]);

  const handleSubmit = useCallback(() => {
    void wrapNativeToken(nativeTokenAmount);
  }, [nativeTokenAmount, wrapNativeToken]);

  const showMaxButton = !nativeTokenAmount.eq(nativeToken.balance ?? BN_ZERO) && nativeToken.balance?.gt(minResidualAmount);
  const onMaxClick = useCallback(() => {
    if (nativeToken.balance) {
      const maxAvailableAmount = nativeToken.balance.gt(minResidualAmount) ? nativeToken.balance.sub(minResidualAmount) : BN_ZERO;
      const finalAmount = formatAmountFree(maxAvailableAmount, nativeToken.decimals);
      setInputValue(finalAmount);
    }
  }, [minResidualAmount, nativeToken.balance, nativeToken.decimals]);

  return (
    <Modal isVisible={isVisible} setIsVisible={() => {
      setInputValue("");
      onClose();
    }} label={t`Wrap SOL`}>
      <form onSubmit={(e) => {
        e.preventDefault();
        handleSubmit();
      }}>
        <BuyInputSection
          topLeftLabel={t`Pay`}
          topLeftValue={nativeTokenUsd?.gt(BN_ZERO) ? formatUsd(nativeTokenUsd) : ""}
          topRightLabel={t`Balance`}
          topRightValue={formatTokenAmount(nativeToken?.balance ?? BN_ZERO, nativeToken?.decimals, "", {
            useCommas: true,
          })}
          onClickTopRightLabel={onMaxClick}
          onClickMax={onMaxClick}
          showMaxButton={showMaxButton}
          inputValue={inputValue}
          onInputValueChange={(e) => setInputValue(e.target.value)}
        >
          <div className="selected-token">
            <TokenWithIcon symbol={nativeToken.symbol} displaySize={20} />
          </div>
        </BuyInputSection>

        <Button
          className="w-full"
          variant="primary-action"
          type="submit"
          disabled={isSending}
        >
          {isSending ? <LoadingDots size={14} /> : t`Wrap`}
        </Button>
      </form>
    </Modal>
  );
}
