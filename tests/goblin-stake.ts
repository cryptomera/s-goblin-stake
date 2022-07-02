import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { GoblinStake } from "../target/types/goblin_stake";

describe("goblin-stake", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.GoblinStake as Program<GoblinStake>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
