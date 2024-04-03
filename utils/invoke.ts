import { AnchorError } from "@coral-xyz/anchor";
import { ComputeBudgetProgram, Connection, SendTransactionError, Signer, Transaction, TransactionInstruction, sendAndConfirmTransaction } from "@solana/web3.js";

type ParamsWithSigners<T, S extends PropertyKey[]> = {
    [P in keyof T]: P extends S[number] ? Signer : T[P];
};

export type IxWithOutput<T> = [TransactionInstruction, T];

export const makeInvoke = <
    T extends Record<string, unknown>,
    S extends (keyof T & string)[],
    U = undefined,
>(
    makeInstruction: (params: T) => Promise<TransactionInstruction | IxWithOutput<U>>,
    signers: S,
) => {
    return async (
        connection: Connection,
        params: ParamsWithSigners<T, S>,
        options?: {
            computeUnits?: number,
            computeUnitPrice?: number | bigint,
            skipPreflight?: boolean,
        }
    ) => {
        const originalParams: Partial<T> = { ...params } as any;
        const signerList = [];
        signers.forEach((signerField) => {
            const signer = params[signerField] as Signer;
            originalParams[signerField] = signer.publicKey as T[keyof T & string];
            signerList.push(signer);
        });
        const result = await makeInstruction(originalParams as T);
        let ix: TransactionInstruction;
        let output: U;
        if ((result as TransactionInstruction).programId != undefined) {
            ix = result as TransactionInstruction;
        } else {
            ([ix, output] = result as IxWithOutput<U>);
        }
        const tx = options?.computeUnits ?
            new Transaction()
                .add(ComputeBudgetProgram.setComputeUnitLimit({
                    units: options.computeUnits,
                }))
                .add(ComputeBudgetProgram.setComputeUnitPrice({
                    microLamports: options.computeUnitPrice ?? 1,
                }))
                .add(ix) :
            new Transaction().add(ix);
        try {
            return [await sendAndConfirmTransaction(connection, tx, signerList, {
                skipPreflight: options?.skipPreflight ?? false,
            }), output] as [string, U];
        } catch (error) {
            if ((error as SendTransactionError).logs) {
                const anchorError = AnchorError.parse(error.logs);
                if (anchorError) {
                    throw anchorError;
                }
            }
            throw error;
        }
    }
};
