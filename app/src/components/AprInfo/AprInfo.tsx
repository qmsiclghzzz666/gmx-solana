import { BN_ZERO } from "@/config/constants";
import { BN } from "@coral-xyz/anchor";
import { formatAmount } from "../MarketsList/utils";
import Tooltip from "../Tooltip/Tooltip";
import { useCallback } from "react";
import StatsTooltipRow from "../StatsTooltipRow/StatsTooltipRow";
import { t } from "@lingui/macro";

export function AprInfo({
  apr,
  showTooltip = true,
}: {
  apr?: BN,
  showTooltip?: boolean,
}) {
  const totalApr = apr ?? BN_ZERO;
  const aprNode = <>{apr ? `${formatAmount(totalApr, 2, 2)}%` : "..."}</>;
  const renderTooltipContent = useCallback(() => {
    return <StatsTooltipRow showDollar={false} label={t`Base APR`} value={`${formatAmount(apr ?? null, 2, 2)}%`} />;
    // return (
    //   <>
    //     <StatsTooltipRow showDollar={false} label={t`Base APR`} value={`${formatAmount(apr, 2, 2)}%`} />
    //     <StatsTooltipRow showDollar={false} label={t`Bonus APR`} value={`${formatAmount(incentiveApr, 2, 2)}%`} />
    //     <br />
    //     <Trans>
    //       The Bonus APR will be airdropped as ARB tokens.{" "}
    //       <ExternalLink href="https://gmxio.notion.site/GMX-S-T-I-P-Incentives-Distribution-1a5ab9ca432b4f1798ff8810ce51fec3#5c07d62e5676466db25f30807ef0a647">
    //         Read more
    //       </ExternalLink>
    //       .
    //     </Trans>
    //   </>
    // );
  }, [apr]);
  return showTooltip ? (
    <Tooltip maxAllowedWidth={280} handle={aprNode} position="bottom-end" renderContent={renderTooltipContent} />
  ) : (
    aprNode
  );
}
