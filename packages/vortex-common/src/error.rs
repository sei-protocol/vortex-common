use cosmwasm_std::{Decimal, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Insufficient funds sent")]
    InsufficientFundsSend {},

    #[error("Unexpected Error")]
    UnexpectedError {},

    #[error("Insufficient collateral")]
    InsufficientCollateral {},

    #[error("Premature liquidation")]
    PrematureLiquidation {},

    #[error("Duplicated liquidation")]
    DuplicatedLiquidation {},

    #[error("Failed liquidation")]
    FailedLiquidation {},

    #[error("Failed to serialize")]
    FailedToSerialize { err_msg: String },

    #[error("Failed to convert to binary")]
    FailedToBinary { err_msg: String },

    #[error("Failed to get equity and total market value")]
    FailedToGetEquityAndTotalMarketValue { err_msg: String },

    #[error("Failed to get insurance fund from storage")]
    FailedToGetInsuranceFund {},

    #[error("Failed to get order with order id")]
    FailedToGetOrder { order_id: String },

    #[error("Failed to fetch all balances")]
    FailedToFetchBalances { err_msg: String },

    #[error("Insufficient balance")]
    InsufficientBalance {},

    #[error("Insufficient balance for funding payment")]
    InsufficientBalanceForFundingPayment {},

    #[error("Invalid coin type")]
    InvalidCoinType {},

    #[error("Invalid position effect")]
    InvalidPositionEffect {},

    #[error("Invalid position direction")]
    InvalidPositionDirection {},

    #[error("Invalid cw20 token")]
    Invalidcw20token {},

    #[error("Invalid order data")]
    InvalidOrderData {},

    #[error("Insufficient open amount to close")]
    InsufficientOpenPositionToClose {
        intended_close_amount: Decimal,
        can_be_closed: Decimal,
    },

    #[error("Unsupported Denom")]
    InvalidDenom { unsupported_denom: String },

    #[error("Twap does not exist")]
    TwapNotExist {},

    #[error("Order not found")]
    OrderNotFound {},

    #[error("User not whitelisted for this feature")]
    UnwhitelistedUser {},

    #[error("Pool does not have enough liquidity")]
    InsufficientLiquidity {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
