import { useConnection } from "@solana/wallet-adapter-react";

export const useEndpointName = () => {
  const connection = useConnection();
  return connection.connection.rpcEndpoint;
};
