import { DEFAULT_CLUSTER } from "@/config/env";

export const getTransactionUrl = (signature: string, cluster: string = DEFAULT_CLUSTER) => {
  return `https://explorer.solana.com/tx/${signature}?cluster=${cluster}`;
};
