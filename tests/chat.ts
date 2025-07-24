import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Chat } from "../target/types/chat";
import { utf8 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { assert } from "chai"
import bs58 from 'bs58';
import { Keypair } from '@solana/web3.js'; // Import the Keypair class

describe("chat", () => 
{
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.chat as Program<Chat>;
  const commentSectionName = "M4A_Overview"
  const userName144 = "Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Aenean commodo ligula eget dolor. Aenean massa. Cum sociis natoque penatibus et magnis"
  const message444 = "Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Aenean commodo ligula eget dolor. Aenean massa. Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Donec quam felis, ultricies nec, pellentesque eu, pretium quis, sem. Nulla consequat massa quis enim. Donec pede justo, fringilla vel, aliquet nec, vulputate eget, arcu. In enim justo, rhoncus ut, imperdiet a, venenatis vitae, justo. Nullam dictum feli"

  it("Creates Chat Account", async () => 
  {
    //const keypair = Keypair.fromSecretKey(bs58.decode("Private Key String")); 
    //console.log(keypair.secretKey) //prints out U8Int array to put in .config/solana/id.json file if you want to put in your own wallet.
    //Will need to delete target folder, run "cargo clean" cmd, "solana air drop 100 sol <pulicAddressString>" to wallet to have enough to deploy, then build and deploy

    const keypair = Keypair.fromSecretKey(bs58.decode("2GpFkiMYqcxKPrHuAETH4gHFmmja2EU2bMjTcDV9SG37tk8vJ9LZuLAqWfNEgTo1mtFBW5FJMMhnRNpWSNNcwGUW")); 
    console.log(keypair.secretKey) //prints out U8Int array to put in .config/solana/id.json file if you want to put in your own wallet.
    //Will need to delete target folder, run "cargo clean" cmd, "solana air drop 100 sol <pulicAddressString>" to wallet to have enough to deploy, then build and deploy

    await program.methods.createChatAccount().rpc()

    const chatAccountPDA = getChatAccountPDA(program.provider.wallet.publicKey)
    const chatAccount = await program.account.chatAccount.fetch(chatAccountPDA)

    assert.equal(chatAccount.postCount, 0)
  });

  it("Sets A Custom User Name", async () => 
  {
    await program.methods.setUserName(userName144).rpc()

    const chatAccountPDA = getChatAccountPDA(program.provider.wallet.publicKey)
    const chatAccount = await program.account.chatAccount.fetch(chatAccountPDA)

    assert.equal(chatAccount.userName, userName144)
    assert.equal(chatAccount.useCustomName, true)
  });

  it("Sets Use Custom Name Flag To False", async () => 
  {
    await program.methods.setUserNameFlag(false).rpc()

    const chatAccountPDA = getChatAccountPDA(program.provider.wallet.publicKey)
    const chatAccount = await program.account.chatAccount.fetch(chatAccountPDA)

    assert.equal(chatAccount.useCustomName, false)
  });

  it("Sets Use Custom Name Flag To True", async () => 
  {
    await program.methods.setUserNameFlag(true).rpc()

    const chatAccountPDA = getChatAccountPDA(program.provider.wallet.publicKey)
    const chatAccount = await program.account.chatAccount.fetch(chatAccountPDA)

    assert.equal(chatAccount.useCustomName, true)
  });

  it("Creates Comment Section", async () => 
  {
    await program.methods.createCommentSection(commentSectionName).rpc();

    const commentSectionPDA = getCommentSectionPDA(commentSectionName)
    const commentSection = await program.account.commentSection.fetch(commentSectionPDA)

    assert.equal(commentSection.postCount, 0)
  });

  it("Posts, Votes, Stars, Feds, Edits, Deletes, and Replies to Comment", async () => 
  {
    var testPostCount = 0;

    for(var i = 0; i < 1; i++)
    {
      //Post Comment
      await program.methods.postComment(commentSectionName, message444).rpc()
      testPostCount += 1

      const commentSectionPDA = getCommentSectionPDA(commentSectionName)
      var commentSection = await program.account.commentSection.fetch(commentSectionPDA)

      assert.equal(commentSection.postCount, testPostCount)

      const chatAccountPDA = getChatAccountPDA(program.provider.wallet.publicKey)

      var chatAccount = await program.account.chatAccount.fetch(chatAccountPDA)

      var commentPDA = getCommentPDA(commentSectionName, chatAccount.postCount-1, program.provider.wallet.publicKey) //Latest post from user
      var comment = await program.account.comment.fetch(commentPDA)

      assert.equal(comment.msg, message444)

      //UpVote Post
      await program.methods.voteComment(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey, true).rpc()

      var commentPDA = getCommentPDA(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey) //Latest post from user
      var comment = await program.account.comment.fetch(commentPDA)

      assert(new anchor.BN(1).eq(comment.votes))

      //DownVote Post
      await program.methods.voteComment(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey, false).rpc()

      var commentPDA = getCommentPDA(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey) //Latest post from user
      var comment = await program.account.comment.fetch(commentPDA)

      assert(new anchor.BN(0).eq(comment.votes))

      //Star Post
      await program.methods.starComment(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey, true).rpc()

      var commentPDA = getCommentPDA(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey) //Latest post from user
      var comment = await program.account.comment.fetch(commentPDA)

      assert.equal(comment.isStarred, true)

      //UnStar Post
      await program.methods.starComment(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey, false).rpc()

      var commentPDA = getCommentPDA(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey) //Latest post from user
      var comment = await program.account.comment.fetch(commentPDA)

      assert.equal(comment.isStarred, false)

      //FED Post
      await program.methods.fedComment(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey, true).rpc()

      var commentPDA = getCommentPDA(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey) //Latest post from user
      var comment = await program.account.comment.fetch(commentPDA)

      assert.equal(comment.isFed, true)

      //UnFED Post
      await program.methods.fedComment(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey, false).rpc()

      var commentPDA = getCommentPDA(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey) //Latest post from user
      var comment = await program.account.comment.fetch(commentPDA)

      assert.equal(comment.isFed, false)

      //Edit Post
      const editedMessage = "editedMessage"
      await program.methods.editComment(commentSectionName, comment.userPostCountIndex, editedMessage).rpc()
      
      commentPDA = getCommentPDA(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey)
      comment = await program.account.comment.fetch(commentPDA)

      assert.equal(comment.msg, editedMessage)
      assert.equal(comment.isEdited, true)

      //Delete Post
      await program.methods.deleteComment(commentSectionName, comment.userPostCountIndex).rpc()
      
      commentPDA = getCommentPDA(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey)
      comment = await program.account.comment.fetch(commentPDA)

      assert.equal(comment.isDeleted, true)

      //Post Reply
      await program.methods.postReply(commentSectionName, comment.userPostCountIndex, program.provider.wallet.publicKey, message444).rpc()
      testPostCount += 1

      commentSection = await program.account.commentSection.fetch(commentSectionPDA)

      assert.equal(commentSection.postCount, testPostCount)
      
      chatAccount = await program.account.chatAccount.fetch(chatAccountPDA)
      commentPDA = getCommentPDA(commentSectionName, chatAccount.postCount-1, program.provider.wallet.publicKey) //Latest post from user
      const reply = await program.account.comment.fetch(commentPDA)

      assert.equal(reply.msg, message444)
    }
  });

  function getChatAccountPDA(userAddress: anchor.web3.PublicKey)
  {
    const [chatAccountPDA] = anchor.web3.PublicKey.findProgramAddressSync
    (
      [
        utf8.encode("chatAccount"),
        userAddress.toBuffer()
      ],
      program.programId
    );
    return chatAccountPDA;
  }

  function getCommentSectionPDA(commentSection: string)
  {
    const [commentSectionPDA] = anchor.web3.PublicKey.findProgramAddressSync
    (
      [
        utf8.encode("commentSection"),
        utf8.encode(commentSection)
      ],
      program.programId
    );
    return commentSectionPDA;
  }

  function getCommentPDA(commentSection: string, id: number, postOwnerAddress: anchor.web3.PublicKey)
  {
    const [commentPDA] = anchor.web3.PublicKey.findProgramAddressSync
    (
      [
        utf8.encode("comment"),
        utf8.encode(commentSection),
        new anchor.BN(id).toBuffer('le', 4),
        postOwnerAddress.toBuffer()
      ],
      program.programId
    );
    return commentPDA;
  }

  function getReplyPDA(commentSection: string, id: number)
  {
    const [replyPDA] = anchor.web3.PublicKey.findProgramAddressSync
    (
      [
        utf8.encode("reply"),
        utf8.encode(commentSection),
        new anchor.BN(id).toBuffer('le', 4)
      ],
      program.programId
    );
    return replyPDA;
  }
});


