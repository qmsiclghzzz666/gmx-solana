import * as anchor from "@coral-xyz/anchor";
import { DataStore } from "../target/types/data_store";
import { keyToSeed } from "./seed";
import { EventManager } from "./event";
import { Keypair, PublicKey } from "@solana/web3.js";
import { BTC_FEED, BTC_TOKEN_MINT, SOL_FEED, SOL_TOKEN_MINT, USDC_FEED } from "./token";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

export const dataStore = anchor.workspace.DataStore as anchor.Program<DataStore>;

const encodeUtf8 = anchor.utils.bytes.utf8.encode;

// Data Store seed.
export const DATA_STORE_SEED = encodeUtf8("data_store");
// Roles seed.
export const ROLES_SEED = encodeUtf8("roles");
// Token Config seed.
export const TOKEN_CONFIG_SEED = encodeUtf8("token_config");
// Market seeds.
export const MARKET_SEED = encodeUtf8("market");
export const MARKET_SIGN_SEED = encodeUtf8("market_sign");
export const MARKET_TOKEN_MINT_SEED = encodeUtf8("market_token_mint");
export const MARKET_VAULT_SEED = encodeUtf8("market_vault");
// Oracle seed.
export const ORACLE_SEED = encodeUtf8("oracle");
// Nonce seed.
export const NONCE_SEED = encodeUtf8("nonce");
// Deposit seed.
export const DEPOSIT_SEED = encodeUtf8("deposit");

// Role keys.
export const CONTROLLER = "CONTROLLER";
export const MARKET_KEEPER = "MARKET_KEEPER";
export const ORDER_KEEPER = "ORDER_KEEPER";

export const createDataStorePDA = (key: string) => anchor.web3.PublicKey.findProgramAddressSync([
    DATA_STORE_SEED,
    keyToSeed(key),
], dataStore.programId);

export const createRolesPDA = (store: PublicKey, authority: PublicKey) => PublicKey.findProgramAddressSync([
    ROLES_SEED,
    store.toBytes(),
    authority.toBytes(),
], dataStore.programId);

export const createTokenConfigPDA = (store: anchor.web3.PublicKey, key: string) => anchor.web3.PublicKey.findProgramAddressSync([
    TOKEN_CONFIG_SEED,
    store.toBytes(),
    keyToSeed(key),
], dataStore.programId);

export const createMarketPDA = (store: PublicKey, marketToken: PublicKey) => PublicKey.findProgramAddressSync([
    MARKET_SEED,
    store.toBytes(),
    keyToSeed(marketToken.toBase58()),
], dataStore.programId);

export const createMarketTokenMintPDA = (
    store: PublicKey,
    indexTokenMint: PublicKey,
    longTokenMint: PublicKey,
    shortTokenMint: PublicKey,
) => PublicKey.findProgramAddressSync([
    MARKET_TOKEN_MINT_SEED,
    store.toBytes(),
    indexTokenMint.toBytes(),
    longTokenMint.toBytes(),
    shortTokenMint.toBytes(),
], dataStore.programId);

export const createMarketVaultPDA = (store: PublicKey, tokenMint: PublicKey, marketTokenMint?: PublicKey) => PublicKey.findProgramAddressSync([
    MARKET_VAULT_SEED,
    store.toBytes(),
    tokenMint.toBytes(),
    marketTokenMint?.toBytes() ?? new Uint8Array(),
], dataStore.programId);

export const getMarketSignPDA = () => PublicKey.findProgramAddressSync([MARKET_SIGN_SEED], dataStore.programId);

export const createOraclePDA = (store: PublicKey, index: number) => PublicKey.findProgramAddressSync([
    ORACLE_SEED,
    store.toBytes(),
    new Uint8Array([index]),
], dataStore.programId);

export const createNoncePDA = (store: PublicKey) => PublicKey.findProgramAddressSync([
    NONCE_SEED,
    store.toBytes(),
], dataStore.programId);

export const createDepositPDA = (store: PublicKey, user: PublicKey, nonce: Uint8Array) => PublicKey.findProgramAddressSync([
    DEPOSIT_SEED,
    store.toBytes(),
    user.toBytes(),
    nonce,
], dataStore.programId);

export const createMarketVault = async (provider: anchor.AnchorProvider, signer: Keypair, dataStoreAddress: PublicKey, mint: PublicKey) => {
    const [vault] = createMarketVaultPDA(dataStoreAddress, mint);
    const [roles] = createRolesPDA(dataStoreAddress, signer.publicKey);

    await dataStore.methods.initializeMarketVault(null).accounts({
        authority: signer.publicKey,
        onlyMarketKeeper: roles,
        store: dataStoreAddress,
        mint,
        vault,
        marketSign: getMarketSignPDA()[0],
        tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([signer]).rpc();
    return vault;
};

export const initializeDataStore = async (
    provider: anchor.AnchorProvider,
    eventManager: EventManager,
    signer: anchor.web3.Keypair,
    dataStoreKey: string,
    oracleIndex: number,
    fakeToken: PublicKey,
    usdG: PublicKey,
) => {
    const [dataStorePDA] = createDataStorePDA(dataStoreKey);
    const [rolesPDA] = createRolesPDA(dataStorePDA, provider.publicKey);
    const [signerRoles] = createRolesPDA(dataStorePDA, signer.publicKey);

    eventManager.subscribe(dataStore, "DataStoreInitEvent");
    eventManager.subscribe(dataStore, "TokenConfigChangeEvent");
    eventManager.subscribe(dataStore, "MarketChangeEvent");

    // Initialize a DataStore with the given key.
    try {
        const tx = await dataStore.methods.initialize(dataStoreKey).accounts({
            authority: provider.publicKey,
            dataStore: dataStorePDA,
            roles: rolesPDA,
        }).rpc();
        console.log(`Initialized a new data store account ${dataStorePDA.toBase58()} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to initialize a data store with the given key:", error);
    }

    // Initiliaze a roles account for `signer`.
    try {
        const tx = await dataStore.methods.initializeRoles().accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            roles: signerRoles,
        }).signers([signer]).rpc();
        console.log(`Initialized a roles account ${signerRoles} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to initialize roles account:", error);
    }

    // Enable the required roles and grant to `signer`.
    const enabled_roles = [CONTROLLER, MARKET_KEEPER, ORDER_KEEPER];
    for (let index = 0; index < enabled_roles.length; index++) {
        const role = enabled_roles[index];
        {
            const tx = await dataStore.methods.enableRole(role).accounts({
                authority: provider.publicKey,
                store: dataStorePDA,
                onlyAdmin: rolesPDA,
            }).rpc();
            console.log(`Enabled ${role} in tx: ${tx}`);
        }
        {
            const tx = await dataStore.methods.grantRole(signer.publicKey, role).accounts({
                authority: provider.publicKey,
                store: dataStorePDA,
                onlyAdmin: rolesPDA,
                userRoles: signerRoles,
            }).rpc();
            console.log(`Grant ${role} to signer in tx: ${tx}`);
        }
    }

    const HEARTBEAT = 120;

    // Insert BTC token config.
    try {
        const key = BTC_TOKEN_MINT.toBase58();
        const [tokenConfigPDA] = createTokenConfigPDA(dataStorePDA, key);
        const tx = await dataStore.methods.initializeTokenConfig(key, BTC_FEED, HEARTBEAT, 8, 2).accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            onlyController: createRolesPDA(dataStorePDA, signer.publicKey)[0],
            tokenConfig: tokenConfigPDA,
        }).signers([signer]).rpc();
        console.log(`Init a token config account ${tokenConfigPDA} for ${BTC_TOKEN_MINT} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to init the token config account", error);
    }

    // Insert SOL token config.
    try {
        const key = SOL_TOKEN_MINT.toBase58();
        const [tokenConfigPDA] = createTokenConfigPDA(dataStorePDA, key);
        const tx = await dataStore.methods.initializeTokenConfig(key, SOL_FEED, HEARTBEAT, 8, 4).accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            onlyController: createRolesPDA(dataStorePDA, signer.publicKey)[0],
            tokenConfig: tokenConfigPDA,
        }).signers([signer]).rpc();
        console.log(`Init a token config account ${tokenConfigPDA} for ${SOL_TOKEN_MINT} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to init the token config account", error);
    }

    // Insert FakeToken token config.
    try {
        const key = fakeToken.toBase58();
        const [tokenConfigPDA] = createTokenConfigPDA(dataStorePDA, key);
        const tx = await dataStore.methods.initializeTokenConfig(key, BTC_FEED, HEARTBEAT, 9, 2).accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            onlyController: createRolesPDA(dataStorePDA, signer.publicKey)[0],
            tokenConfig: tokenConfigPDA,
        }).signers([signer]).rpc();
        console.log(`Init a token config account ${tokenConfigPDA} for ${fakeToken} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to init the token config account", error);
    }

    // Insert UsdG token config.
    try {
        const key = usdG.toBase58();
        const [tokenConfigPDA] = createTokenConfigPDA(dataStorePDA, key);
        const tx = await dataStore.methods.initializeTokenConfig(key, USDC_FEED, HEARTBEAT, 8, 4).accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            onlyController: createRolesPDA(dataStorePDA, signer.publicKey)[0],
            tokenConfig: tokenConfigPDA,
        }).signers([signer]).rpc();
        console.log(`Init a token config account ${tokenConfigPDA} for ${usdG} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to init the token config account", error);
    }

    // Init an oracle.
    try {
        const [oraclePDA] = createOraclePDA(dataStorePDA, oracleIndex);
        const tx = await dataStore.methods.initializeOracle(oracleIndex).accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            onlyController: signerRoles,
            oracle: oraclePDA,
        }).signers([signer]).rpc();
        console.log(`Inited an oracle account ${oraclePDA} in tx: ${tx}`);
    } catch (error) {
        console.warn(`Failed to init an oracle account with index ${oracleIndex}:`, error);
    }

    // Init a nonce.
    try {
        const [noncePDA] = createNoncePDA(dataStorePDA);
        const tx = await dataStore.methods.initializeNonce().accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            onlyController: signerRoles,
            nonce: noncePDA,
        }).signers([signer]).rpc();
        console.log(`Inited a nonce account ${noncePDA} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to init a nonce account", error);
    }
};
