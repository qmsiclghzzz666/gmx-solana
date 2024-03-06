import * as anchor from "@coral-xyz/anchor";
import { keyToSeed } from "./seed";
import { Oracle } from "../target/types/oracle";

export const oracle = anchor.workspace.Oracle as anchor.Program<Oracle>;

export const ORACLE_SEED = anchor.utils.bytes.utf8.encode("oracle");

export const createOraclePDA = (store: anchor.web3.PublicKey, key: string) => anchor.web3.PublicKey.findProgramAddressSync([
    ORACLE_SEED,
    store.toBytes(),
    keyToSeed(key),
], oracle.programId);

export const initializeOracle = async (signer: anchor.web3.Keypair, dataStoreAddress: anchor.web3.PublicKey, oracleKey: string) => {
    const [oraclePDA] = createOraclePDA(dataStoreAddress, oracleKey);

    // Initialize a oracle with the given key.
    try {
        const tx = await oracle.methods.initialize(oracleKey).accounts({
            authority: signer.publicKey,
            store: dataStoreAddress,
            oracle: oraclePDA,
        }).signers([signer]).rpc();
        console.log(`Initialized a new oracle ${oraclePDA.toBase58()} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to initialize a oracle with the given key:", error);
    }
};
