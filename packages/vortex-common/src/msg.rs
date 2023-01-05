use std::collections::HashSet;

use crate::{
    error::ContractError,
    types::{
        i32_to_direction, i32_to_order_type, GetPositionQuery, GetPositionsQuery, MarginRatios,
        Order, OrderType, PositionDirection, PositionEffect,
    },
    utils::SignedDecimal,
};
use cosmwasm_std::{Addr, Coin, Decimal};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub whitelist: Vec<String>,
    pub denoms: Vec<String>,
    pub supported_collateral_denoms: Vec<String>,
    pub supported_multicollateral_denoms: Vec<String>,
    pub full_denom_mapping: Vec<(String, String, Decimal)>,
    pub oracle_denom_mapping: Vec<(String, String, Decimal)>,
    pub use_whitelist: bool,
    pub multicollateral_whitelist: Vec<Addr>,
    pub multicollateral_whitelist_enable: bool,
    pub admin: String,
    pub limit_order_fee: SignedDecimal,
    pub market_order_fee: SignedDecimal,
    pub liquidation_order_fee: SignedDecimal,
    pub max_leverage: SignedDecimal,
    pub funding_payment_lookback: u64,
    pub native_token: String,
    pub default_base: String,
    pub spot_market_contract: Addr,
    pub funding_payment_pairs: Vec<(String, String)>,
    pub default_margin_ratios: MarginRatios,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Deposit {},
    Withdraw {
        coins: Vec<Coin>,
    },
    WithdrawInsuranceFund {
        coin: Coin,
    },
    SwapMulticollateralToBase {
        orders: Vec<OrderPlacement>,
    },
    UseWhitelist(bool),
    AddToCW20DenomMapping {
        address: String,
        denom: String,
    },
    AddToFullDenomMapping {
        full_denom: String,
        internal_denom: String,
        conversion_rate: Decimal,
    },
    AddToOracleDenomMapping {
        oracle_denom: String,
        internal_denom: String,
        conversion_rate: Decimal,
    },
    AddToWhitelist {
        converter: String,
    },
    AddToSupportedMultiCollateralDenoms {
        denom: String,
    },
    AddToFundingPaymentPairs {
        price_denom: String,
        asset_denom: String,
    },
    RemoveFromWhitelist {
        converter: String,
    },
    AddDenom {
        denom: String,
    },
    RemoveDenom {
        denom: String,
    },
    UpdateMarginRatio {
        margin_ratio: MarginRatios,
    },
    UpdateMaxLeverage {
        max_leverage: SignedDecimal,
    },
    UpdateMarketOrderFee {
        market_order_fee: SignedDecimal,
    },
    UpdateLimitOrderFee {
        limit_order_fee: SignedDecimal,
    },
    UpdateLiquidationOrderFee {
        liquidation_order_fee: SignedDecimal,
    },
    UpdateAdmin {
        admin: String,
    },
    UpdateFundingPaymentLookback {
        funding_payment_lookback: u64,
    },
    UpdateNativeToken {
        native_token: String,
    },
    UpdateBase {
        default_base: String,
    },
    UpdateSpotMarketContract {
        contract_addr: String,
    },
    UpdateMultiCollateralWhitelist {
        whitelist: Vec<Addr>,
        whitelist_enable: bool,
    },
    Liquidate {
        account: Addr,
        multicollateral_liquidation: bool,
    },
    CreateDenom {
        denom_name: String,
    },
    MintDenom {
        denom_name: String,
        denom_amount: u128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DecimalCoin {
    pub denom: String,
    pub amount: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    Settlement {
        epoch: i64,
        entries: Vec<SettlementEntry>,
    },

    NewBlock {
        epoch: i64,
    },

    BulkOrderPlacements {
        orders: Vec<OrderPlacement>,
        deposits: Vec<DepositInfo>,
    },

    BulkOrderCancellations {
        ids: Vec<u64>,
    },

    Liquidation {
        requests: Vec<LiquidationRequest>,
    },

    FinalizeBlock {
        contract_order_results: Vec<ContractOrderResult>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetBalance {
        account: String,
        symbol: String,
    },

    GetBalances {
        account: String,
    },

    GetFundingPaymentRates {
        price_denom: String,
        asset_denom: String,
        start_epoch: i64,
        end_epoch: i64,
    },

    GetPosition(GetPositionQuery),

    GetPositions(GetPositionsQuery),

    GetOrder {
        account: String,
        price_denom: String,
        asset_denom: String,
    },

    GetPortfolioSpecs {
        account: String,
    },

    GetInsuranceFundBalance {
        denom: String,
    },

    GetOrderEstimate {
        order: Order,
    },

    GetConfig {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct GetBalanceResponse {
    pub amount: SignedDecimal,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct GetBalancesResponse {
    pub symbols: Vec<String>,
    pub amounts: Vec<SignedDecimal>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct GetPositionResponse {
    pub long_position: SignedDecimal,
    pub long_position_margin_debt: SignedDecimal,
    pub long_position_last_funding_payment_epoch: i64,
    pub long_position_pnl: SignedDecimal,
    pub short_position: SignedDecimal,
    pub short_position_margin_debt: SignedDecimal,
    pub short_position_last_funding_payment_epoch: i64,
    pub short_position_pnl: SignedDecimal,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct GetPositionsResponse {
    pub positions: Vec<GetPositionResponse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct GetPortfolioSpecsResponse {
    pub equity: SignedDecimal,
    pub total_position_value: SignedDecimal,
    pub buying_power: SignedDecimal,
    pub unrealized_pnl: SignedDecimal,
    pub leverage: SignedDecimal,
    pub balance: SignedDecimal,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct GetInsuranceFundBalanceResponse {
    pub balance: SignedDecimal,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct GetOrderResponse {
    pub orders: Vec<Order>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct GetFundingPaymentRatesResponse {
    pub price_diffs: Vec<Decimal>,
    pub negatives: Vec<bool>,
    pub epochs: Vec<i64>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct GetOrderEstimateResponse {
    pub order_fee_estimate: SignedDecimal,
    pub deposits_required: Coin,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct GetConfigResponse {
    pub admin: String,
    pub whitelist: HashSet<Addr>,
    pub use_whitelist: bool,
    pub limit_order_fee: SignedDecimal,
    pub market_order_fee: SignedDecimal,
    pub liquidation_order_fee: SignedDecimal,
    pub default_margin_ratios: MarginRatios,
    pub max_leverage: SignedDecimal,
    pub spot_market_contract: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct BulkOrderPlacementsResponse {
    pub unsuccessful_orders: Vec<UnsuccessfulOrder>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct UnsuccessfulOrder {
    pub id: u64,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct LiquidationResponse {
    pub successful_accounts: Vec<String>,
    pub liquidation_orders: Vec<OrderPlacement>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SettlementEntry {
    pub account: String,
    pub price_denom: String,
    pub asset_denom: String,
    pub quantity: Decimal,
    pub execution_cost_or_proceed: Decimal,
    pub expected_cost_or_proceed: Decimal,
    pub position_direction: PositionDirection,
    pub order_type: OrderType,
    pub order_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderPlacement {
    pub id: u64,
    pub status: i32,
    pub account: String,
    pub contract_address: String,
    pub price_denom: String,
    pub asset_denom: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub order_type: i32,
    pub position_direction: i32,
    pub data: String,
    pub status_description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderData {
    pub leverage: Decimal,
    pub position_effect: PositionEffect,
}

impl OrderPlacement {
    pub fn to_order(&self) -> Result<Order, ContractError> {
        let order_data: OrderData = match serde_json_wasm::from_str(&self.data) {
            Ok(data) => data,
            Err(_) => return Result::Err(ContractError::InvalidOrderData {}),
        };
        let order = Order {
            id: self.id,
            account: self.account.to_owned(),
            price_denom: self.price_denom.to_owned(),
            asset_denom: self.asset_denom.to_owned(),
            price: SignedDecimal::new(self.price),
            quantity: SignedDecimal::new(self.quantity),
            remaining_quantity: SignedDecimal::new(self.quantity),
            direction: i32_to_direction(self.position_direction),
            order_type: i32_to_order_type(self.order_type),
            effect: order_data.position_effect,
            leverage: SignedDecimal::new(order_data.leverage),
        };
        Result::Ok(order)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositInfo {
    pub account: String,
    pub denom: String,
    pub amount: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidationRequest {
    pub requestor: String,
    pub account: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractOrderResult {
    pub contract_address: String,
    pub order_placement_results: Vec<OrderPlacementResult>,
    pub order_execution_results: Vec<OrderExecutionResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderPlacementResult {
    pub order_id: u64,
    pub status_code: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderExecutionResult {
    pub order_id: u64,
    pub execution_price: Decimal,
    pub executed_quantity: Decimal,
    pub total_notional: Decimal,
    pub position_direction: String,
}
