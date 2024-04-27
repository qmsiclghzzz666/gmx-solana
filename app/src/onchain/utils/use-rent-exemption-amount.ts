import { BN } from "@coral-xyz/anchor";
import { useConnection } from "@solana/wallet-adapter-react";
import { useEffect, useRef } from "react";
import useSWR from "swr";
import { toBN } from "gmsol";

export const useRentExemptionAmount = (dataLength: number) => {
  const connection = useConnection();
  const { data } = useSWR(['rent-exemption-amount', dataLength], async (key) => {
    return await connection.connection.getMinimumBalanceForRentExemption(key[1]);
  }, { refreshInterval: 3600 });

  const dataRef = useRef<BN | null>(null);

  useEffect(() => {
    if (data) {
      dataRef.current = toBN(data);
    }
  }, [data]);

  return dataRef.current;
};
