use crate::state::{Len, Proposal, ProposalType, VoteCount, WalletAuth, WalletConfig};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct CreateWallet<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init, payer = user,
              space = WalletConfig::len())]
    pub wallet: Account<'info, WalletConfig>,
    #[account(init, payer = user,
              space = WalletAuth::len(),
              seeds = ["owner".as_bytes().as_ref(), wallet.key().as_ref(), user.key().as_ref()],
              bump)]
    pub wallet_auth: Account<'info, WalletAuth>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub wallet: Account<'info, WalletConfig>,
    /// CHECK: pda acting as the authority of all wallet token accounts
    #[account(seeds = ["authority".as_bytes().as_ref(), wallet.key().as_ref()], bump)]
    pub wallet_authority: UncheckedAccount<'info>,
    pub mint: Account<'info, Mint>,
    #[account(init, payer = payer,
              associated_token::mint = mint,
              associated_token::authority = wallet_authority)]
    pub account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GiveUpOwnership<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub wallet: Account<'info, WalletConfig>,
    #[account(mut, close = user,
              seeds = ["owner".as_bytes().as_ref(), wallet.key().as_ref(), user.key().as_ref()], bump)]
    pub wallet_auth: Account<'info, WalletAuth>,
    /// CHECK: pda acting as the authority of all wallet token accounts
    #[account(seeds = ["authority".as_bytes().as_ref(), wallet.key().as_ref()], bump)]
    pub wallet_authority: Option<UncheckedAccount<'info>>,
    pub token_program: Option<Program<'info, Token>>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub wallet: Account<'info, WalletConfig>,
    #[account(seeds = ["owner".as_bytes().as_ref(), wallet.key().as_ref(), user.key().as_ref()], bump)]
    pub wallet_auth: Account<'info, WalletAuth>,
    #[account(init, payer = user, space = Proposal::len())]
    pub proposal: Account<'info, Proposal>,
    #[account(init, payer = user, space = VoteCount::len(),
              seeds = ["votes".as_bytes().as_ref(), wallet.key().as_ref(), proposal.key().as_ref()], bump)]
    pub vote_count: Account<'info, VoteCount>,
    pub system_program: Program<'info, System>,

    // required in case of creating a transfer proposal
    pub receive_account: Option<Account<'info, TokenAccount>>,
}

// used for vote and revoke_vote instruction
#[derive(Accounts)]
pub struct Voting<'info> {
    pub user: Signer<'info>,
    pub wallet: Account<'info, WalletConfig>,
    #[account(seeds = ["owner".as_bytes().as_ref(), wallet.key().as_ref(), user.key().as_ref()], bump)]
    pub wallet_auth: Account<'info, WalletAuth>,
    pub proposal: Account<'info, Proposal>,
    #[account(mut,
              seeds = ["votes".as_bytes().as_ref(), wallet.key().as_ref(), proposal.key().as_ref()], bump)]
    pub vote_count: Account<'info, VoteCount>,
}

#[derive(Accounts)]
pub struct TransferFunds<'info> {
    pub wallet: Account<'info, WalletConfig>,
    #[account(mut, close = proposer)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut, close = proposer,
              seeds = ["votes".as_bytes().as_ref(), wallet.key().as_ref(), proposal.key().as_ref()], bump)]
    pub vote_count: Account<'info, VoteCount>,
    /// CHECK: proposer will receive funds from closing the accounts, just need to check the address
    #[account(mut, address = proposal.proposer)]
    pub proposer: UncheckedAccount<'info>,
    #[account(mut, token::authority = wallet_authority)]
    pub send_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub receive_account: Account<'info, TokenAccount>,
    /// CHECK: pda acting as the authority of all wallet token accounts
    #[account(seeds=["authority".as_bytes().as_ref(), wallet.key().as_ref()], bump)]
    pub wallet_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AddOwner<'info> {
    pub wallet: Account<'info, WalletConfig>,
    #[account(mut, close = proposer)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut, close = proposer,
              seeds = ["votes".as_bytes().as_ref(), wallet.key().as_ref(), proposal.key().as_ref()], bump)]
    pub vote_count: Account<'info, VoteCount>,
    /// CHECK: proposer will receive funds from closing the accounts, just need to check the address
    #[account(mut, address = proposal.proposer)]
    pub proposer: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, space = WalletAuth::len(),
              seeds = ["owner".as_bytes().as_ref(), wallet.key().as_ref(),
                       (if let ProposalType::AddOwner{user}=proposal.proposal { user }
                        else { panic!("redundant account, not an add owner proposal") }).as_ref()], bump)]
    pub wallet_auth: Account<'info, WalletAuth>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ChangeLifetime<'info> {
    pub wallet: Account<'info, WalletConfig>,
    #[account(mut, close = proposer)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut, close = proposer,
              seeds = ["votes".as_bytes().as_ref(), wallet.key().as_ref(), proposal.key().as_ref()], bump)]
    pub vote_count: Account<'info, VoteCount>,
    /// CHECK: proposer will receive funds from closing the accounts, just need to check the address
    #[account(mut, address = proposal.proposer)]
    pub proposer: UncheckedAccount<'info>,
}
