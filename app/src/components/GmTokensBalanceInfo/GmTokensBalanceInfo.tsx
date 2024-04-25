import { BN_ZERO, GM_DECIMALS } from "@/config/constants";
import { TokenData } from "@/onchain/token";
import { convertToUsd, formatTokenAmount } from "@/utils/number";
import { BN } from "@coral-xyz/anchor";
import { t } from "@lingui/macro";
import { useCallback, useMemo } from "react";
import { formatUsd } from "../MarketsList/utils";
import StatsTooltipRow from "../StatsTooltipRow/StatsTooltipRow";
import Tooltip, { TooltipPosition } from "../Tooltip/Tooltip";

export const GmTokensBalanceInfo = ({
  token,
  // earnedTotal,
  // earnedRecently,
  // daysConsidered,
  oneLine = false,
}: {
  token: TokenData;
  earnedTotal?: BN;
  earnedRecently?: BN;
  daysConsidered: number;
  oneLine?: boolean;
}) => {
  const content = (
    <>
      {formatTokenAmount(token.balance ?? BN_ZERO, token.decimals, "GM", {
        useCommas: true,
        displayDecimals: 2,
        fallbackToZero: true,
      })}
      {oneLine ? " " : <br />}(
      {formatUsd(convertToUsd(token.balance ?? BN_ZERO, token.decimals, token.prices?.minPrice), {
        fallbackToZero: true,
      })}
      )
    </>
  );

  // const renderTooltipContent = useCallback(() => {
  //   if (!earnedTotal && !earnedRecently) return null;
  //   return (
  //     <>
  //       {earnedTotal && (
  //         <StatsTooltipRow
  //           showDollar={false}
  //           label={t`Total accrued Fees`}
  //           className={getPositiveOrNegativeClass(earnedTotal)}
  //           value={formatDeltaUsd(earnedTotal, undefined)}
  //         />
  //       )}
  //       {earnedRecently && (
  //         <StatsTooltipRow
  //           showDollar={false}
  //           className={getPositiveOrNegativeClass(earnedRecently)}
  //           label={t`${daysConsidered}d accrued Fees`}
  //           value={formatDeltaUsd(earnedRecently, undefined)}
  //         />
  //       )}
  //       <br />
  //       <div className="text-white">
  //         <Trans>The fees' USD value is calculated at the time they are accrued and does not include incentives.</Trans>
  //       </div>
  //     </>
  //   );
  // }, [daysConsidered, earnedRecently, earnedTotal]);
  // if (!earnedTotal && !earnedRecently) {
  //   return content;
  // }

  // return <TooltipWithPortal renderContent={renderTooltipContent} handle={content} position="bottom-end" />;
  return content;
};

export const GmTokensTotalBalanceInfo = ({
  balance,
  balanceUsd,
  // userEarnings,
  tooltipPosition,
  label,
}: {
  balance?: BN;
  balanceUsd?: BN;
  // userEarnings: UserEarningsData | null;
  tooltipPosition?: TooltipPosition;
  label: string;
}) => {
  // const shouldShowIncentivesNote = useLpIncentivesIsActive();
  // const daysConsidered = useDaysConsideredInMarketsApr();
  const walletTotalValue = useMemo(
    () => [
      formatTokenAmount(balance, GM_DECIMALS, "GM", {
        useCommas: true,
        fallbackToZero: true,
      }),
      `(${formatUsd(balanceUsd)})`,
    ],
    [balance, balanceUsd]
  );

  const renderTooltipContent = useCallback(() => {
    return (
      <>
        <StatsTooltipRow label={t`Wallet total`} value={walletTotalValue} showDollar={false} />
        {/* {userEarnings && (
          <>
            <StatsTooltipRow
              label={t`Wallet total accrued Fees`}
              className={getPositiveOrNegativeClass(userEarnings.allMarkets.total)}
              value={formatDeltaUsd(userEarnings.allMarkets.total, undefined, { showPlusForZero: true })}
              showDollar={false}
            />
            <StatsTooltipRow
              label={t`Wallet ${daysConsidered}d accrued Fees `}
              className={getPositiveOrNegativeClass(userEarnings.allMarkets.recent)}
              value={formatDeltaUsd(userEarnings.allMarkets.recent, undefined, { showPlusForZero: true })}
              showDollar={false}
            />
            {userEarnings.allMarkets.expected365d.gt(0) && (
              <>
                <StatsTooltipRow
                  label={t`Wallet 365d expected Fees`}
                  className={getPositiveOrNegativeClass(userEarnings.allMarkets.expected365d)}
                  value={formatDeltaUsd(userEarnings.allMarkets.expected365d, undefined, { showPlusForZero: true })}
                  showDollar={false}
                />
                <br />
                <div className="text-white">
                  <Trans>Expected 365d Fees are projected based on past {daysConsidered}d base APR.</Trans>
                </div>
                {shouldShowIncentivesNote && (
                  <>
                    <br />
                    <div className="text-white">
                      <Trans>Fee values do not include incentives.</Trans>
                    </div>
                  </>
                )}
              </>
            )}
          </>
        )} */}
      </>
    );
  }, [walletTotalValue]);

  return balance && balanceUsd ? (
    <Tooltip
      handle={label}
      className="text-none"
      maxAllowedWidth={340}
      position={tooltipPosition ?? "bottom-end"}
      renderContent={renderTooltipContent}
    />
  ) : (
    <>{label}</>
  );
};

// function useLpIncentivesIsActive() {
//   return useIncentiveStats()?.lp?.isActive ?? false;
// }
