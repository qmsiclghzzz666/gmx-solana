import { ComputeBudgetProgram, Connection, Signer, Transaction, TransactionInstruction, sendAndConfirmTransaction } from "@solana/web3.js";

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
        computeUnits?: number,
        computeUnitPrice?: number | bigint,
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
        const tx = computeUnits ?
            new Transaction()
                .add(ComputeBudgetProgram.setComputeUnitLimit({
                    units: computeUnits,
                }))
                .add(ComputeBudgetProgram.setComputeUnitPrice({
                    microLamports: computeUnitPrice ?? 1,
                }))
                .add(ix) :
            new Transaction().add(ix);
        return [await sendAndConfirmTransaction(connection, tx, signerList), output] as [string, U];
    }
};
