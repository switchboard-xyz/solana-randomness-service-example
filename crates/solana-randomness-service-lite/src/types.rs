use crate::*;
use borsh::{to_vec, BorshDeserialize, BorshSerialize};

#[derive(Default, Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct TransactionOptions {
    pub compute_units: Option<u32>,
    pub compute_unit_price: Option<u64>,
}
impl TransactionOptions {
    pub const DEFAULT_COMPUTE_UNITS: u32 = 1_000_000;
    pub const MINIMUM_COMPUTE_UNITS: u32 = 200_000;
    pub const MAXIMUM_COMPUTE_UNITS: u32 = 1_400_000;

    pub const DEFAULT_COMPUTE_UNIT_PRICE: u64 = 1;
    pub const MINIMUM_COMPUTE_UNIT_PRICE: u64 = 1;
    pub const MAXIMUM_COMPUTE_UNIT_PRICE: u64 = 1_000_000_000;

    pub fn get_compute_units(&self) -> u32 {
        std::cmp::max(
            Self::MINIMUM_COMPUTE_UNITS,
            std::cmp::min(
                Self::MAXIMUM_COMPUTE_UNITS,
                self.compute_units.unwrap_or(Self::DEFAULT_COMPUTE_UNITS),
            ),
        )
    }

    pub fn get_compute_unit_price(&self) -> u64 {
        std::cmp::max(
            Self::MINIMUM_COMPUTE_UNIT_PRICE,
            std::cmp::min(
                Self::MAXIMUM_COMPUTE_UNIT_PRICE,
                self.compute_unit_price
                    .unwrap_or(Self::DEFAULT_COMPUTE_UNIT_PRICE),
            ),
        )
    }

    pub fn get_priority_fee_lamports(&self) -> u64 {
        // 1_000_000 compute units * 1 micro_lamports per compute unit
        // 1 micro_lamports per compute unit * 1_000_000 compute units = 1_000_000 micro_lamports
        // 1_000_000 micro_lamports / 1_000_000 micro_lamports per lamport = 1 lamport

        // 1_000_000 compute units * 1 micro_lamports per compute unit = 1 lamports
        // 1_000_000 compute units * 1000 micro_lamports per compute unit = 1_000 lamports

        (u64::from(self.get_compute_units()) * self.get_compute_unit_price()) / 1_000_000
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, ProgramError> {
        to_vec(self).map_err(|e| ProgramError::BorshIoError(format!("Serialization failed: {}", e)))
    }

    pub fn to_opt_vec(opt: &Option<TransactionOptions>) -> Result<Vec<u8>, ProgramError> {
        to_vec(opt).map_err(|e| ProgramError::BorshIoError(format!("Serialization failed: {}", e)))
    }
}

#[derive(Clone, Debug, Default, BorshSerialize, BorshDeserialize)]
pub struct AccountMetaBorsh {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}
impl From<AccountMeta> for AccountMetaBorsh {
    fn from(value: AccountMeta) -> Self {
        Self {
            pubkey: value.pubkey,
            is_signer: value.is_signer,
            is_writable: value.is_writable,
        }
    }
}
impl From<&AccountMetaBorsh> for AccountMeta {
    fn from(val: &AccountMetaBorsh) -> Self {
        AccountMeta {
            pubkey: val.pubkey,
            is_signer: val.is_signer,
            is_writable: val.is_writable,
        }
    }
}

#[derive(Clone, Debug, Default, BorshSerialize, BorshDeserialize)]
pub struct Callback {
    pub program_id: Pubkey,
    pub accounts: Vec<AccountMetaBorsh>,
    pub ix_data: Vec<u8>,
}
impl Callback {
    pub fn new(program_id: Pubkey, accounts: Vec<AccountMetaBorsh>, ix_data: Vec<u8>) -> Self {
        Self {
            program_id,
            accounts,
            ix_data,
        }
    }
    pub fn to_vec(&self) -> Result<Vec<u8>, ProgramError> {
        to_vec(self).map_err(|e| ProgramError::BorshIoError(format!("Serialization failed: {}", e)))
    }
}

impl From<AccountMetaBorsh> for AccountMeta {
    fn from(val: AccountMetaBorsh) -> Self {
        AccountMeta {
            pubkey: val.pubkey,
            is_signer: val.is_signer,
            is_writable: val.is_writable,
        }
    }
}
