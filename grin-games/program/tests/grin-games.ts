import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GrinGames } from "../target/types/grin_games";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo } from "@solana/spl-token";
import { assert } from "chai";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram } from '@solana/web3.js';

describe("grin-games", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.GrinGames as Program<GrinGames>;
  
  let mint: PublicKey;
  let player1TokenAccount: PublicKey;
  let player2TokenAccount: PublicKey;
  let gameTokenAccount: PublicKey;
  let player2: Keypair;

  before(async () => {
    // Create second player keypair
    player2 = Keypair.generate();
    
    // Airdrop SOL to player2
    const signature = await provider.connection.requestAirdrop(
      player2.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);

    // Create mint (representing GRIN token)
    mint = await createMint(
      provider.connection,
      (provider.wallet as anchor.Wallet).keypair,
      provider.wallet.publicKey,
      null,
      9
    );

    // Create token accounts
    player1TokenAccount = await createAccount(
      provider.connection,
      (provider.wallet as anchor.Wallet).keypair,
      mint,
      provider.wallet.publicKey
    );

    player2TokenAccount = await createAccount(
      provider.connection,
      (provider.wallet as anchor.Wallet).keypair,
      mint,
      player2.publicKey
    );

    gameTokenAccount = await createAccount(
      provider.connection,
      (provider.wallet as anchor.Wallet).keypair,
      mint,
      provider.wallet.publicKey
    );

    // Mint tokens to players
    await mintTo(
      provider.connection,
      (provider.wallet as anchor.Wallet).keypair,
      mint,
      player1TokenAccount,
      provider.wallet.publicKey,
      1000000000
    );

    await mintTo(
      provider.connection,
      (provider.wallet as anchor.Wallet).keypair,
      mint,
      player2TokenAccount,
      provider.wallet.publicKey,
      1000000000
    );
  });

  it("Initialize a game", async () => {
    const [gameAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("game"),
        provider.wallet.publicKey.toBuffer(),
      ],
      program.programId
    );

    const betAmount = new anchor.BN(100000000);

    try {
      await program.methods
        .initializeGame(betAmount)
        .accounts({
          game: gameAccount,
          player: provider.wallet.publicKey,
          playerTokenAccount: player1TokenAccount,
          gameTokenAccount: gameTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const gameState = await program.account.gameState.fetch(gameAccount);
      assert.ok(gameState.isActive);
      assert.equal(gameState.betAmount.toNumber(), betAmount.toNumber());
      assert.deepEqual(gameState.player1.toBase58(), provider.wallet.publicKey.toBase58());
    } catch (error) {
      console.error("Error:", error);
      throw error;
    }
  });

  it("Join a game", async () => {
    const [gameAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("game"),
        provider.wallet.publicKey.toBuffer(),
      ],
      program.programId
    );

    try {
      await program.methods
        .joinGame()
        .accounts({
          game: gameAccount,
          player: player2.publicKey,
          player1: provider.wallet.publicKey,
          playerTokenAccount: player2TokenAccount,
          player1TokenAccount: player1TokenAccount,
          gameTokenAccount: gameTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([player2])
        .rpc();

      const gameState = await program.account.gameState.fetch(gameAccount);
      assert.ok(!gameState.isActive); // Game should be completed after join
      assert.ok(gameState.player2); // Player 2 should be set
    } catch (error) {
      console.error("Error:", error);
      throw error;
    }
  });
});
