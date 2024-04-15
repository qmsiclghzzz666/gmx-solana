import { PublicKey } from "@solana/web3.js";
import { getAddresses, getMarkets, getProvider, getUsers } from "../../utils/fixtures";
import { invokeCreateOrder, invokeExecuteOrder } from "../../utils/exchange";

describe("exchange: order", () => {
    const provider = getProvider();
    const { signer0, user0 } = getUsers();

    let dataStoreAddress: PublicKey;
    let oracleAddress: PublicKey;
    let user0FakeTokenAccount: PublicKey;
    let user0UsdGTokenAccount: PublicKey;
    let GMFakeFakeUsdG: PublicKey;

    before(async () => {
        ({
            dataStoreAddress,
            oracleAddress,
            user0FakeTokenAccount,
            user0UsdGTokenAccount,
        } = await getAddresses());
        ({ GMFakeFakeUsdG } = await getMarkets());
    });

    it("increase and decrease position", async () => {
        // Increase position.
        let increaseOrder: PublicKey;
        try {
            const [signature, address] = await invokeCreateOrder(provider.connection, {
                store: dataStoreAddress,
                payer: user0,
                orderType: "marketIncrease",
                marketToken: GMFakeFakeUsdG,
                isCollateralTokenLong: false,
                initialCollateralDeltaAmount: 2_000_000,
                isLong: true,
                sizeDeltaUsd: 200_000_000_000_000_000_000n,
                fromTokenAccount: user0FakeTokenAccount,
                options: {
                    swapPath: [
                        GMFakeFakeUsdG
                    ],
                }
            });
            increaseOrder = address;
            console.log(`order ${increaseOrder} created at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }
        try {
            const signature = await invokeExecuteOrder(provider.connection, {
                authority: signer0,
                store: dataStoreAddress,
                oracle: oracleAddress,
                order: increaseOrder,
            }, {
                computeUnits: 400_000,
            });
            console.log(`order ${increaseOrder} executed at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }

        // Decrease position.
        let decreaseOrder: PublicKey;
        try {
            const [signature, address] = await invokeCreateOrder(provider.connection, {
                store: dataStoreAddress,
                payer: user0,
                orderType: "marketDecrease",
                marketToken: GMFakeFakeUsdG,
                isCollateralTokenLong: false,
                initialCollateralDeltaAmount: 0,
                isLong: true,
                sizeDeltaUsd: 100_000_000_000_000_000_000n,
                toTokenAccount: user0FakeTokenAccount,
                secondaryTokenAccount: user0FakeTokenAccount,
                options: {
                    swapPath: [
                        GMFakeFakeUsdG
                    ],
                }
            });
            decreaseOrder = address;
            console.log(`order ${decreaseOrder} created at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }
        try {
            const signature = await invokeExecuteOrder(provider.connection, {
                authority: signer0,
                store: dataStoreAddress,
                oracle: oracleAddress,
                order: decreaseOrder,
            }, {
                computeUnits: 400_000,
            });
            console.log(`order ${decreaseOrder} executed at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }
    });
});
