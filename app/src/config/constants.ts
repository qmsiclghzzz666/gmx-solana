import { getUnit } from "@/utils/number";
import { BN } from "@coral-xyz/anchor";

export const USD_DECIMALS = 20;
export const GM_DECIMALS = 9;
export const BN_ZERO = new BN(0);
export const MAX_SIGNED_USD = new BN("170141183460469231731687303715884105727");
export const MIN_SIGNED_USD = new BN("-170141183460469231731687303715884105728");
export const ONE_USD = getUnit(USD_DECIMALS);
