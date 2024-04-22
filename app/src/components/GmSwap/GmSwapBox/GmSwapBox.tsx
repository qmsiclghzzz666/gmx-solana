import { t, Trans } from "@lingui/macro";
import { useLingui } from "@lingui/react";
import cx from "classnames";
import mapValues from "lodash/mapValues";
import { Dispatch, SetStateAction, useCallback, useEffect, useMemo, useState } from "react";

import "./GmSwapBox.scss";
import { Market, MarketInfo, MarketInfos } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { Mode, Operation } from "./utils";
import { useSearchParams } from "react-router-dom";
import { getByKey } from "@/utils/objects";
import { NATIVE_TOKEN_ADDRESS } from "@/config/tokens";
import Inner from "./Inner";

type Props = {
  selectedMarketAddress?: string;
  markets: Market[];
  marketsInfoData?: MarketInfos;
  tokensData?: Tokens;
  onSelectMarket: (marketAddress: string) => void;
  operation: Operation;
  mode: Mode;
  setMode: (mode: Mode) => void;
  setOperation: (operation: Operation) => void;
};

export function GmSwapBox({
  operation,
  mode,
  setMode,
  setOperation,
  onSelectMarket,
  marketsInfoData,
  tokensData,
  selectedMarketAddress: marketAddress,
}: Props) {
  return (
    <Inner
      operation={operation}
      setOperation={setOperation}
      mode={mode}
      setMode={setMode}
      marketsInfoData={marketsInfoData}
      marketAddress={marketAddress}
    />
  );
}

// function showMarketToast(market: MarketInfo) {
//   if (!market) return;
//   const indexName = getMarketIndexName(market);
//   const poolName = getMarketPoolName(market);
//   helperToast.success(
//     <Trans>
//       <div className="inline-flex">
//         GM:&nbsp;<span>{indexName}</span>
//         <span className="subtext gm-toast">[{poolName}]</span>
//       </div>{" "}
//       <span>selected in order form</span>
//     </Trans>
//   );
// }
