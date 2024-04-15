import { PublicKey } from "@solana/web3.js";
import { getAddresses, getMarkets, getProvider, getUsers } from "../../utils/fixtures";
import { invokeCreateOrder } from "../../utils/exchange";

describe("exchange: order", () => {
    const provider = getProvider();
    const { signer0, user0 } = getUsers();

    let dataStoreAddress: PublicKey;
    let user0FakeTokenAccount: PublicKey;
    let user0UsdGTokenAccount: PublicKey;
    let GMFakeFakeUsdG: PublicKey;

    before(async () => {
        ({
            dataStoreAddress,
            user0FakeTokenAccount,
            user0UsdGTokenAccount,
        } = await getAddresses());
        ({ GMFakeFakeUsdG } = await getMarkets());
    });

    it("create an increase order", async () => {
        try {
            const [signature, order] = await invokeCreateOrder(provider.connection, {
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
            console.log(`order ${order} created at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }
    });
});
