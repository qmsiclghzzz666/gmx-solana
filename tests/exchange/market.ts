import { getAddresses, getTokenMints, getUsers } from "../../utils/fixtures";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createRolesPDA } from "../../utils/data";
import { createMarket } from "../../utils/exchange";

describe("exchange: market", () => {
    const { signer0 } = getUsers();
    const { BTC_TOKEN_MINT, SOL_TOKEN_MINT } = getTokenMints();

    const indexTokenMint = Keypair.generate().publicKey;
    const longTokenMint = BTC_TOKEN_MINT;
    const shortTokenMint = SOL_TOKEN_MINT;

    let dataStoreAddress: PublicKey;
    let roles: PublicKey;
    before(async () => {
        ({ dataStoreAddress } = await getAddresses());
        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
    });

    it("create market", async () => {
        await createMarket(signer0, dataStoreAddress, indexTokenMint, longTokenMint, shortTokenMint);
    });
});
