import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { GoblinNft } from "../target/types/goblin_nft";
import {
  TOKEN_PROGRAM_ID,
  MINT_SIZE,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  createInitializeMintInstruction
} from '@solana/spl-token';
import { assert } from "chai";

describe("goblin-nft", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.GoblinNft as Program<GoblinNft>;

  const mintkey: anchor.web3.Keypair = anchor.web3.Keypair.generate();

  let associatedTokenAccount = undefined;

  it("Mint a token", async () => {
    const key = anchor.AnchorProvider.env().wallet.publicKey;
    const lamports: number = await program.provider.connection.getMinimumBalanceForRentExemption(
      MINT_SIZE
    );

    // get the ATA for a token on a public key (but might not exist yet)

    associatedTokenAccount = await getAssociatedTokenAddress(
      mintkey.publicKey,
      key
    );

    // Fires a list of instructions
    const mint_tx = new anchor.web3.Transaction().add(
      // use anchor to create an account from the key that we created
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: key,
        newAccountPubkey: mintkey.publicKey,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
        lamports,
      }),
      // fire a transactions to create our mint account that is controlled by our anchor wallet (key)
      createInitializeMintInstruction(
        mintkey.publicKey, 0, key, key
      ),
      // create the ATA account that is associated with our mint on our anchor wallet (key)
      createAssociatedTokenAccountInstruction(
        key, associatedTokenAccount, key, mintkey.publicKey
      )
    );
    const res = await anchor.AnchorProvider.env().sendAndConfirm(mint_tx, [mintkey]);
    console.log(
      await program.provider.connection.getParsedAccountInfo(mintkey.publicKey)
    );
    console.log("Account: ", res);
    console.log("Mint key: ", mintkey.publicKey.toString());
    console.log("User: ", key.toString());

    // executes our code to mint our token into our specified ATA
    // TO TEST: as long as we provide the right authority and mint token, can we mint to any account?
    const tx = await program.methods.mintToken().accounts({
      mint: mintkey.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
      tokenAccount: associatedTokenAccount,
      payer: key,
    }).rpc();
    console.log("Your transaction signature", tx);
    // @ts-ignore
    const minted = (await program.provider.connection.getParsedAccountInfo(associatedTokenAccount)).value.data.parsed.info.tokenAmount.amount;
    assert.equal(minted, 10);
  });
  it("Transfer token", async () => {
    // authority of the account sending
    const myWallet = anchor.AnchorProvider.env().wallet.publicKey;
    // ATA that I'm using to send
    // associatedTokenAccount = associatedTokenAccount;
    // Account that is receiving the ATA
    const toWallet: anchor.web3.Keypair = anchor.web3.Keypair.generate();
    // get the ATA for a token on a public key (but might not exist yet)
    const toATA = await getAssociatedTokenAddress(
      mintkey.publicKey,
      toWallet.publicKey
    );

    // Fires a list of instructions
    const mint_tx = new anchor.web3.Transaction().add(
      // create the ATA account that is assciated with our mint on our anchor wallet (key)
      createAssociatedTokenAccountInstruction(
        myWallet, toATA, toWallet.publicKey, mintkey.publicKey
      )
    );

    // sends and create the transaction
    const res = await anchor.AnchorProvider.env().sendAndConfirm(mint_tx, []);
    console.log(res);

    const tx = await program.methods.transferToken().accounts({
      tokenProgram: TOKEN_PROGRAM_ID,
      from: associatedTokenAccount,
      signer: myWallet,
      to: toATA,
    }).rpc();
    console.log("Your transaction signature", tx);
    // @ts-ignore
    const minted = (await program.provider.connection.getParsedAccountInfo(associatedTokenAccount)).value.data.parsed.info.tokenAmount.amount;
    console.log(minted)
  });
});