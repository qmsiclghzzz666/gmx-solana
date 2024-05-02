import { BN } from "@coral-xyz/anchor";

export function getPositiveOrNegativeClass(value?: BN, zeroValue: "" | "text-red" | "text-green" = ""): string {
  if (!value) {
    return "";
  }
  return value.isZero() ? zeroValue : value.isNeg() ? "text-red" : "text-green";
}
