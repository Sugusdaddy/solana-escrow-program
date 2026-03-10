use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Escrow has already been initialized")]
    AlreadyInitialized,
    
    #[msg("Escrow is not in the correct state for this operation")]
    InvalidState,
    
    #[msg("Only the maker can perform this action")]
    OnlyMaker,
    
    #[msg("Only the taker can perform this action")]
    OnlyTaker,
    
    #[msg("Only the arbiter can perform this action")]
    OnlyArbiter,
    
    #[msg("Escrow has expired")]
    Expired,
    
    #[msg("Escrow has not expired yet")]
    NotExpired,
    
    #[msg("Invalid amount")]
    InvalidAmount,
    
    #[msg("Dispute already exists")]
    DisputeExists,
    
    #[msg("No dispute to resolve")]
    NoDispute,
    
    #[msg("Arbiter fee too high (max 10%)")]
    ArbiterFeeTooHigh,
    
    #[msg("Release amounts don't match escrow amount")]
    AmountMismatch,
    
    #[msg("Unauthorized")]
    Unauthorized,
}
