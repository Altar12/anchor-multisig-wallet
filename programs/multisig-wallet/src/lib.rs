pub mod error;
pub mod instruction_accounts;
pub mod state;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    account_info::next_account_info, program::invoke_signed, rent::Rent, system_instruction,
};
use anchor_spl::token::{self, TokenAccount, Transfer};
use error::*;
use instruction_accounts::*;
use state::{AccountType, Len, Proposal, ProposalType, VoteCount, WalletAuth, WalletConfig};
use std::convert::TryInto;

declare_id!("5wgwCaNBvEBz2LCxdL5nTZSab8wwDDpHfX8RaoW1jRpu");

#[program]
pub mod multisig_wallet {

    use super::*;

    pub fn create_wallet(
        ctx: Context<CreateWallet>,
        m: u8,
        n: u8,
        owners: Vec<Pubkey>,
        proposal_lifetime: i64,
    ) -> Result<()> {
        require!(m > 0 && n > 0, WalletError::ZeroParameters);
        require!(m <= n, WalletError::InvalidParameters);
        require_eq!(
            owners.len(),
            ctx.remaining_accounts.len(),
            WalletError::SizeMismatch
        );
        require!(proposal_lifetime >= 600, WalletError::TooShortDuration);

        ctx.accounts.wallet_auth.set_inner(WalletAuth {
            discriminator: AccountType::WalletAuth,
            owner: ctx.accounts.user.key(),
            wallet: ctx.accounts.wallet.key(),
            id: 0,
            added_time: Clock::get()?.unix_timestamp,
        });

        let wallet_key = ctx.accounts.wallet.key();
        let rent_amount = Rent::get()?.minimum_balance(WalletAuth::len());
        let account_size: u64 = WalletAuth::len().try_into().unwrap();
        let user = ctx.accounts.user.to_account_info();
        let current_time = Clock::get()?.unix_timestamp;
        let mut wallet_auth_account: &AccountInfo;
        let mut wallet_auth_address: Pubkey;
        let mut wallet_auth: WalletAuth;
        let mut bump: u8;
        let mut owner: Pubkey;
        let mut id = 1;
        let account_info_iter = &mut ctx.remaining_accounts.iter();
        for owner in owners {
            wallet_auth_account = next_account_info(account_info_iter)?;
            // verify the wallet_auth passed has correct address
            (wallet_auth_address, bump) = Pubkey::find_program_address(
                &[
                    "owner".as_bytes().as_ref(),
                    wallet_key.as_ref(),
                    owner.as_ref(),
                ],
                &ID,
            );
            require_keys_eq!(
                wallet_auth_address,
                wallet_auth_account.key(),
                WalletError::InvalidWalletAuth
            );
            // create the wallet_auth
            invoke_signed(
                &system_instruction::create_account(
                    user.key,
                    wallet_auth_account.key,
                    rent_amount,
                    account_size,
                    &ID,
                ),
                &[user.clone(), wallet_auth_account.clone()],
                &[&[
                    "owner".as_bytes().as_ref(),
                    wallet_key.as_ref(),
                    owner.as_ref(),
                    &[bump],
                ]],
            )?;
            // initialise the wallet_auth
            wallet_auth = WalletAuth {
                discriminator: AccountType::WalletAuth,
                owner,
                wallet: wallet_key,
                id,
                added_time: current_time,
            };
            wallet_auth.serialize(&mut &mut wallet_auth_account.data.borrow_mut()[..])?;
            id += 1;
        }
        // initialise wallet_config
        let mut owner_identities = [0u8; 32];
        let last_byte = ((id - 1) / 8) as usize;
        let last_owner_pos = (id - 1) % 8;
        for byte in 0..last_byte {
            owner_identities[byte] = 255;
        }
        let mut record_string = String::new();
        for _ in 0..=last_owner_pos {
            record_string.push('1');
        }
        for _ in (last_owner_pos + 1)..8 {
            record_string.push('0');
        }
        owner_identities[last_byte] = u8::from_str_radix(&record_string, 2).unwrap();
        ctx.accounts.wallet.set_inner(WalletConfig {
            discriminator: AccountType::WalletConfig,
            m,
            n,
            owners: ctx.remaining_accounts.len().try_into().unwrap(),
            owner_identities,
            proposal_lifetime,
        });
        Ok(())
    }
    pub fn give_up_ownership(ctx: Context<GiveUpOwnership>) -> Result<()> {
        // if owner count = 1 and accounts present in remaining accounts
        // transfer the funds to specified accounts
        // else update owner count and owner record in wallet
        let wallet = &mut ctx.accounts.wallet;
        if wallet.owners == 1 {
            if ctx.remaining_accounts.len() == 0 {
                return Ok(());
            }
            require_eq!(
                ctx.remaining_accounts.len() % 2,
                0,
                WalletError::InsufficientAccounts
            );
            let authority = ctx
                .accounts
                .wallet_authority
                .as_ref()
                .unwrap()
                .to_account_info();
            let authority_bump = *ctx.bumps.get("wallet_authority").unwrap();
            let cpi_program = ctx
                .accounts
                .token_program
                .as_ref()
                .unwrap()
                .to_account_info();
            let account_info_iter = &mut ctx.remaining_accounts.iter();
            let mut send_account;
            let mut receive_account;
            let mut amount;
            let mut cpi_context;
            while account_info_iter.len() > 0 {
                send_account = next_account_info(account_info_iter)?.clone();
                receive_account = next_account_info(account_info_iter)?.clone();
                amount =
                    TokenAccount::try_deserialize(&mut &send_account.data.borrow()[..])?.amount;
                cpi_context = CpiContext::new(
                    cpi_program.clone(),
                    Transfer {
                        from: send_account,
                        to: receive_account,
                        authority: authority.clone(),
                    },
                );
                token::transfer(
                    cpi_context
                        .with_signer(&[&["authority".as_bytes().as_ref(), &[authority_bump]]]),
                    amount,
                )?;
            }
        }
        Ok(())
    }
    pub fn create_token_account(_ctx: Context<CreateTokenAccount>) -> Result<()> {
        Ok(())
    }
    pub fn create_proposal(ctx: Context<CreateProposal>, proposal: ProposalType) -> Result<()> {
        match proposal {
            ProposalType::Transfer {
                token_mint,
                receive_account,
                amount,
            } => {
                require!(amount > 0, WalletError::ZeroSendAmount);
                let token_account = ctx.accounts.receive_account.as_ref().unwrap();
                require_keys_eq!(
                    receive_account,
                    token_account.key(),
                    WalletError::TokenAccountMismatch
                );
                require_keys_eq!(token_mint, token_account.mint, WalletError::MintMismatch);
            } // duration should be atleast 10 minutes
            ProposalType::ChangeProposalLifetime { duration } => {
                require!(duration >= 600, WalletError::TooShortDuration)
            }
            _ => (),
        }
        ctx.accounts.proposal.set_inner(Proposal {
            discriminator: AccountType::Proposal,
            wallet: ctx.accounts.wallet.key(),
            proposer: ctx.accounts.user.key(),
            proposal,
        });
        let mut vote_record = [0u8; 32];
        let user_id = ctx.accounts.wallet_auth.id;
        let mut vote_string = String::from("00000000");
        vote_string.insert((user_id % 8) as usize, '1');
        if user_id % 8 == 0 {
            vote_string.pop();
        } else {
            vote_string.remove(0);
        }
        vote_record[(user_id / 8) as usize] = u8::from_str_radix(&vote_string, 2).unwrap();
        ctx.accounts.vote_count.set_inner(VoteCount {
            discriminator: AccountType::VoteCount,
            proposed_time: Clock::get()?.unix_timestamp,
            votes: 1,
            vote_record,
        });
        Ok(())
    }
    pub fn vote(ctx: Context<Voting>) -> Result<()> {
        let user_id = ctx.accounts.wallet_auth.id;
        let user_pos = (user_id % 8) as usize;
        let vote_count = &mut ctx.accounts.vote_count;
        let mut vote_string = format!("{:08b}", vote_count.vote_record[(user_id / 8) as usize]);
        require_eq!(
            &vote_string[user_pos..user_pos + 1],
            "0",
            WalletError::AlreadyVoted
        );
        vote_string.replace_range(user_pos..user_pos + 1, "1");
        vote_count.vote_record[(user_id / 8) as usize] =
            u8::from_str_radix(&vote_string, 2).unwrap();
        vote_count.votes = vote_count.votes.checked_add(1).unwrap();
        Ok(())
    }
    pub fn revoke_vote(ctx: Context<Voting>) -> Result<()> {
        let user_id = ctx.accounts.wallet_auth.id;
        let user_pos = (user_id % 8) as usize;
        let vote_count = &mut ctx.accounts.vote_count;
        let mut vote_string = format!("{:08b}", vote_count.vote_record[(user_id / 8) as usize]);
        require_eq!(
            &vote_string[user_pos..user_pos + 1],
            "1",
            WalletError::NotVoted
        );
        vote_string.replace_range(user_pos..user_pos + 1, "0");
        vote_count.vote_record[(user_id / 8) as usize] =
            u8::from_str_radix(&vote_string, 2).unwrap();
        vote_count.votes = vote_count.votes.checked_sub(1).unwrap();
        Ok(())
    }
    pub fn close_proposal(ctx: Context<CloseProposal>) -> Result<()> {
        let wallet = &ctx.accounts.wallet;
        let vote_count = &ctx.accounts.vote_count;
        if Clock::get()?.unix_timestamp >= vote_count.proposed_time + wallet.proposal_lifetime {
            return Ok(());
        }
        require_gte!(
            vote_count.votes,
            (wallet.owners * wallet.m) / wallet.n,
            WalletError::NotEnoughVotes
        );
        match ctx.accounts.proposal.proposal {
            ProposalType::Transfer { amount, .. } => {
                let cpi_program = ctx
                    .accounts
                    .token_program
                    .as_ref()
                    .unwrap()
                    .to_account_info();
                let cpi_context = CpiContext::new(
                    cpi_program,
                    Transfer {
                        from: ctx
                            .accounts
                            .send_account
                            .as_ref()
                            .unwrap()
                            .to_account_info(),
                        to: ctx
                            .accounts
                            .receive_account
                            .as_ref()
                            .unwrap()
                            .to_account_info(),
                        authority: ctx
                            .accounts
                            .wallet_authority
                            .as_ref()
                            .unwrap()
                            .to_account_info(),
                    },
                );
                token::transfer(
                    cpi_context.with_signer(&[&[
                        "authority".as_bytes().as_ref(),
                        &[*ctx.bumps.get("wallet_authority").unwrap()],
                    ]]),
                    amount,
                )?;
            }
            ProposalType::AddOwner { user } => {
                let wallet = &mut ctx.accounts.wallet;
                require!(wallet.owners < 255, WalletError::MaxOwners);
                // find available id and assign it to the new owner
                let mut byte_count = 0;
                let mut byte;
                let mut owner_str;
                loop {
                    byte = wallet.owner_identities[byte_count];
                    owner_str = format!("{:08b}", byte);
                    for pos in 0..8 {
                        if &owner_str[pos..pos + 1] == "0" {
                            owner_str.replace_range(pos..pos + 1, "1");
                            wallet.owner_identities[byte_count] =
                                u8::from_str_radix(&owner_str, 2).unwrap();
                            wallet.owners = wallet.owners.checked_add(1).unwrap();
                            ctx.accounts
                                .wallet_auth
                                .as_mut()
                                .unwrap()
                                .set_inner(WalletAuth {
                                    discriminator: AccountType::WalletAuth,
                                    owner: user,
                                    wallet: wallet.key(),
                                    id: (byte_count * 8 + pos) as u8,
                                    added_time: Clock::get()?.unix_timestamp,
                                });
                        }
                    }
                    byte_count += 1;
                }
            }
            ProposalType::ChangeProposalLifetime { duration } => {
                ctx.accounts.wallet.proposal_lifetime = duration;
            }
        }
        Ok(())
    }
}
