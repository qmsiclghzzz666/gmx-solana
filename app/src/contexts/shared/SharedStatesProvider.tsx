import { useDeployedInfos } from "@/onchain/utils";
import { ReactNode, useMemo, useState } from "react";
import { SharedStates } from "./types";
import { SharedStatesCtx } from ".";
import { useTradeBoxState } from "@/onchain/trade";
import { useGenesisHash } from "@/onchain/utils";
import { PublicKey } from "@solana/web3.js";
import { Address, translateAddress } from "@coral-xyz/anchor";
import { helperToast } from "@/utils/helperToast";
import { Trans } from "@lingui/macro";

export function SharedStatesProvider({ children }: { children: ReactNode }) {
  const chainId = useGenesisHash();
  const {
    marketInfos,
    tokens,
    marketTokens,
    positionInfos,
    isPositionsLoading,
    isMarketLoading,
    isMarketTokenLoading,
  } = useDeployedInfos();
  const tradeBox = useTradeBoxState(chainId, { marketInfos, tokens });

  const [closingPositionAddress, setClosingPositionAddress] = useState<PublicKey | undefined>();

  const state = useMemo(() => {
    const state: SharedStates = {
      chainId,
      market: {
        marketInfos: marketInfos,
        tokens,
        marketTokens,
      },
      tradeBox,
      position: {
        isLoading: isPositionsLoading || isMarketLoading || isMarketTokenLoading,
        positionInfos,
      },
      positionSeller: {
        address: closingPositionAddress,
        setAddress: (address: Address | null) => {
          if (!address) return setClosingPositionAddress(undefined);
          try {
            const translated = translateAddress(address);
            setClosingPositionAddress(translateAddress(translated));
          } catch (error) {
            helperToast.error(<div>
              <Trans>Invalid closing position address</Trans>
              <br />
              {`${(error as Error).message}`}
            </div>)
          }
        }
      }
    };
    return state;
  }, [chainId, marketInfos, tokens, marketTokens, tradeBox, isPositionsLoading, isMarketLoading, isMarketTokenLoading, positionInfos, closingPositionAddress]);
  return (
    <SharedStatesCtx.Provider value={state}>
      {children}
    </SharedStatesCtx.Provider>
  );
}
