use std::io::Write;

use crate::utils::SignedDecimal;
use cosmwasm_std::{Decimal, StdError};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Order {
    pub id: u64,
    pub account: String,
    pub price_denom: String,
    pub asset_denom: String,
    pub price: SignedDecimal,
    pub quantity: SignedDecimal,
    pub remaining_quantity: SignedDecimal,
    pub direction: PositionDirection,
    pub effect: PositionEffect,
    pub leverage: SignedDecimal,
    pub order_type: OrderType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingPaymentRate {
    pub price_diff: SignedDecimal,
    pub epoch: i64,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema, Eq, Hash)]
pub enum PositionDirection {
    Unknown,
    Long,
    Short,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema, Eq, Hash)]
pub enum PositionEffect {
    Unknown,
    Open,
    Close,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema, Eq, Hash)]
pub enum OrderType {
    Unknown,
    Limit,
    Market,
    Liquidation,
    FokMarket,
}

pub fn i32_to_order_type(i: i32) -> OrderType {
    match i {
        0i32 => OrderType::Limit,
        1i32 => OrderType::Market,
        2i32 => OrderType::Liquidation,
        3i32 => OrderType::FokMarket,
        _ => OrderType::Unknown,
    }
}

pub fn order_type_to_i32(o: OrderType) -> i32 {
    match o {
        OrderType::Limit => 0i32,
        OrderType::Market => 1i32,
        OrderType::Liquidation => 2i32,
        OrderType::FokMarket => 3i32,
        OrderType::Unknown => -1i32,
    }
}

pub fn i32_to_direction(i: i32) -> PositionDirection {
    match i {
        0i32 => PositionDirection::Long,
        1i32 => PositionDirection::Short,
        _ => PositionDirection::Unknown,
    }
}

pub fn direction_to_i32(d: PositionDirection) -> i32 {
    match d {
        PositionDirection::Long => 0i32,
        PositionDirection::Short => 1i32,
        PositionDirection::Unknown => -1i32,
    }
}

#[derive(Clone, Serialize, Deserialize, Hash, PartialEq, Eq, Debug, JsonSchema)]
// price denom, asset denom
// use string because we want to be able to dynamically add new token support
pub struct Pair {
    pub price_denom: String,
    pub asset_denom: String,
}

impl Pair {
    fn to_bytes(&self) -> [u8; 16] {
        let mut price_denom_bytes: [u8; 8] = [0; 8];
        let mut asset_denom_bytes: [u8; 8] = [0; 8];
        let mut bytes = [0 as u8; 16];

        self.fill_bytes_from_price_denom(&mut price_denom_bytes);
        self.fill_bytes_from_asset_denom(&mut asset_denom_bytes);

        for i in 0..8 {
            bytes[i] = price_denom_bytes[i];
            bytes[i + 8] = asset_denom_bytes[i];
        }

        bytes
    }

    pub fn fill_bytes_from_price_denom(&self, mut bytes: &mut [u8]) {
        bytes.write(self.price_denom.as_bytes()).unwrap();
    }

    pub fn fill_bytes_from_asset_denom(&self, mut bytes: &mut [u8]) {
        bytes.write(self.asset_denom.as_bytes()).unwrap();
    }
}

// enable Pair to be returned from `range_de()` and friends.
impl KeyDeserialize for Pair {
    type Output = Pair;

    fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
        if value.len() != 16 {
            return Err(StdError::ParseErr {
                target_type: "pair".to_owned(),
                msg: "bytes should have a length of 16".to_owned(),
            });
        }
        let mut price_denom_last_char_idx = 8;
        while price_denom_last_char_idx > 0 && value[price_denom_last_char_idx - 1] == 0 {
            price_denom_last_char_idx -= 1;
        }
        let price_value = value.get(0..price_denom_last_char_idx).unwrap();
        let price_denom = std::str::from_utf8(price_value).unwrap().to_string();
        let mut asset_denom_last_char_idx = 16;
        while asset_denom_last_char_idx > 8 && value[asset_denom_last_char_idx - 1] == 0 {
            asset_denom_last_char_idx -= 1;
        }
        let asset_value = value.get(8..asset_denom_last_char_idx).unwrap();
        let asset_denom = std::str::from_utf8(asset_value).unwrap().to_string();

        Ok(Pair {
            price_denom: price_denom,
            asset_denom: asset_denom,
        })
    }
}

impl<'a> Prefixer<'a> for Pair {
    fn prefix(&self) -> Vec<Key> {
        vec![Key::Val128(self.to_bytes())]
    }
}

// allow Pair as part of key of cw_storage_plus::Map
impl<'a> PrimaryKey<'a> for Pair {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        vec![Key::Val128(self.to_bytes())]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Copy)]
pub struct Position {
    // an account can have a long position and a short position for the same pair at the same time. These two positions
    // will be represented in two Position entries, with opposite `direction` field.
    pub direction: PositionDirection,
    // these values should always be non-negative. A negative value would indicate a bug that breaks certain assumptions
    pub quantity: SignedDecimal,
    // the aggregated amount (in price denom) that the account borrowed to establish the position
    pub total_margin_debt: SignedDecimal,
    // the aggregated amount (in price denom) that the account paid to establish the position, including both
    // borrowed fund and out-of-pocket fund
    pub total_cost: SignedDecimal,
    // the last time the position has been charged/paid with funding payment (e.g. due to liquidation, etc.)
    pub last_funding_payment_epoch: i64,
    // the last paid cumulative funding rate for the position
    // used to calculate remaining payment amount by finding the difference with the current cumulative funding rate
    pub last_paid_funding_payment_rate: SignedDecimal,
}

pub fn opposite_direction(direction: PositionDirection) -> PositionDirection {
    match direction {
        PositionDirection::Long => PositionDirection::Short,
        PositionDirection::Short => PositionDirection::Long,
        PositionDirection::Unknown => PositionDirection::Unknown,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarginRatios {
    pub initial: Decimal,
    pub partial: Decimal,
    pub maintenance: Decimal,
}
