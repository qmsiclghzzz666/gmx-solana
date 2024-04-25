import { BN_ZERO, MIN_RESIDUAL_AMOUNT } from "@/config/constants";
import { useWrapNativeToken } from "@/onchain";
import { TokenData } from "@/onchain/token";
import { convertToUsd, formatAmountFree, formatTokenAmount, parseValue } from "@/utils/number";
import { useCallback, useMemo, useState } from "react";
import Modal from "../Modal/Modal";
import { t } from "@lingui/macro";
import BuyInputSection from "../BuyInputSection/BuyInputSection";
import { formatUsd } from "../MarketsList/utils";
import TokenWithIcon from "../TokenIcon/TokenWithIcon";
import Button from "../Button/Button";

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

  const wrapNativeToken = useWrapNativeToken(handleSubmitted);

  const { nativeTokenAmount, nativeTokenUsd } = useMemo(() => {
    const nativeTokenAmount = parseValue(inputValue, nativeToken.decimals) ?? BN_ZERO;
    const nativeTokenUsd = convertToUsd(nativeTokenAmount, nativeToken.decimals, nativeToken.prices.minPrice);
    return {
      nativeTokenAmount,
      nativeTokenUsd,
    }
  }, [inputValue, nativeToken]);

  const handleSubmit = useCallback(() => {
    wrapNativeToken(nativeTokenAmount);
  }, [nativeTokenAmount, wrapNativeToken]);

  const showMaxButton = !nativeTokenAmount.eq(nativeToken.balance ?? BN_ZERO) && nativeToken.balance?.gt(MIN_RESIDUAL_AMOUNT);
  const onMaxClick = useCallback(() => {
    if (nativeToken.balance) {
      const maxAvailableAmount = nativeToken.balance.gt(MIN_RESIDUAL_AMOUNT) ? nativeToken.balance.sub(MIN_RESIDUAL_AMOUNT) : BN_ZERO;
      const finalAmount = formatAmountFree(maxAvailableAmount, nativeToken.decimals);
      setInputValue(finalAmount);
    }
  }, [nativeToken]);

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
        >
          {t`Wrap`}
        </Button>
      </form>
    </Modal>
  );
}
