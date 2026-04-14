// service_types.rs

use serde::{Deserialize, Serialize};

// Order actions
pub const ORDER_NO: &str = "order_no";
pub const PREVIEW_ORDER: &str = "preview_order";
pub const PLACE_ORDER: &str = "place_order";
pub const CANCEL_ORDER: &str = "cancel_order";
pub const MODIFY_ORDER: &str = "modify_order";

// Account / Asset
pub const ACCOUNTS: &str = "accounts";
pub const ASSETS: &str = "assets";
pub const PRIME_ASSETS: &str = "prime_assets";
pub const POSITIONS: &str = "positions";
pub const ORDERS: &str = "orders";
pub const ACTIVE_ORDERS: &str = "active_orders";
pub const INACTIVE_ORDERS: &str = "inactive_orders";
pub const FILLED_ORDERS: &str = "filled_orders";
pub const ORDER_TRANSACTIONS: &str = "order_transactions";
pub const ANALYTICS_ASSET: &str = "analytics_asset";
pub const USER_LICENSE: &str = "user_license";
pub const ESTIMATE_TRADABLE_QUANTITY: &str = "estimate_tradable_quantity";
pub const SEGMENT_FUND_HISTORY: &str = "segment_fund_history";
pub const SEGMENT_FUND_AVAILABLE: &str = "segment_fund_available";
pub const TRANSFER_SEGMENT_FUND: &str = "transfer_segment_fund";
pub const PLACE_FOREX_ORDER: &str = "place_forex_order";

// Contract
pub const CONTRACT: &str = "contract";
pub const CONTRACTS: &str = "contracts";
pub const QUOTE_CONTRACT: &str = "quote_contract";

// Market
pub const MARKET_STATE: &str = "market_state";
pub const ALL_SYMBOLS: &str = "all_symbols";
pub const ALL_SYMBOL_NAMES: &str = "all_symbol_names";
pub const BRIEF: &str = "brief";
pub const STOCK_DETAIL: &str = "stock_detail";
pub const TIMELINE: &str = "timeline";
pub const HISTORY_TIMELINE: &str = "history_timeline";
pub const KLINE: &str = "kline";
pub const TRADE_TICK: &str = "trade_tick";
pub const QUOTE_REAL_TIME: &str = "quote_real_time";
pub const QUOTE_DELAY: &str = "quote_delay";
pub const QUOTE_SHORTABLE_STOCKS: &str = "quote_shortable_stocks";
pub const QUOTE_STOCK_TRADE: &str = "quote_stock_trade";
pub const QUOTE_DEPTH: &str = "quote_depth";
pub const GRAB_QUOTE_PERMISSION: &str = "grab_quote_permission";
pub const MARKET_SCANNER: &str = "market_scanner";
pub const GET_QUOTE_PERMISSION: &str = "get_quote_permission";
pub const TRADING_CALENDAR: &str = "trading_calendar";
pub const STOCK_BROKER: &str = "stock_broker";
pub const CAPITAL_DISTRIBUTION: &str = "capital_distribution";
pub const CAPITAL_FLOW: &str = "capital_flow";
pub const KLINE_QUOTA: &str = "kline_quota";

// Option
pub const OPTION_EXPIRATION: &str = "option_expiration";
pub const OPTION_CHAIN: &str = "option_chain";
pub const OPTION_BRIEF: &str = "option_brief";
pub const OPTION_KLINE: &str = "option_kline";
pub const OPTION_TRADE_TICK: &str = "option_trade_tick";
pub const WARRANT_FILTER: &str = "warrant_filter";
pub const WARRANT_REAL_TIME_QUOTE: &str = "warrant_real_time_quote";

// Future
pub const FUTURE_EXCHANGE: &str = "future_exchange";
pub const FUTURE_CONTRACT_BY_CONTRACT_CODE: &str = "future_contract_by_contract_code";
pub const FUTURE_CONTRACT_BY_EXCHANGE_CODE: &str = "future_contract_by_exchange_code";
pub const FUTURE_CONTRACTS: &str = "future_contracts";
pub const FUTURE_CONTINUOUS_CONTRACTS: &str = "future_continuous_contracts";
pub const FUTURE_CURRENT_CONTRACT: &str = "future_current_contract";
pub const FUTURE_KLINE: &str = "future_kline";
pub const FUTURE_REAL_TIME_QUOTE: &str = "future_real_time_quote";
pub const FUTURE_TICK: &str = "future_tick";
pub const FUTURE_TRADING_DATE: &str = "future_trading_date";

// Financial
pub const FINANCIAL_DAILY: &str = "financial_daily";
pub const FINANCIAL_REPORT: &str = "financial_report";
pub const CORPORATE_ACTION: &str = "corporate_action";

// Industry
pub const INDUSTRY_LIST: &str = "industry_list";
pub const INDUSTRY_STOCKS: &str = "industry_stocks";
pub const STOCK_INDUSTRY: &str = "stock_industry";


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub account_id: String,
    pub segments: Vec<Segment>,
    pub update_timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub buying_power: f64,
    pub capability: String,
    pub cash_available_for_trade: f64,
    pub cash_balance: f64,
    pub category: String,
    pub consolidated_seg_types: Vec<String>,
    pub currency: String,
    pub currency_assets: Vec<CurrencyAsset>,
    pub equity_with_loan: f64,
    pub excess_liquidation: f64,
    pub gross_position_value: f64,
    pub init_margin: f64,
    pub leverage: f64,
    pub locked_funds: f64,
    pub maintain_margin: f64,
    pub net_liquidation: f64,
    pub overnight_liquidation: f64,
    pub overnight_margin: f64,
    #[serde(rename = "realizedPL")]
    pub realized_pl: f64,
    #[serde(rename = "totalTodayPL")]
    pub total_today_pl: f64,
    #[serde(rename = "unrealizedPL")]
    pub unrealized_pl: f64,
    #[serde(rename = "unrealizedPLByCostOfCarry")]
    pub unrealized_pl_by_cost_of_carry: f64,
    pub uncollected: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyAsset {
    pub cash_available_for_trade: f64,
    pub cash_balance: f64,
    pub currency: String,
}