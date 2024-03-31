import { setProvider, AnchorProvider } from "@coral-xyz/anchor";

export const provider = AnchorProvider.env();
export const isDevNet = provider.connection.rpcEndpoint == "https://api.devnet.solana.com";

setProvider(provider);

export * as anchor from "@coral-xyz/anchor";
