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
pub struct CloseProposal<'info> {
    pub wallet: Account<'info, WalletConfig>,
    #[account(mut, close = proposer)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut, close = proposer,
              seeds = ["votes".as_bytes().as_ref(), wallet.key().as_ref(), proposal.key().as_ref()], bump)]
    pub vote_count: Account<'info, VoteCount>,
    /// CHECK: proposer will receive funds from closing the accounts, just need to check the address
    #[account(mut, address = proposal.proposer)]
    pub proposer: UncheckedAccount<'info>,

    // below accounts required in case of an add owner proposal
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    #[account(init, payer = payer, space = WalletAuth::len(),
              seeds = ["owner".as_bytes().as_ref(), wallet.key().as_ref(),
                       (if let ProposalType::AddOwner{user}=proposal.proposal { user }
                        else { panic!("redundant account, not an add owner proposal") }).as_ref()], bump)]
    pub wallet_auth: Option<Account<'info, WalletAuth>>,
    pub system_program: Option<Program<'info, System>>,

    // below accounts required in case of a transfer proposal
    #[account(mut, token::authority = wallet_authority,
              constraint = send_account.mint == receive_account.as_ref().unwrap().mint)]
    pub send_account: Option<Account<'info, TokenAccount>>,
    #[account(mut, address = if let ProposalType::Transfer{ receive_account: address, token_mint, .. } = proposal.proposal { require_keys_eq!(receive_account.mint, token_mint); address }
                             else { panic!("redundant account, not a transfer proposal") })]
    pub receive_account: Option<Account<'info, TokenAccount>>,
    /// CHECK: pda acting as the authority of all wallet token accounts
    pub wallet_authority: Option<UncheckedAccount<'info>>,
    pub token_program: Option<Program<'info, Token>>,
}
