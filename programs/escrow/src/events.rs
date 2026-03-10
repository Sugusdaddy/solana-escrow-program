use anchor_lang::prelude::*;

#[event]
pub struct EscrowCreated {
    pub escrow: Pubkey,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub amount: u64,
    pub token_mint: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct EscrowCompleted {
    pub escrow: Pubkey,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub amount: u64,
    pub completed_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct EscrowCancelled {
    pub escrow: Pubkey,
    pub cancelled_by: Pubkey,
    pub reason: String,
    pub timestamp: i64,
}

#[event]
pub struct DisputeRaised {
    pub escrow: Pubkey,
    pub raised_by: Pubkey,
    pub reason: String,
    pub timestamp: i64,
}

#[event]
pub struct DisputeResolved {
    pub escrow: Pubkey,
    pub arbiter: Pubkey,
    pub winner: Pubkey,
    pub maker_amount: u64,
    pub taker_amount: u64,
    pub timestamp: i64,
}
