import { NATIVE_TOKEN_ADDRESS } from "@/config/tokens";
import { Tokens } from "./types";
import { PublicKey } from "@solana/web3.js";

export function getTokenData(tokensData?: Tokens, address?: PublicKey, convertTo?: "wrapped" | "native") {
  if (!address || !tokensData?.[address.toBase58()]) {
    return undefined;
  }

  const token = tokensData[address.toBase58()];

  if (convertTo === "wrapped" && token.isNative && token.wrappedAddress) {
    return tokensData[token.wrappedAddress.toBase58()];
  }

  if (convertTo === "native" && token.isWrapped) {
    return tokensData[NATIVE_TOKEN_ADDRESS.toBase58()];
  }

  return token;
}
