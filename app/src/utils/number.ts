import { BN } from "@coral-xyz/anchor";

export function getUnit(decimals: number) {
  return (new BN(10)).pow(new BN(decimals));
}

export function expandDecimals(n: BN, decimals: number) {
  return n.mul(getUnit(decimals));
}
