import { AnchorError, Idl, Program } from "@coral-xyz/anchor";
import { ComputeBudgetProgram, ConfirmOptions, SendTransactionError, Signer, Transaction, TransactionInstruction, sendAndConfirmTransaction } from "@solana/web3.js";

type ParamsWithSigners<T, S extends PropertyKey[]> = {
    [P in keyof T]: P extends S[number] ? Signer : T[P];
};

export type IxWithOutput<T> = [TransactionInstruction, T];

export const makeInvoke = <
    IDL extends Idl,
    T extends Record<string, unknown>,
    S extends (keyof T & string)[],
    U = undefined,
>(
    makeInstruction: (program: Program<IDL>, params: T) => Promise<TransactionInstruction | IxWithOutput<U>>,
    signers: S,
    defaultSignByProvider?: boolean,
) => {
    return async (
        program: Program<IDL>,
        params: ParamsWithSigners<T, S>,
        options?: ConfirmOptions & {
            signByProvider?: boolean,
            computeUnits?: number,
            computeUnitPrice?: number | bigint,
        }
    ) => {
        const originalParams: Partial<T> = { ...params } as any;
        const signerList: Signer[] = [];
        signers.forEach((signerField) => {
            const signer = params[signerField] as Signer;
            originalParams[signerField] = signer.publicKey as T[keyof T & string];
            signerList.push(signer);
        });
        const result = await makeInstruction(program, originalParams as T);
        let ix: TransactionInstruction;
        let output: U | undefined = undefined;
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
            const signByProvider = options?.signByProvider ?? defaultSignByProvider;
            if (signByProvider && program.provider.sendAndConfirm) {
                const hash = await program.provider.connection.getLatestBlockhash();
                tx.recentBlockhash = hash.blockhash;
                return [await program.provider.sendAndConfirm(tx, signerList, {
                    skipPreflight: options?.skipPreflight ?? false,
                }), output] as [string, U];
            } else {
                return [await sendAndConfirmTransaction(program.provider.connection, tx, signerList, options), output] as [string, U];
            }
        } catch (error) {
            if ((error as SendTransactionError).logs) {
                const anchorError = AnchorError.parse((error as SendTransactionError).logs ?? []);
                if (anchorError) {
                    throw anchorError;
                }
            }
            throw error;
        }
    }
};
