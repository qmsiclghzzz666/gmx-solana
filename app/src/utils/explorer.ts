import { DEFAULT_CLUSTER } from "@/config/env";
import { Address } from "@coral-xyz/anchor";

export const getTransactionUrl = (signature: string, cluster: string = DEFAULT_CLUSTER) => {
  return `https://explorer.solana.com/tx/${signature}?cluster=${cluster}`;
};

export const getAddressUrl = (address: Address, cluster: string = DEFAULT_CLUSTER) => {
  return `https://explorer.solana.com/address/${address.toString()}?cluster=${cluster}`;
};
