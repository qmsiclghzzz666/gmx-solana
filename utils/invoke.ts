import { ComputeBudgetProgram, Connection, Signer, Transaction, TransactionInstruction, sendAndConfirmTransaction } from "@solana/web3.js";

type ParamsWithSigners<T, S extends PropertyKey[]> = {
    [P in keyof T]: P extends S[number] ? Signer : T[P];
};

export const makeInvoke = <
    T extends Record<string, unknown>,
    S extends (keyof T & string)[],
>(
    makeInstruction: (params: T) => Promise<TransactionInstruction>,
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
        const ix = await makeInstruction(originalParams as T);
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
        return await sendAndConfirmTransaction(connection, tx, signerList);
    }
};
