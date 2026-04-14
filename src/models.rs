// src/models.rs
// 推荐文件名 models.rs（复数形式），因为是多个模型的集合

use chrono::{DateTime, NaiveDate, Utc};
use libc::time_t;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contract {
    pub symbol: Option<String>,
    pub currency: Option<String>,
    pub sec_type: Option<String>,
    pub exchange: Option<String>,
    pub local_symbol: Option<String>,
    pub expiry: Option<String>,
    pub strike: Option<String>,
    pub right: Option<String>,
    pub multiplier: Option<i32>,
    pub contract_month: Option<String>,
    pub contract_id: Option<i64>,
    pub identifier: Option<String>,
    pub market: Option<String>,
}

impl Contract {
    /// 默认构造函数
    pub fn new() -> Self {
        Self {
            symbol: None,
            currency: None,
            sec_type: None,
            exchange: None,
            local_symbol: None,
            expiry: None,
            strike: None,
            right: None,
            multiplier: None,
            contract_month: None,
            contract_id: None,
            identifier: None,
            market: None,
        }
    }

    /// 基础构造函数 (sec_type, symbol, currency, local_symbol, exchange, contract_id)
    pub fn basic(
        sec_type: String,
        symbol: String,
        currency: String,
        local_symbol: String,
        exchange: String,
        contract_id: i64,
    ) -> Self {
        let mut c = Self::new();
        c.sec_type = Some(sec_type);
        c.symbol = Some(symbol);
        c.currency = Some(currency);
        c.local_symbol = Some(local_symbol);
        c.exchange = Some(exchange);
        c.contract_id = Some(contract_id);
        c
    }

    /// 期权构造函数 (sec_type, symbol, expiry, strike, right, currency, multiplier, local_symbol, contract_id)
    pub fn option(
        sec_type: String,
        symbol: String,
        expiry: String,
        strike: String,
        right: String,
        currency: String,
        multiplier: i32,
        local_symbol: String,
        contract_id: i64,
    ) -> Self {
        let mut c = Self::new();
        c.sec_type = Some(sec_type);
        c.symbol = Some(symbol);
        c.expiry = Some(expiry);
        c.strike = Some(strike);
        c.right = Some(right);
        c.currency = Some(currency);
        c.multiplier = Some(multiplier);
        c.local_symbol = Some(local_symbol);
        c.contract_id = Some(contract_id);
        c
    }

    /// 期货构造函数 (sec_type, symbol, expiry, multiplier, contract_month, currency, exchange, local_symbol)
    pub fn future(
        sec_type: String,
        symbol: String,
        expiry: String,
        multiplier: i32,
        contract_month: String,
        currency: String,
        exchange: String,
        local_symbol: String,
    ) -> Self {
        let mut c = Self::new();
        c.sec_type = Some(sec_type);
        c.symbol = Some(symbol);
        c.expiry = Some(expiry);
        c.multiplier = Some(multiplier);
        c.contract_month = Some(contract_month);
        c.currency = Some(currency);
        c.exchange = Some(exchange);
        c.local_symbol = Some(local_symbol);
        c
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    #[serde(rename = "MKT")]
    Market,
    #[serde(rename = "LMT")]
    Limit,
    #[serde(rename = "STP")]
    Stop,
    #[serde(rename = "STP_LMT")]
    StopLimit,
    #[serde(rename = "TRAIL")]
    Trail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum Action {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum TimeInForce {
    #[serde(rename = "DAY")]
    Day,
    #[serde(rename = "GTC")]
    GoodTillCancel,
    #[serde(rename = "GTD")]
    GoodTillDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub contract: Contract,
    pub account: String,
    pub id: Option<u64>,
    pub order_id: Option<i64>,
    pub order_type: OrderType,
    pub action: Action,
    pub total_quantity: i64,
    pub total_quantity_scale: Option<i32>,
    pub limit_price: Option<f64>,
    pub s_limit_price: Option<String>,
    pub aux_price: Option<f64>,
    pub trail_stop_price: Option<f64>,
    pub trailing_percent: Option<f64>,
    pub percent_offset: Option<f64>,
    pub time_in_force: TimeInForce,
    pub outside_rth: bool,
    pub adjust_limit: f64,
    pub user_mark: Option<String>,
    pub expire_time: Option<time_t>,
    pub status: Option<String>,
    pub parent_id: Option<u64>,
    pub open_time: Option<time_t>,
    pub reason: Option<String>,
    pub latest_time: Option<time_t>,
    pub update_time: Option<time_t>,
    pub filled_quantity: Option<i64>,
    pub filled_quantity_scale: Option<i32>,
    pub avg_fill_price: Option<f64>,
    pub realized_pnl: Option<f64>,
    pub secret_key: Option<String>,
    pub sub_ids: Option<serde_json::Value>,
    pub algo_strategy: Option<String>,
    pub commission: Option<f64>,
    pub gst: Option<f64>,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Order(id={:?} status={:?} total_quantity: {} limit_price: {:?})",
            self.id, self.status, self.total_quantity, self.limit_price
        )
    }
}

impl Order {
    /// 基础构造函数
    pub fn new(
        contract: Contract,
        account: String,
        order_type: OrderType,
        action: Action,
        total_quantity: i64,
        time_in_force: TimeInForce,
    ) -> Self {
        Self {
            contract,
            account,
            id: None,
            order_id: None,
            order_type,
            action,
            total_quantity,
            total_quantity_scale: None,
            limit_price: None,
            s_limit_price: None,
            aux_price: None,
            trail_stop_price: None,
            trailing_percent: None,
            percent_offset: None,
            time_in_force,
            outside_rth: false,
            adjust_limit: 0.0,
            user_mark: None,
            expire_time: None,
            status: None,
            parent_id: None,
            open_time: None,
            reason: None,
            latest_time: None,
            update_time: None,
            filled_quantity: None,
            filled_quantity_scale: None,
            avg_fill_price: None,
            realized_pnl: None,
            secret_key: None,
            sub_ids: None,
            algo_strategy: None,
            commission: None,
            gst: None,
        }
    }

    /// 创建一个限价买单
    pub fn limit_buy(
        contract: Contract,
        account: String,
        total_quantity: i64,
        limit_price: f64,
        time_in_force: TimeInForce,
    ) -> Self {
        let mut order = Self::new(
            contract,
            account,
            OrderType::Limit,
            Action::Buy,
            total_quantity,
            time_in_force,
        );
        order.limit_price = Some(limit_price);
        order
    }

    /// 创建一个市价买单
    pub fn market_buy(
        contract: Contract,
        account: String,
        total_quantity: i64,
        time_in_force: TimeInForce,
    ) -> Self {
        Self::new(
            contract,
            account,
            OrderType::Market,
            Action::Buy,
            total_quantity,
            time_in_force,
        )
    }

    /// 创建一个限价卖单
    pub fn limit_sell(
        contract: Contract,
        account: String,
        total_quantity: i64,
        limit_price: f64,
        time_in_force: TimeInForce,
    ) -> Self {
        let mut order = Self::new(
            contract,
            account,
            OrderType::Limit,
            Action::Sell,
            total_quantity,
            time_in_force,
        );
        order.limit_price = Some(limit_price);
        order
    }

    /// 创建一个市价卖单
    pub fn market_sell(
        contract: Contract,
        account: String,
        total_quantity: i64,
        time_in_force: TimeInForce,
    ) -> Self {
        Self::new(
            contract,
            account,
            OrderType::Market,
            Action::Sell,
            total_quantity,
            time_in_force,
        )
    }

    pub fn to_value(&self, account_param: Value, secret_key: Option<String>) -> Map<String, Value> {
        let mut obj = Map::new();
        let c = &self.contract;

        // Contract 部分
        if let Some(symbol) = &c.symbol {
            obj.insert("symbol".into(), json!(symbol));
        }
        if let Some(currency) = &c.currency {
            obj.insert("currency".into(), json!(currency));
        }
        if let Some(sec_type) = &c.sec_type {
            obj.insert("sec_type".into(), json!(sec_type));
        }
        if let Some(exchange) = &c.exchange {
            obj.insert("exchange".into(), json!(exchange));
        }
        if let Some(expiry) = &c.expiry {
            obj.insert("expiry".into(), json!(expiry));
        }
        if let Some(strike) = &c.strike {
            obj.insert("strike".into(), json!(strike));
        }
        if let Some(right) = &c.right {
            obj.insert("right".into(), json!(right));
        }
        if let Some(multiplier) = c.multiplier
            && multiplier != 0
        {
            obj.insert("multiplier".into(), json!(multiplier));
        }

        // Order 部分
        obj.insert("account".into(), account_param);
        if let Some(sk) = secret_key {
            obj.insert("secret_key".into(), json!(sk));
        }
        if let Some(order_id) = self.order_id
            && order_id != 0
        {
            obj.insert("order_id".into(), json!(order_id));
        }
        if let Some(id) = self.id
            && id != 0
        {
            obj.insert("id".into(), json!(id));
        }

        obj.insert("order_type".into(), json!(self.order_type));

        obj.insert("action".into(), json!(self.action));

        if self.total_quantity != 0 {
            obj.insert("total_quantity".into(), json!(self.total_quantity));
        }
        if let Some(limit_price) = self.limit_price
            && limit_price != 0.0
        {
            obj.insert("limit_price".into(), json!(limit_price));
        }
        if let Some(s_limit_price) = &self.s_limit_price
            && !s_limit_price.is_empty()
        {
            obj.insert("limit_price".into(), json!(s_limit_price));
        }
        if let Some(aux_price) = self.aux_price
            && aux_price != 0.0
        {
            obj.insert("aux_price".into(), json!(aux_price));
        }
        if let Some(trail_stop_price) = self.trail_stop_price
            && trail_stop_price != 0.0
        {
            obj.insert("trail_stop_price".into(), json!(trail_stop_price));
        }
        if let Some(trailing_percent) = self.trailing_percent
            && trailing_percent != 0.0
        {
            obj.insert("trailing_percent".into(), json!(trailing_percent));
        }
        if let Some(percent_offset) = self.percent_offset
            && percent_offset != 0.0
        {
            obj.insert("percent_offset".into(), json!(percent_offset));
        }

        obj.insert("time_in_force".into(), json!(self.time_in_force));

        if self.outside_rth {
            obj.insert("outside_rth".into(), json!(self.outside_rth));
        }
        if self.adjust_limit != 0.0 {
            obj.insert("adjust_limit".into(), json!(self.adjust_limit));
        }
        if let Some(user_mark) = &self.user_mark {
            obj.insert("user_mark".into(), json!(user_mark));
        }
        if let Some(expire_time) = self.expire_time {
            obj.insert("expire_time".into(), json!(expire_time));
        }

        obj
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub account: String,
    pub contract: Contract,
    pub position: i64,
    pub average_cost: f64,
    pub market_value: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub latest_price: f64,
    pub status: i32,
    pub update_timestamp: Option<time_t>,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Position(symbol={:?} position={})",
            self.contract.symbol, self.position
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyAsset {
    pub cash_available_for_trade: f64,
    pub cash_balance: f64,
    pub currency: String,
    pub gross_position_value: f64,
    pub option_market_value: f64,
    pub realized_pl: f64,
    pub stock_market_value: f64,
    pub unrealized_pl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub category: String,
    pub capability: String,
    pub currency: String,
    pub buying_power: f64,
    pub cash_available_for_trade: f64,
    pub cash_available_for_withdrawal: f64,
    pub cash_balance: f64,
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
    pub realized_pl: f64,
    pub total_today_pl: f64,
    pub unrealized_pl: f64,
    pub unrealized_pl_by_cost_of_carry: f64,
    pub currency_assets: Vec<CurrencyAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioAccount {
    pub account: String,
    pub update_timestamp: Option<time_t>,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KlineItem {
    pub amount: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub time: Option<time_t>,
    pub volume: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Kline {
    pub items: Vec<KlineItem>,
    pub symbol: String,
    pub contract_code: Option<String>,
    pub period: String,
    pub expiry: Option<time_t>,
    pub right: Option<String>,
    pub strike: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeQuote {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub adj_pre_close: f64,
    pub pre_close: f64,
    pub ask_price: f64,
    pub ask_size: f64,
    pub bid_price: f64,
    pub bid_size: f64,
    pub latest_price: f64,
    pub latest_time: Option<time_t>,
    pub latest_size: i64,
    pub status: String,
    pub symbol: String,
    pub contract_code: Option<String>,
    pub open_interest: i64,
    pub limit_down: i32,
    pub limit_up: i32,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct TradeTickItem {
//     pub tick_type: String,
//     pub price: f64,
//     pub volume: i32,
//     pub part_code: String,
//     pub part_code_name: String,
//     pub cond: String,
//     pub time: String,
//     pub sn: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct TradeTick {
//     pub symbol: String,
//     pub sec_type: String,
//     pub quote_level: String,
//     pub timestamp: String,
//     pub ticks: Vec<TradeTickItem>,
// }

use chrono::TimeZone;
use std::str::FromStr;

// 单个交易数据项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeTickItem {
    #[serde(rename = "avgPrice")]
    pub avg_price: Option<f64>, // 平均价格
    pub price: f64,  // 当前价格，对历史数据而言，这里是每分钟收盘价
    pub volume: i64, // 成交量
    pub time: i64,   // 时间戳（毫秒）

    // 可选的额外字段（JSON 中可能不包含）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_code_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cond: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sn: Option<String>,
}

impl TradeTickItem {
    pub fn new(price: f64, volume: i64, time: i64) -> Self {
        Self {
            avg_price: None,
            price,
            volume,
            time,
            tick_type: None,
            part_code: None,
            part_code_name: None,
            cond: None,
            sn: None,
        }
    }

    // Getter 方法
    pub fn get_price(&self) -> f64 {
        self.price
    }

    pub fn get_volume(&self) -> i64 {
        self.volume
    }

    pub fn get_time(&self) -> i64 {
        self.time
    }

    pub fn get_avg_price(&self) -> Option<f64> {
        self.avg_price
    }

    // 获取格式化的时间字符串
    pub fn get_formatted_time(&self, format: &str) -> String {
        let dt = Utc.timestamp_millis_opt(self.time).unwrap();
        dt.format(format).to_string()
    }

    // 获取 DateTime 对象
    pub fn get_datetime(&self) -> DateTime<Utc> {
        Utc.timestamp_millis_opt(self.time).unwrap()
    }
}

impl fmt::Display for TradeTickItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let time_str = self.get_formatted_time("%Y-%m-%d %H:%M:%S");
        write!(
            f,
            "TradeTickItem<{{price: {}, volume: {}, time: {}, avg_price: {:?}}}>",
            self.price, self.volume, time_str, self.avg_price
        )
    }
}

// 交易数据容器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeTick {
    pub symbol: String,            // 股票代码
    pub items: Vec<TradeTickItem>, // 交易数据项

    // 可选的额外字段
    #[serde(rename = "sec_type", skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(rename = "quote_level", skip_serializing_if = "Option::is_none")]
    pub quote_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl TradeTick {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            items: Vec::new(),
            sec_type: None,
            quote_level: None,
            timestamp: None,
        }
    }

    // 从 JSON 字符串解析
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    // 解析你提供的特定格式
    pub fn parse_tick_data(json_str: &str) -> Result<Self, serde_json::Error> {
        // 先解析为中间结构
        #[derive(Deserialize)]
        struct RawData {
            items: Vec<TradeTickItem>,
            symbol: String,
        }

        let raw: RawData = serde_json::from_str(json_str)?;

        Ok(Self {
            symbol: raw.symbol,
            items: raw.items,
            sec_type: None,
            quote_level: None,
            timestamp: None,
        })
    }

    // Getter 方法
    pub fn get_symbol(&self) -> &str {
        &self.symbol
    }

    pub fn get_ticks(&self) -> &[TradeTickItem] {
        &self.items
    }

    // 添加交易项
    pub fn add_tick(&mut self, tick: TradeTickItem) {
        self.items.push(tick);
    }

    // 按时间范围过滤
    pub fn filter_by_time_range(&self, start_time: i64, end_time: i64) -> Vec<&TradeTickItem> {
        self.items
            .iter()
            .filter(|item| item.time >= start_time && item.time <= end_time)
            .collect()
    }

    // 获取成交量总和
    pub fn total_volume(&self) -> i64 {
        self.items.iter().map(|item| item.volume).sum()
    }

    // 获取最新价格
    pub fn latest_price(&self) -> Option<f64> {
        self.items.last().map(|item| item.price)
    }

    // 转换为 JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl fmt::Display for TradeTick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TradeTick<{{symbol: {}, ticks: [", self.symbol)?;

        for (i, tick) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", tick)?;
        }

        write!(f, "]}}>")
    }
}

impl FromStr for TradeTick {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TradeTick::parse_tick_data(s)
    }
}

// 批量查询的交易数据响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTradeTickResponse {
    #[serde(default)]
    pub data: Vec<TradeTick>,

    #[serde(default)]
    pub code: Option<i32>,

    #[serde(default)]
    pub msg: Option<String>,

    #[serde(default)]
    pub timestamp: Option<i64>,
}

impl BatchTradeTickResponse {
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn is_success(&self) -> bool {
        self.code.unwrap_or(-1) == 0
    }

    pub fn merge_all(&self) -> Vec<TradeTickItem> {
        self.data
            .iter()
            .flat_map(|tick| tick.items.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sample_data() {
        let json_str = r#"[{"items":[{"avgPrice":150.924,"price":150.5,"time":1767576600000,"volume":3873461},{"avgPrice":152.337,"price":153.0,"time":1767591840000,"volume":67100},{"avgPrice":152.461,"price":152.6,"time":1767599880000,"volume":245400},{"avgPrice":152.462,"price":152.8,"time":1767599940000,"volume":4676300}],"symbol":"09988"}]"#;

        // 解析批量数据
        let ticks: Vec<TradeTick> = serde_json::from_str(json_str).unwrap();

        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0].symbol, "09988");
        assert_eq!(ticks[0].items.len(), 4);

        // 使用 parse_tick_data
        let json_str_single = r#"{"items":[{"avgPrice":150.924,"price":150.5,"time":1767576600000,"volume":3873461}],"symbol":"09988"}"#;
        let tick = TradeTick::parse_tick_data(json_str_single).unwrap();
        assert_eq!(tick.symbol, "09988");
        assert_eq!(tick.items[0].price, 150.5);

        // 使用 FromStr trait
        let tick: TradeTick = json_str_single.parse().unwrap();
        assert_eq!(tick.symbol, "09988");
    }

    #[test]
    fn test_tick_item_methods() {
        let item = TradeTickItem {
            avg_price: Some(150.924),
            price: 150.5,
            volume: 3873461,
            time: 1767576600000,
            tick_type: Some("T".to_string()),
            part_code: None,
            part_code_name: None,
            cond: None,
            sn: None,
        };

        assert_eq!(item.get_price(), 150.5);
        assert_eq!(item.get_volume(), 3873461);

        let time_str = item.get_formatted_time("%Y-%m-%d");
        println!("Formatted time: {}", time_str);
    }

    #[test]
    fn test_trade_tick_methods() {
        let json_str = r#"{"items":[{"avgPrice":150.924,"price":150.5,"time":1767576600000,"volume":3873461},{"avgPrice":152.337,"price":153.0,"time":1767591840000,"volume":67100}],"symbol":"09988"}"#;

        let tick: TradeTick = json_str.parse().unwrap();

        assert_eq!(tick.get_symbol(), "09988");
        assert_eq!(tick.get_ticks().len(), 2);
        assert_eq!(tick.total_volume(), 3873461 + 67100);

        if let Some(price) = tick.latest_price() {
            assert_eq!(price, 153.0);
        }

        // 测试添加
        let mut tick = TradeTick::new("0700");
        tick.add_tick(TradeTickItem::new(320.0, 1000, 1767576600000));
        assert_eq!(tick.items.len(), 1);
    }
}
