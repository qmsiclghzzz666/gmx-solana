import { PublicKey } from "@solana/web3.js";
import { expect, getAddresses, getUsers } from "../../utils/fixtures";
import { invokeCloseEmptyClaimableAccount, invokeUseClaimableAccount } from "../../utils/data/token";
import { dataStore } from "../../utils/data";

describe("data store: Token", () => {
    const { signer0, user0 } = getUsers();

    let dataStoreAddress: PublicKey;
    let fakeTokenMint: PublicKey;
    before("init", async () => {
        ({ dataStoreAddress, fakeTokenMint } = await getAddresses());
    });

    it("prepare a claimable account", async () => {
        const [signature1, timestamp] = await invokeUseClaimableAccount(dataStore, {
            authority: signer0,
            store: dataStoreAddress,
            user: user0.publicKey,
            mint: fakeTokenMint,
        });
        console.log(`prepared a claimable account at tx ${signature1}`);

        const [signature2] = await invokeUseClaimableAccount(dataStore, {
            authority: signer0,
            store: dataStoreAddress,
            user: user0.publicKey,
            mint: fakeTokenMint,
            timestamp,
        });
        console.log(`prepared the same claimable account at tx ${signature2}`);

        const signature3 = await invokeCloseEmptyClaimableAccount(dataStore, {
            authority: signer0,
            store: dataStoreAddress,
            user: user0.publicKey,
            mint: fakeTokenMint,
            timestamp,
        });

        console.log(`closed the claimable account at tx ${signature3}`);
    });
});
