import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SentinalVault } from "../target/types/sentinal_vault";
import { expect } from "chai";

describe("sentinal-vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .SentinalVault as Program<SentinalVault>;

  const user = provider.wallet.publicKey;

  const cooldownSeconds = 1; // Short cooldown for testing
  const inactivityWindowSeconds = 4; // Short inactivity window for testing

  const [vaultState] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state"), user.toBuffer()],
    program.programId
  );

  const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), vaultState.toBuffer()],
    program.programId
  );

  const sleep = (ms: number) =>
    new Promise((resolve) => setTimeout(resolve, ms));

  const assertAnchorError = (err: any, expected: string) => {
    const msg =
      err?.error?.errorMessage ||
      err?.message ||
      "";
    expect(msg.toLowerCase()).to.include(expected.toLowerCase());
  };

  it("Initializes the vault", async () => {
    await program.methods
      .initialize(
        new anchor.BN(cooldownSeconds),
        new anchor.BN(inactivityWindowSeconds)
      )
      .accountsStrict({
        user,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const state = await program.account.vaultState.fetch(vaultState);

    expect(state.owner.toBase58()).to.equal(user.toBase58());
    expect(state.cooldownSeconds.toNumber()).to.equal(cooldownSeconds);
    expect(state.inactivityWindowSeconds.toNumber()).to.equal(
      inactivityWindowSeconds
    );
  });

  it("Deposits SOL into the vault", async () => {
    const amount = 1_000_000;

    await program.methods
      .deposit(new anchor.BN(amount))
      .accountsStrict({
        user,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const state = await program.account.vaultState.fetch(vaultState);
    expect(state.totalDeposited.toNumber()).to.equal(amount);
  });

  it("Fails to withdraw without check-in (after inactivity window)", async () => {
  await sleep((inactivityWindowSeconds + 1) * 1000);

  try {
    await program.methods
      .withdraw(new anchor.BN(100_000))
      .accountsStrict({
        user,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    expect.fail("Expected inactivity failure");
  } catch (err) {
    assertAnchorError(err, "inactive");
  }
});

  it("Check-in succeeds", async () => {
    await program.methods
      .checkIn()
      .accounts({
        owner: user,
        vaultState,
      })
      .rpc();

    const state = await program.account.vaultState.fetch(vaultState);
    expect(state.lastCheckIn.toNumber()).to.be.greaterThan(0);
  });

  it("Withdraw succeeds after cooldown", async () => {
    await sleep((cooldownSeconds + 1) * 1000);

    await program.methods
      .withdraw(new anchor.BN(500_000))
      .accountsStrict({
        user,
        vaultState,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const state = await program.account.vaultState.fetch(vaultState);
    expect(state.totalWithdrawn.toNumber()).to.equal(500_000);
  });

  it("Fails to withdraw during cooldown", async () => {
    try {
      await program.methods
        .withdraw(new anchor.BN(100_000))
        .accountsStrict({
          user,
          vaultState,
          vault,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      expect.fail("Withdrawal during cooldown should fail");
    } catch (err) {
      assertAnchorError(err, "cooldown");
    }
  });

  it("Fails to withdraw more than deposited", async () => {
    await program.methods
      .checkIn()
      .accounts({
        owner: user,
        vaultState,
      })
      .rpc();

    await sleep((cooldownSeconds + 1) * 1000);

    try {
      await program.methods
        .withdraw(new anchor.BN(10_000_000))
        .accountsStrict({
          user,
          vaultState,
          vault,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      expect.fail("Over-withdraw should fail");
    } catch (err) {
      assertAnchorError(err, "insufficient");
    }
  });

  it("Fails when non-owner tries to withdraw", async () => {
    const attacker = anchor.web3.Keypair.generate();

    await provider.connection.requestAirdrop(
      attacker.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );
    await sleep((inactivityWindowSeconds + 1) * 1000);

    try {
      await program.methods
        .withdraw(new anchor.BN(10_000))
        .accountsStrict({
          user: attacker.publicKey,
          vaultState,
          vault,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([attacker])
        .rpc();
      

      expect.fail("Unauthorized withdraw should fail");
    } catch (err) {
      assertAnchorError(err, "constraint");
    }
  });
});