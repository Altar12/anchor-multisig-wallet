use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};

pub trait Len {
    fn len() -> usize;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ProposalType {
    Transfer {
        token_mint: Pubkey,
        receive_account: Pubkey,
        amount: u64,
    },
    AddOwner {
        user: Pubkey,
    },
    ChangeProposalLifetime {
        duration: i64,
    },
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct RawWalletAuth {
    pub discriminator: [u8; 8],
    pub owner: Pubkey,
    pub wallet: Pubkey,
    pub id: u8,
    pub added_time: i64,
}

#[account]
pub struct WalletConfig {
    pub name: String, // max length of 20 bytes
    pub m: u8,
    pub n: u8,
    pub owners: u8,
    pub owner_identities: [u8; 32],
    pub proposal_lifetime: i64,
}

#[account]
pub struct WalletAuth {
    pub owner: Pubkey,
    pub wallet: Pubkey,
    pub id: u8,
    pub added_time: i64,
}

#[account]
pub struct Proposal {
    pub wallet: Pubkey,
    pub proposer: Pubkey,
    pub proposal: ProposalType,
}

#[account]
pub struct VoteCount {
    pub proposed_time: i64,
    pub votes: u8,
    pub vote_record: [u8; 32],
}

macro_rules! generate_implementations {
    ($($account:ident),+ $(,)?) => {
        $(
            impl Len for $account {
                fn len() -> usize {
                    8 + std::mem::size_of::<$account>()
                }
            }
        )+
    }
}

generate_implementations!(WalletAuth, Proposal, VoteCount);

impl WalletConfig {
    pub const MAX_NAME_LEN: usize = 20;
}

impl Len for WalletConfig {
    fn len() -> usize {
        8 + (4 + Self::MAX_NAME_LEN) + 1 + 1 + 1 + 32 + 8
    }
}
