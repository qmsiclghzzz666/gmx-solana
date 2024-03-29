import { BN } from "@coral-xyz/anchor";

/**
 * Converts a value of type number or bigint to a BN object
 * @param {number | bigint} value - The value to convert
 * @returns {BN} - The resulting BN object
 */
export function toBN(value: number | bigint) {
    if (typeof value === 'number') {
        return new BN(value);
    } else if (typeof value === 'bigint') {
        return new BN(value.toString(), 10);
    } else {
        throw new TypeError('The input value must be a number or a bigint.');
    }
}
