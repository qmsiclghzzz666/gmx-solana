import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { getAddresses, getMarkets, getPrograms, getUsers } from "../../utils/fixtures";
import { deposit, swap, withdraw, wrap } from "../../utils/exchange/action";
import { SOL_TOKEN_MINT } from "../../utils/token";
import { getAccount, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";

describe("exchange: Swap", () => {
    const { storeProgram, exchangeProgram } = getPrograms();
    const { signer0, user0 } = getUsers();

    let dataStoreAddress: PublicKey;
    let tokenMap: PublicKey;
    let GMFakeFakeUsdG: PublicKey;
    let GMWsolWsolUsdG: PublicKey;
    let oracleAddress: PublicKey;
    let fakeTokenMint: PublicKey;
    let usdGTokenMint: PublicKey;

    before(async () => {
        ({
            dataStoreAddress,
            oracleAddress,
            fakeTokenMint,
            usdGTokenMint,
        } = await getAddresses());
        ({ GMFakeFakeUsdG, GMWsolWsolUsdG } = await getMarkets());
        tokenMap = (await storeProgram.account.store.fetch(dataStoreAddress)).tokenMap;

        await wrap(exchangeProgram.provider, user0, 1.5 * LAMPORTS_PER_SOL);
    });

    it("deposit into the markets to be used", async () => {
        await deposit(
            exchangeProgram,
            user0,
            signer0,
            dataStoreAddress,
            tokenMap,
            oracleAddress,
            GMFakeFakeUsdG,
            fakeTokenMint,
            usdGTokenMint,
            1_000n * 1_000_000_000n,
            50_000_000n * 100_000_000n,
            {
                storeProgram,
            }
        );
        await deposit(
            exchangeProgram,
            user0,
            signer0,
            dataStoreAddress,
            tokenMap,
            oracleAddress,
            GMWsolWsolUsdG,
            SOL_TOKEN_MINT,
            usdGTokenMint,
            0,
            2_000_000n * 1_000_000_000n,
            {
                storeProgram,
            }
        );
    });

    it("swap for fake token with more USDG", async () => {
        await swap(
            storeProgram,
            exchangeProgram,
            user0,
            signer0,
            dataStoreAddress,
            tokenMap,
            oracleAddress,
            GMFakeFakeUsdG,
            fakeTokenMint,
            usdGTokenMint,
            1_000n * 1_000_000_000n,
            [GMFakeFakeUsdG],
            {}
        );
    });

    it("swap for fake token with some WSOL", async () => {
        await swap(
            storeProgram,
            exchangeProgram,
            user0,
            signer0,
            dataStoreAddress,
            tokenMap,
            oracleAddress,
            GMFakeFakeUsdG,
            fakeTokenMint,
            SOL_TOKEN_MINT,
            1 * LAMPORTS_PER_SOL,
            [
                GMWsolWsolUsdG,
                GMFakeFakeUsdG,
            ],
            {}
        );
    });

    it("withdraw all from WSOL balanced pool", async () => {
        let amount = (await getOrCreateAssociatedTokenAccount(exchangeProgram.provider.connection, user0, GMWsolWsolUsdG, user0.publicKey)).amount;

        await withdraw(
            exchangeProgram,
            user0,
            signer0,
            dataStoreAddress,
            tokenMap,
            oracleAddress,
            GMWsolWsolUsdG,
            amount,
            SOL_TOKEN_MINT,
            usdGTokenMint,
            {
                storeProgram,
            }
        );
    });
});
