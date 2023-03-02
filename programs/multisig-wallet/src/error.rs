use anchor_lang::error_code;

#[error_code]
pub enum WalletError {
    #[msg("Wallet parameters m and n can not be zero")]
    ZeroParameters,
    #[msg("Wallet parameter m should be less than or equal to n")]
    InvalidParameters,
    #[msg("One or more wallet auth accounts passed are invalid")]
    InvalidWalletAuth,
    #[msg("Insufficient token accounts passed to retrieve funds")]
    InsufficientAccounts,
    #[msg("Send amount specified is zero for transfer proposal")]
    ZeroSendAmount,
    #[msg("Mint of the receive account does not match with the one specified")]
    MintMismatch,
    #[msg("The passed token account address and the one specified do not match")]
    TokenAccountMismatch,
    #[msg("The duration of proposal can not be less than 10 minutes")]
    TooShortDuration,
    #[msg("The user has already voted the proposal")]
    AlreadyVoted,
    #[msg("Can not revoke vote, the user has not voted")]
    NotVoted,
    #[msg("Not enough votes to execute the proposal")]
    NotEnoughVotes,
    #[msg("Wallet already has max (255) number of owners")]
    MaxOwners,
    #[msg(
        "The number of owner addresses passed and the number of wallet auths passed is different"
    )]
    SizeMismatch,
}
