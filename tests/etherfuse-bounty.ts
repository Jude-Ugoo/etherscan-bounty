import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EtherfuseBounty } from "../target/types/etherfuse_bounty";
import { PublicKey } from "@solana/web3.js";

describe("etherfuse-bounty", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.EtherfuseBounty as Program<EtherfuseBounty>;

  // Metaplex Constants
  const METADATA_SEED = "metadata";
  const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );

  // Constants from program
  const MINT_SEED = "stablecoin_mint";

  // Data from our tests
  const user = provider.wallet.publicKey;

  const metadata = {
    name: "COUNTRYYYY",
    symbol: "CTR",
    uri: "https://5vfxc4tr6xoy23qefqbj4qx2adzkzapneebanhcalf7myvn5gzja.arweave.net/7UtxcnH13Y1uBCwCnkL6APKsge0hAgacQFl-zFW9NlI",
    decimals: 9,
  };

  const mintAmount = 1000;
  const [mint] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(MINT_SEED)],
    program.programId
  );

  const [metadataAddress] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(METADATA_SEED),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );

  it("initialize", async () => {
    const info = await provider.connection.getAccountInfo(mint);

    if (info) {
      return; //? Do not attempt to initialize if already initialized
    }
    console.log("Mint not found. Attempting to initialize.");

    const context = {
      metadata: metadataAddress,
      mint,
      user,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      tokemMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
    };

    const tx = await program.methods
      .initializeToken(metadata)
      .accounts(context)
      .signers([])
      .rpc();
  });

  it("mint tokens", async () => {
    const destination = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: user,
    });
    console.log("Destination ==> ", destination);

    let initialBalance: number;
    try {
      const balance = await provider.connection.getTokenAccountBalance(
        destination
      );
      initialBalance = balance.value.uiAmount;
    } catch {
      //? If the token account doesn't exist, set the initial balance to 0
      initialBalance = 0;
    }

    const context = {
      mint,
      destination,
      user,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
    };

    const tx = await program.methods
      .mintStablecoin(new anchor.BN(mintAmount * 10 ** metadata.decimals))
      .accounts(context)
      .signers([])
      .rpc();

    console.log("Transaction ====> ", tx);

    console.log("You are the winner");
  });

  it("burn tokens", async () => {
    const destination = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: user,
    });

    //? Ensure token account exists and fetch initial balance
    let initialBalance: number;

    try {
      const balance = await provider.connection.getTokenAccountBalance(
        destination
      );
      initialBalance = balance.value.uiAmount || 0;
    } catch (error) {
      //? If the token account doesn't exist, set the initial balance to 0
      initialBalance = 0;
    }
    console.log("Initial Balance ==> ", initialBalance);

    const context = {
      mint,
      destination,
      user,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
    };

    if (initialBalance === 0) {
      console.log("Minting tokens for testing burn...");

      //? Mint tokens if no balance
      await program.methods
        .mintStablecoin(new anchor.BN(mintAmount * 10 ** metadata.decimals))
        .accounts(context)
        .signers([])
        .rpc();
    }

    //? Fetch updated balance after minting
    const updatedBalance = await provider.connection.getTokenAccountBalance(
      destination
    );
    console.log("Updated Balance ==> ", updatedBalance.value.uiAmount);

    //? Set burn quantity
    const burnQuantity = 500;

    //? Burn Tokens
    const tx = await program.methods
      .burnToken(new anchor.BN(burnQuantity * 10 ** metadata.decimals))
      .accounts(context)
      .signers([])
      .rpc();

    console.log("Burn Transaction Signature ==> ", tx);

    //? Fetch balance after burn
    const finalBalance = await provider.connection.getTokenAccountBalance(
      destination
    );
    console.log("Final Balance ==> ", finalBalance.value.uiAmount);

    // Assertions
    const expectedBalance = updatedBalance.value.uiAmount - burnQuantity;
    console.assert(
      finalBalance.value.uiAmount === expectedBalance,
      `Expected ${expectedBalance}, but got ${finalBalance.value.uiAmount}`
    );

    console.log("Token burn test completed successfully.");
  });
});
