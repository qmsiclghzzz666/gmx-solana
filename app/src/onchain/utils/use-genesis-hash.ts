import { useConnection } from "@solana/wallet-adapter-react";
import { useEffect, useRef } from "react";
import useSWR from "swr";

interface Cache {
  endpoint: string,
  genesisHash: string,
}

export const useGenesisHash = () => {
  const connection = useConnection();
  const endpoint = connection.connection.rpcEndpoint;
  const { data } = useSWR(`genesis/${endpoint}`, async () => {
    return await connection.connection.getGenesisHash();
  }, {
    refreshInterval: 0,
  });

  const cache = useRef<Cache | null>(null);

  useEffect(() => {
    if (data || cache.current === null || cache.current.endpoint !== endpoint) {
      if (cache.current !== null && !data) {
        cache.current = null;
      } else if (data) {
        cache.current = {
          endpoint,
          genesisHash: data
        }
      }
      console.debug("chain cache updated:", cache.current);
    }
  }, [data, endpoint]);

  return cache.current?.genesisHash;
};
