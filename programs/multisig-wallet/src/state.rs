use crate::ID;
use anchor_lang::prelude::Pubkey;
use anchor_lang::{
    AccountDeserialize, AccountSerialize, AnchorDeserialize, AnchorSerialize, Owner, Result,
};
use borsh::{BorshDeserialize, BorshSerialize};
use std::io::Write;

pub trait Len {
    fn len() -> usize;
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum AccountType {
    WalletConfig,
    WalletAuth,
    Proposal,
    VoteCount,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
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

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct WalletConfig {
    pub discriminator: AccountType,
    pub m: u8,
    pub n: u8,
    pub owners: u8,
    pub owner_identities: [u8; 32],
    pub proposal_lifetime: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct WalletAuth {
    pub discriminator: AccountType,
    pub owner: Pubkey,
    pub wallet: Pubkey,
    pub id: u8,
    pub added_time: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct Proposal {
    pub discriminator: AccountType,
    pub wallet: Pubkey,
    pub proposer: Pubkey,
    pub proposal: ProposalType,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct VoteCount {
    pub discriminator: AccountType,
    pub proposed_time: i64,
    pub votes: u8,
    pub vote_record: [u8; 32],
}

macro_rules! generate_implementations {
    ($($account:ident),+ $(,)?) => {
        $(
            impl AccountSerialize for $account {
                fn try_serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
                    self.serialize(writer)?;
                    Ok(())
                }
            }
            impl AccountDeserialize for $account {
                fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
                    let result = $account::deserialize(buf)?;
                    Ok(result)
                }
            }
            impl Owner for $account {
                fn owner() -> Pubkey {
                    ID
                }
            }
            impl Len for $account {
                fn len() -> usize {
                    std::mem::size_of::<$account>()
                }
            }
        )+
    }
}

generate_implementations!(WalletConfig, WalletAuth, Proposal, VoteCount);
