use anchor_lang::prelude::*;
use core::mem::size_of;

declare_id!("GyNvHEC54adUfH4n4VfDG2emdo2PWdk1JSCw5fa84U2h");

const CEO_ADDRESS: Pubkey = pubkey!("5p8rYT2aAxVEb1AVaGayNSwHjZuaQozXnz8D5xS7idtJ");

//Comments and replies need atleast 428 extra bytes of space to pass with full load
const COMMENT_REPLY_OR_IDEA_EXTRA_SIZE: usize = 470;

//Comments and replies need atleast 121 extra bytes of space to pass with full load
const USER_NAME_EXTRA_SIZE: usize = 140;

const MAX_COMMENT_LENGTH: usize = 444;
const MAX_USER_NAME_LENGTH: usize = 144;

#[error_code]
pub enum InvalidLengthError 
{
   /* 
    #[msg("Poll or poll option name can't be longer than 140 characters")]
    PollOrPollOptionNameTooLong,
    #[msg("Comment section name prefix can't be longer than 32 characters")]
    CommentSectionNamePrefixTooLong,
    #[msg("Comment section name can't be longer than 32 characters")]
    CommentSectionNameTooLong,*/
    #[msg("User Name can't be longer than 144 characters")]
    UserNameTooLong,
    #[msg("Message can't be longer than 444 characters")]
    MSGTooLong,
}
#[error_code]
pub enum AuthorizationError 
{
    #[msg("Only the CEO can call this function")]
    NotCEO,
    #[msg("This post isn't yours to change")]
    NotPostOwner,
}  

#[error_code]
pub enum InvalidOperationError 
{
    #[msg("This post was deleted")]
    Deleted,
    #[msg("You must vote for the person who wrote the post")]
    WrongDude,
    #[msg("Can't set flag to the same state")]
    FlagSameState
    /*#[msg("Can't delete poll that still has options, please delete remaining options first")]
    PollStillHasOptions */
}


#[program]
pub mod chat
{
    use super::*;

    pub fn create_chat_account(ctx: Context<CreateChatAccount>) -> Result<()>
    {
        msg!("New Chat Account");
        msg!("Address: {}", ctx.accounts.signer.key());
        Ok(())
    }

    pub fn set_user_name(ctx: Context<SetUserName>, user_name: String) -> Result<()>
    {
        //Comment message string must not be longer than 144 characters
        require!(user_name.len() <= MAX_USER_NAME_LENGTH, InvalidLengthError::UserNameTooLong);

        let chat_account = &mut ctx.accounts.chat_account;

        chat_account.user_name = user_name;
        chat_account.use_custom_name = true;

        msg!("New Chat Account User Name");
        msg!("Address: {}", ctx.accounts.signer.key());
        msg!("User Name: {}", chat_account.user_name);
        Ok(())
    }

    pub fn set_user_name_flag(ctx: Context<SetUserNameFlag>, is_enabled: bool) -> Result<()>
    {
        let chat_account = &mut ctx.accounts.chat_account;

        //Can't set flag to the same state
        require!(chat_account.use_custom_name != is_enabled, InvalidOperationError::FlagSameState);

        chat_account.use_custom_name = is_enabled;

        msg!("Chat Account 'Use Custom Name Flag' Changed");
        msg!("Value: {}", is_enabled);
        Ok(())
    }

    pub fn create_comment_section(ctx: Context<CreateCommentSection>, comment_section_name: String) -> Result<()>
    {
        let comment_section = &mut ctx.accounts.comment_section;

        msg!("New Comment Section {}", comment_section_name);
        msg!("Initial Post Count {}", comment_section.post_count);
        Ok(())
    }

    pub fn post_comment(ctx: Context<PostComment>, 
        comment_section_name: String, 
        message: String) -> Result<()> 
    {
        //Comment message string must not be longer than 444 characters
        require!(message.len() <= MAX_COMMENT_LENGTH, InvalidLengthError::MSGTooLong);

        let comment_section = &mut ctx.accounts.comment_section;
        comment_section.post_count += 1;

        msg!("Comment Section: {}", comment_section_name);
        msg!("New Comment: {}", message);
        msg!("New Comment Total: {}", comment_section.post_count);

        let chat_account = &mut ctx.accounts.chat_account;
        let comment = &mut ctx.accounts.comment;
        comment.id = comment_section.post_count;
        comment.user_post_count_index = chat_account.post_count;
        comment.post_owner_address = ctx.accounts.signer.key();
        comment.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        comment.msg = message;

        chat_account.post_count += 1;
 
        Ok(())
    }

    pub fn post_reply(ctx: Context<PostReply>, 
        comment_section_name: String,
        _user_post_count_index: u32,
        _post_owner_address: Pubkey,
        message: String) -> Result<()> 
    {
        let comment_section = &mut ctx.accounts.comment_section;
        comment_section.post_count += 1;

        msg!("Comment Section: {}", comment_section_name);
        msg!("New Reply: {}", message);
        msg!("New Reply Total: {}", comment_section.post_count);

        let chat_account = &mut ctx.accounts.chat_account;
        let comment = &mut ctx.accounts.comment;
        let reply = &mut ctx.accounts.reply;
        reply.id = comment_section.post_count;
        reply.parent_id = comment.id;
        reply.user_post_count_index = chat_account.post_count;
        reply.post_owner_address = ctx.accounts.signer.key();
        reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        reply.msg = message;

        comment.reply_count += 1;
        chat_account.post_count += 1;
 
        Ok(())
    }

    pub fn edit_comment(ctx: Context<EditComment>, 
        _comment_section_name: String, 
        _user_post_count_index: u32,
        message: String) -> Result<()> 
    {
        let comment = &mut ctx.accounts.comment;

        //Only Post owner can make changes to post
        require_keys_eq!(comment.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotPostOwner);

        //Comment message string must not be longer than 444 characters
        require!(message.len() <= MAX_COMMENT_LENGTH, InvalidLengthError::MSGTooLong);

        msg!("User Edited Message");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Old Message: {}", comment.msg);

        comment.msg = message;

        if comment.is_edited == false
        {
            comment.is_edited = true;
        }

        let comment_section = &mut ctx.accounts.comment_section;
        comment_section.post_edited_count += 1;
        
        msg!("New Message: {}", comment.msg);
 
        Ok(())
    }

    pub fn delete_comment(ctx: Context<DeleteComment>, 
        _comment_section_name: String, 
        _user_post_count_index: u32) -> Result<()> 
    {
        let comment = &mut ctx.accounts.comment;

        //Only Post owner can make changes to post
        require_keys_eq!(comment.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotPostOwner);

        msg!("User Has Deleted Message");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Old Message: {}", comment.msg);

        comment.is_deleted = true;

        let comment_section = &mut ctx.accounts.comment_section;
        comment_section.post_deleted_count += 1;

        Ok(())
    }

    pub fn vote_comment(ctx: Context<VoteComment>, 
        _comment_section_name: String, 
        _user_post_count_index: u32,
        _post_owner_address: Pubkey,
        is_up_vote: bool) -> Result<()> 
    {
        let comment = &mut ctx.accounts.comment;

        //Can't vote for deleted post
        require!(comment.is_deleted == false, InvalidOperationError::Deleted);

        if is_up_vote
        {
            comment.votes += 1;
            msg!("User Has Upvoted Message");
            msg!("User Address: {}", ctx.accounts.signer.key());
        }
        else
        {
            comment.votes -= 1;
            msg!("User Has Downvoted Message");
            msg!("User Address: {}", ctx.accounts.signer.key());
        }

        let comment_section = &mut ctx.accounts.comment_section;
        comment_section.post_voted_count += 1;

        Ok(())
    }

    pub fn star_comment(ctx: Context<StarComment>, 
        _comment_section_name: String, 
        _user_post_count_index: u32,
        _post_owner_address: Pubkey,
        is_star: bool) -> Result<()> 
    {
        //Only CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), CEO_ADDRESS, AuthorizationError::NotCEO);

        let comment = &mut ctx.accounts.comment;

        //Can't set flag to the same state
        require!(comment.is_starred != is_star, InvalidOperationError::FlagSameState);

        comment.is_starred = is_star;

        if is_star
        {
            
            msg!("CEO has starred a Post");
            msg!("User Address: {}", ctx.accounts.signer.key());
        }
        else
        {
            msg!("CEO has unstarred a Post");
            msg!("User Address: {}", ctx.accounts.signer.key());
        }

        let comment_section = &mut ctx.accounts.comment_section;
        comment_section.post_starred_count += 1;

        Ok(())
    }

    pub fn fed_comment(ctx: Context<FEDComment>, 
        _comment_section_name: String, 
        _user_post_count_index: u32,
        _post_owner_address: Pubkey,
        is_fed: bool) -> Result<()> 
    {
        //Only CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), CEO_ADDRESS, AuthorizationError::NotCEO);

        let comment = &mut ctx.accounts.comment;

        //Can't set flag to the same state
        require!(comment.is_fed != is_fed, InvalidOperationError::FlagSameState);

        comment.is_fed = is_fed;

        if is_fed
        {
            
            msg!("CEO has fed a Post");
            msg!("User Address: {}", ctx.accounts.signer.key());
        }
        else
        {
            msg!("CEO has unfed a Post");
            msg!("User Address: {}", ctx.accounts.signer.key());
        }

        let comment_section = &mut ctx.accounts.comment_section;
        comment_section.post_fed_count += 1;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateChatAccount<'info> 
{
    #[account(
        init, 
        payer = signer, 
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump, 
        space = size_of::<ChatAccount>() + USER_NAME_EXTRA_SIZE + 8)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetUserName<'info> 
{
    #[account(
        mut, 
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetUserNameFlag<'info> 
{
    #[account(
        mut, 
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(comment_section_name: String)]
pub struct CreateCommentSection<'info> 
{
    #[account(
        init, 
        payer = signer, 
        seeds = [b"commentSection".as_ref(), comment_section_name.as_ref()], 
        bump, 
        space = size_of::<CommentSection>() + 8)]
    pub comment_section: Account<'info, CommentSection>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(comment_section_name: String)]
pub struct PostComment<'info> 
{
    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut,
        seeds = [b"commentSection".as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Account<'info, CommentSection>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"comment".as_ref(), comment_section_name.as_ref(), chat_account.post_count.to_le_bytes().as_ref(), signer.key().as_ref()], 
        bump, 
        space = size_of::<Comment>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub comment: Account<'info, Comment>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(comment_section_name: String, user_post_count_index: u32, post_owner_address: Pubkey)]
pub struct PostReply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut,
        seeds = [b"commentSection".as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Account<'info, CommentSection>,

    #[account(
        mut,
        seeds = [b"comment".as_ref(), comment_section_name.as_ref(), user_post_count_index.to_le_bytes().as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub comment: Account<'info, Comment>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"comment".as_ref(), comment_section_name.as_ref(), chat_account.post_count.to_le_bytes().as_ref(), signer.key().as_ref()], 
        bump, 
        space = size_of::<Comment>() + (4 * MAX_COMMENT_LENGTH) + 8)]
    pub reply: Account<'info, Comment>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(comment_section_name: String, user_post_count_index: u32)]
pub struct EditComment<'info> 
{
    #[account(
        mut,
        seeds = [b"commentSection".as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Account<'info, CommentSection>,

    #[account(
        mut,
        seeds = [b"comment".as_ref(), comment_section_name.as_ref(), user_post_count_index.to_le_bytes().as_ref(), signer.key().as_ref()], 
        bump)]
    pub comment: Account<'info, Comment>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(comment_section_name: String, user_post_count_index: u32)]
pub struct DeleteComment<'info> 
{
    #[account(
        mut,
        seeds = [b"commentSection".as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Account<'info, CommentSection>,

    #[account(
        mut,
        seeds = [b"comment".as_ref(), comment_section_name.as_ref(), user_post_count_index.to_le_bytes().as_ref(), signer.key().as_ref()], 
        bump)]
    pub comment: Account<'info, Comment>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(comment_section_name: String, user_post_count_index: u32, post_owner_address: Pubkey)]
pub struct VoteComment<'info> 
{
    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut,
        seeds = [b"commentSection".as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Account<'info, CommentSection>,

    #[account(
        mut,
        seeds = [b"comment".as_ref(), comment_section_name.as_ref(), user_post_count_index.to_le_bytes().as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub comment: Account<'info, Comment>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(comment_section_name: String, user_post_count_index: u32, post_owner_address: Pubkey)]
pub struct StarComment<'info> 
{
    #[account(
        mut,
        seeds = [b"commentSection".as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Account<'info, CommentSection>,

    #[account(
        mut,
        seeds = [b"comment".as_ref(), comment_section_name.as_ref(), user_post_count_index.to_le_bytes().as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub comment: Account<'info, Comment>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(comment_section_name: String, user_post_count_index: u32, post_owner_address: Pubkey)]
pub struct FEDComment<'info> 
{
    #[account(
        mut,
        seeds = [b"commentSection".as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Account<'info, CommentSection>,

    #[account(
        mut,
        seeds = [b"comment".as_ref(), comment_section_name.as_ref(), user_post_count_index.to_le_bytes().as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub comment: Account<'info, Comment>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct ChatAccount
{
    pub post_count: u32,
    pub user_name: String,
    pub use_custom_name: bool
}

#[account]
pub struct CommentSection
{
    pub post_count: u32,
    pub post_edited_count: u32,
    pub post_deleted_count: u32,
    pub post_voted_count: u32,
    pub post_starred_count: u32,
    pub post_fed_count: u32
}

#[account]
pub struct Comment
{
    pub id: u32,
    pub parent_id: u32,
    pub user_post_count_index: u32,
    pub post_owner_address: Pubkey,
    pub votes: i64,
    pub unix_creation_time_stamp: u64,
    pub msg: String,
    pub reply_count: u32,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
}