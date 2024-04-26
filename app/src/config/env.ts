import { WalletAdapterNetwork } from "@solana/wallet-adapter-base";

export const IS_TOUCH = "ontouchstart" in window;
export const DEFAULT_CLUSTER = WalletAdapterNetwork.Devnet;
export const IS_DEVELOPMENT = import.meta.env.MODE === 'development';
