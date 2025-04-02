import * as anchor from "@coral-xyz/anchor";
import { Program, Wallet } from "@coral-xyz/anchor";
import { BravenciaProgram } from "../target/types/bravencia_program";
// import { sleep } from '@coral-xyz/anchor';

// Add 500ms-1s delay between critical requests
import { assert } from "chai";
import {
	createMint,
	getOrCreateAssociatedTokenAccount,
	mintTo,
	TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
	PublicKey,
	Keypair,
	SystemProgram,
	SYSVAR_RENT_PUBKEY,
  LAMPORTS_PER_SOL 
} from "@solana/web3.js";

describe("bravencia_program", () => {
	// Configure the client to use the local cluster.
	const provider = anchor.AnchorProvider.env();
	anchor.setProvider(provider);
	// Get the payer properly
	const payer = (provider.wallet as Wallet).payer;

	const program = anchor.workspace
		.BravenciaProgram as Program<BravenciaProgram>;

	// Test accounts
	let adminWallet: Keypair;
	let userWallet: Keypair;
	let usdcMint: PublicKey;
	let userUsdcAccount: PublicKey;
	let adminUsdcAccount: PublicKey;

	// Mock Chainlink accounts
	const chainlinkProgramId = new PublicKey(
		"HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny"
	);
	const solUsdFeed = new PublicKey(
		"HgTtcbcmp5BeThax5AU8vg4VwK79qAvAKKFMs8txMLW6"
	);

	// Mock Chainlink data
	const mockSolPrice = 100 * 10 ** 8; // $100 with 8 decimals
	let mockChainlinkAccount: Keypair;

	before(async () => {
		// Generate test wallets
		adminWallet = Keypair.generate();
		userWallet = Keypair.generate();

		// Fund wallets using provider's payer
		const fundAdminTx = await provider.connection.requestAirdrop(
			adminWallet.publicKey,
			100 * anchor.web3.LAMPORTS_PER_SOL
		);
		await provider.connection.confirmTransaction(fundAdminTx);
		const fundUserTx = await provider.connection.requestAirdrop(
			userWallet.publicKey,
			100 * anchor.web3.LAMPORTS_PER_SOL
		);
		await provider.connection.confirmTransaction(fundUserTx);

		// Create USDC mint (mock)
		usdcMint = await createMint(
			provider.connection,
			payer, // Payer
			payer.publicKey, // Mint authority
			null, // Freeze authority
			6 // Decimals
		);

		// Create user and admin USDC accounts
		const userUsdc = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer, // Use the payer keypair
      usdcMint,
      userWallet.publicKey
    );
    userUsdcAccount = userUsdc.address;
		const adminUsdc = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      usdcMint,
      adminWallet.publicKey
    );
    adminUsdcAccount = adminUsdc.address;

    const balance = await provider.connection.getTokenAccountBalance(userUsdcAccount);
    console.log(balance.value.amount);

		// Mint 1000 USDC to user
    await mintTo(
      provider.connection,
      payer,
      usdcMint,
      userUsdcAccount,
      payer,
      1000 * 10**6
    );

		// Setup mock Chainlink account
		mockChainlinkAccount = Keypair.generate();
		// We would normally initialize this with mock data, but simplified for this example
	});

  describe("USDC deposits", () => {
    it ('should deposit USDC and emit event', async () => {
      const depositAmount = 10 * 10**6; // 10USDC

      // Initial balances
      const initialUserBalance = await provider.connection.getTokenAccountBalance(userUsdcAccount);
      const initialAdminBalance = await provider.connection.getTokenAccountBalance(adminUsdcAccount);
      await program.methods.depositUsdc(new anchor.BN(depositAmount))
      .accounts({
        userUsdcAccount,
        adminUsdcAccount,
        userWallet: userWallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userWallet])
      .rpc();

      // Verify balances
      const updatedUserBalance = await provider.connection.getTokenAccountBalance(userUsdcAccount);
      const updatedAdminBalance = await provider.connection.getTokenAccountBalance(adminUsdcAccount);

      assert.equal(
        parseInt(initialUserBalance.value.amount) - parseInt(updatedUserBalance.value.amount),
        depositAmount
      );
      assert.equal(
        parseInt(updatedAdminBalance.value.amount) - parseInt(initialAdminBalance.value.amount),
        depositAmount
      );
    })
  })

  // describe("SOL deposits", () => {
  //   it("should deposit SOL", async () => {
  //     const depositAmount = 0.1 * LAMPORTS_PER_SOL; // 0.1 sol
  //      // Initial balances
  //      const initialUserSol = await provider.connection.getBalance(userWallet.publicKey);
  //      const initialAdminSol = await provider.connection.getBalance(adminWallet.publicKey);

  //      // Deposit
  //     await program.methods.depositSol(new anchor.BN(depositAmount))
  //     .accounts({
  //       userWallet: userWallet.publicKey,
  //       adminWallet: adminWallet.publicKey,
  //       chainlinkFeed: solUsdFeed,
  //       chainlinkProgram: chainlinkProgramId,
  //     })
  //     .signers([userWallet])
  //     .rpc();

  //     // Verify SOL transfer (allow for fees)
  //     const updatedUserSol = await provider.connection.getBalance(userWallet.publicKey);
  //     assert.isAtMost(initialUserSol - updatedUserSol, depositAmount * 1.1); // Allow 10% fee buffer
  //   })
  // })
});
