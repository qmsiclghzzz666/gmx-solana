import { getUnit } from "@/utils/number";
import { BN } from "@coral-xyz/anchor";

export const USD_DECIMALS = 20;
export const GM_DECIMALS = 9;
export const BN_ZERO = new BN(0);
export const BN_ONE = new BN(1);
export const MAX_SIGNED_USD = new BN("170141183460469231731687303715884105727");
export const MIN_SIGNED_USD = new BN("-170141183460469231731687303715884105728");
export const ONE_USD = getUnit(USD_DECIMALS);

export const ESTIMATED_EXECUTION_FEE = new BN(45000);
export const DEFAULT_RENT_EXEMPT_FEE_FOR_ZERO = new BN(890880);
