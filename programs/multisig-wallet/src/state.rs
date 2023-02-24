use crate::ID;
use anchor_lang::prelude::Pubkey;
use anchor_lang::{AccountDeserialize, AccountSerialize, Owner, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use std::io::Write;

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
    pub key: AccountType,
    pub m: u8,
    pub n: u8,
    pub owners: u8,
    pub owner_identities: [u8; 32],
    pub proposal_lifetime: i32,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct WalletAuth {
    pub key: AccountType,
    pub owner: Pubkey,
    pub wallet: Pubkey,
    pub id: u8,
    pub added_time: i32,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct Proposal {
    pub key: AccountType,
    pub wallet: Pubkey,
    pub proposer: Pubkey,
    pub proposal: ProposalType,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct VoteCount {
    pub key: AccountType,
    pub proposed_time: i32,
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
        )+
    }
}

generate_implementations!(WalletConfig, WalletAuth, Proposal, VoteCount);