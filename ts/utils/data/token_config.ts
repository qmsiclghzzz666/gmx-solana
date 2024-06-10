import { Keypair, PublicKey, Signer } from "@solana/web3.js";
import { dataStore } from "./program";
import { utils } from "@coral-xyz/anchor";
import { DataStoreProgram, PriceProvider, makeInvoke, toBN } from "gmsol";
import { update } from "lodash";

// Token Config map seed.
export const TOKEN_CONFIG_MAP_SEED = utils.bytes.utf8.encode("token_config_map");

const hexStringToPublicKey = (hex: string) => {
    const decoded = utils.bytes.hex.decode(hex);
    return new PublicKey(decoded);
};

export interface FeedsOptions {
    pythFeedId?: string,
    chainlinkFeed?: PublicKey,
    pythDevFeed?: PublicKey,
    expectedProvider?: PriceProvider,
}

export const toggleTokenConfig = async (
    authority: Signer,
    store: PublicKey,
    tokenMap: PublicKey,
    token: PublicKey,
    enable: boolean,
) => {
    await dataStore.methods.toggleTokenConfig(token, enable).accountsPartial({
        authority: authority.publicKey,
        store,
        tokenMap,
    }).signers([authority]).rpc();
};

export const setExpectedProvider = async (
    authority: Signer,
    store: PublicKey,
    tokenMap: PublicKey,
    token: PublicKey,
    provider: PriceProvider,
) => {
    await dataStore.methods.setExpectedProvider(token, provider).accountsPartial({
        authority: authority.publicKey,
        store,
        tokenMap,
    }).signers([authority]).rpc();
};

export interface TokenConfig {
    enabled: boolean,
    heartbeatDuration: number,
    tokenDecimals: number,
    precision: number,
    feeds: PublicKey[],
    expectedProvider: number,
}

export const makeInitializeTokenMapInstruction = async (
    program: DataStoreProgram,
    { payer, store, tokenMap }: {
        payer: PublicKey,
        store: PublicKey,
        tokenMap: PublicKey,
    }
) => {
    return await program.methods.initializeTokenMap().accounts({
        payer,
        store,
        tokenMap,
    }).instruction();
}

export const invokeInitializeTokenMap = makeInvoke(makeInitializeTokenMapInstruction, ["payer", "tokenMap"]);

export const makePushToTokenMapInstruction = async (
    program: DataStoreProgram,
    {
        authority,
        store,
        tokenMap,
        token,
        heartbeatDuration,
        precision,
        feeds: {
            pythFeedId,
            chainlinkFeed,
            pythDevFeed,
            expectedProvider,
        },
        enable = true,
        update = false,
    }: {
        authority: PublicKey,
        store: PublicKey,
        tokenMap: PublicKey,
        token: PublicKey,
        heartbeatDuration: number,
        precision: number,
        feeds: FeedsOptions,
        enable?: boolean,
        update?: boolean,
    }
) => {
    return await program.methods.pushToTokenMap({
        heartbeatDuration,
        precision,
        feeds: [
            pythFeedId ? hexStringToPublicKey(pythFeedId) : PublicKey.default,
            chainlinkFeed ?? PublicKey.default,
            pythDevFeed ?? PublicKey.default,
            PublicKey.default,
        ],
        expectedProvider,
    }, enable, !update).accountsPartial({
        authority,
        store,
        tokenMap,
        token,
    }).instruction();
};

export const invokePushToTokenMap = makeInvoke(makePushToTokenMapInstruction, ["authority"]);

export const makePushToTokenMapSyntheticInstruction = async (
    program: DataStoreProgram,
    {
        authority,
        store,
        tokenMap,
        token,
        tokenDecimals,
        heartbeatDuration,
        precision,
        feeds: {
            pythFeedId,
            chainlinkFeed,
            pythDevFeed,
            expectedProvider,
        },
        enable = true,
        update = false,
    }: {
        authority: PublicKey,
        store: PublicKey,
        tokenMap: PublicKey,
        token: PublicKey,
        tokenDecimals: number,
        heartbeatDuration: number,
        precision: number,
        feeds: FeedsOptions,
        enable?: boolean,
        update?: boolean,
    }
) => {
    return await program.methods.pushToTokenMapSynthetic(
        token,
        tokenDecimals,
        {
            heartbeatDuration,
            precision,
            feeds: [
                pythFeedId ? hexStringToPublicKey(pythFeedId) : PublicKey.default,
                chainlinkFeed ?? PublicKey.default,
                pythDevFeed ?? PublicKey.default,
                PublicKey.default,
            ],
            expectedProvider,
        }, enable, !update).accountsPartial({
            authority,
            store,
            tokenMap,
        }).instruction();
};

export const invokePushToTokenMapSynthetic = makeInvoke(makePushToTokenMapSyntheticInstruction, ["authority"]);
