import { useAnchor, useStoreProgram } from "@/contexts/anchor";
import { Address, translateAddress } from "@coral-xyz/anchor";
import { useMemo } from "react";
import { findPositionPDA, findPositionPDAWithKind } from "gmsol";
import { Market } from "../market";
import useSWR from "swr";
import { Position, Positions } from "./types";
import { PublicKey } from "@solana/web3.js";
import { isObject } from "lodash";

const POSITIONS_KEY = "data_store/positions";

export const usePositions = (params?: { store: Address, markets: Market[] }) => {
  const dataStore = useStoreProgram();
  const { owner } = useAnchor();
  const request = useMemo(() => {
    const storeAddress = params ? translateAddress(params.store) : undefined;
    return params && owner ? {
      key: POSITIONS_KEY,
      owner: owner.toBase58(),
      store: storeAddress!.toBase58(),
      positionAddresses: params.markets.flatMap(market => {
        return Array.from(new Set([
          findPositionPDA(storeAddress!, owner, market.marketTokenAddress, market.longTokenAddress, true)[0].toBase58(),
          findPositionPDA(storeAddress!, owner, market.marketTokenAddress, market.longTokenAddress, false)[0].toBase58(),
          findPositionPDA(storeAddress!, owner, market.marketTokenAddress, market.shortTokenAddress, true)[0].toBase58(),
          findPositionPDA(storeAddress!, owner, market.marketTokenAddress, market.shortTokenAddress, false)[0].toBase58(),
        ]).values());
      }),
    } : null;
  }, [owner, params]);

  const { data, isLoading } = useSWR(request, async ({ positionAddresses, owner, store }) => {
    const data = await dataStore.account.position.fetchMultiple(positionAddresses);
    const positions = data.filter(position => position).map((position) => {
      if (!(position!.kind === 1 || position!.kind === 2)) return;
      return {
        address: findPositionPDAWithKind(
          new PublicKey(store),
          new PublicKey(owner),
          position!.marketToken,
          position!.collateralToken,
          position!.kind,
        )[0],
        owner: new PublicKey(owner),
        marketTokenAddress: position!.marketToken,
        collateralTokenAddress: position!.collateralToken,
        isLong: position!.kind === 1,
        sizeInUsd: position!.state.sizeInUsd,
        sizeInTokens: position!.state.sizeInTokens,
        collateralAmount: position!.state.collateralAmount,
      } satisfies Position as Position;
    });
    return positions.reduce((acc, position) => {
      if (position) {
        acc[position.address.toBase58()] = position;
      }
      return acc;
    }, {} as Positions);
  });

  return {
    positions: data ?? {},
    isLoading,
  }
};

export const fitlerPositions = (value: unknown) => {
  if (isObject(value)) {
    const { key } = value as { key?: string };
    if (key === POSITIONS_KEY) {
      console.debug("filtered positions");
      return true;
    }
  }
  return false;
};
