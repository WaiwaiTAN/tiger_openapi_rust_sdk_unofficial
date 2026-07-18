// Translated from C++ TIGERAPI_ENUMS.h to Rust
#![allow(non_camel_case_types)]

use lazy_static::lazy_static;
use std::collections::HashMap;

pub const HK_QUOTE_LEVEL_PREFIX: &str = "hk";
pub const US_QUOTE_LEVEL_PREFIX: &str = "us";

#[derive(Debug)]
pub enum Market {
    ALL,
    US,
    HK,
    CN,
    SG,
}

impl Market {
    pub fn to_str(&self) -> &'static str {
        match self {
            Market::ALL => "ALL",
            Market::US => "US",
            Market::HK => "HK",
            Market::CN => "CN",
            Market::SG => "SG",
        }
    }
}

#[derive(Debug)]
pub enum License {
    TBNZ,
    TBSG,
    TBHK,
    TBAU,
    TBUS,
}

impl License {
    pub fn to_str(&self) -> &'static str {
        match self {
            License::TBNZ => "TBNZ",
            License::TBSG => "TBSG",
            License::TBHK => "TBHK",
            License::TBAU => "TBAU",
            License::TBUS => "TBUS",
        }
    }
}

#[derive(Debug)]
pub enum TradingSession {
    PreMarket,
    Regular,
    AfterHours,
}

impl TradingSession {
    pub fn to_str(&self) -> &'static str {
        match self {
            TradingSession::PreMarket => "PreMarket",
            TradingSession::Regular => "Regular",
            TradingSession::AfterHours => "AfterHours",
        }
    }
}

#[derive(Debug)]
pub enum SecType {
    ALL,
    STK,
    OPT,
    WAR,
    IOPT,
    FUT,
    FOP,
    CASH,
}

impl SecType {
    pub fn to_str(&self) -> &'static str {
        match self {
            SecType::ALL => "",
            SecType::STK => "STK",
            SecType::OPT => "OPT",
            SecType::WAR => "WAR",
            SecType::IOPT => "IOPT",
            SecType::FUT => "FUT",
            SecType::FOP => "FOP",
            SecType::CASH => "CASH",
        }
    }
}

#[derive(Debug)]
pub enum SegmentType {
    ALL,
    SEC,
    FUT,
}

impl SegmentType {
    pub fn to_str(&self) -> &'static str {
        match self {
            SegmentType::ALL => "ALL",
            SegmentType::SEC => "SEC",
            SegmentType::FUT => "FUT",
        }
    }
}

#[derive(Debug)]
pub enum Currency {
    ALL,
    USD,
    HKD,
    CNH,
    SGD,
}

impl Currency {
    pub fn to_str(&self) -> &'static str {
        match self {
            Currency::ALL => "ALL",
            Currency::USD => "USD",
            Currency::HKD => "HKD",
            Currency::CNH => "CNH",
            Currency::SGD => "SGD",
        }
    }
}

#[derive(Debug)]
pub enum Language {
    zh_CN,
    zh_TW,
    en_US,
}

impl Language {
    pub fn to_str(&self) -> &'static str {
        match self {
            Language::zh_CN => "zh_CN",
            Language::zh_TW => "zh_TW",
            Language::en_US => "en_US",
        }
    }
}

#[derive(Debug)]
pub enum QuoteRight {
    br,
    nr,
}

impl QuoteRight {
    pub fn to_str(&self) -> &'static str {
        match self {
            QuoteRight::br => "br",
            QuoteRight::nr => "nr",
        }
    }
}

#[derive(Debug)]
pub enum Right {
    PUT,
    CALL,
    ALL,
}

impl Right {
    pub fn to_str(&self) -> &'static str {
        match self {
            Right::ALL => "",
            Right::PUT => "PUT",
            Right::CALL => "CALL",
        }
    }
}

#[derive(Debug)]
pub enum TimelinePeriod {
    DAY = 1,
    FIVE_DAYS = 2,
}

pub const TIMELINE_PERIOD_NAMES: [&str; 3] = ["", "day", "5day"];

pub fn get_timeline_period_value(period: TimelinePeriod) -> &'static str {
    TIMELINE_PERIOD_NAMES[period as usize]
}

#[derive(Debug)]
pub enum BarPeriod {
    DAY,
    WEEK,
    MONTH,
    YEAR,
    ONE_MINUTE,
    THREE_MINUTES,
    FIVE_MINUTES,
    TEN_MINUTES,
    FIFTEEN_MINUTES,
    HALF_HOUR,
    FORTY_FIVE_MINUTES,
    ONE_HOUR,
    TWO_HOURS,
    THREE_HOURS,
    FOUR_HOURS,
    SIX_HOURS,
}

impl BarPeriod {
    pub fn to_str(&self) -> &'static str {
        match self {
            BarPeriod::DAY => "day",
            BarPeriod::WEEK => "week",
            BarPeriod::MONTH => "month",
            BarPeriod::YEAR => "year",
            BarPeriod::ONE_MINUTE => "1min",
            BarPeriod::THREE_MINUTES => "3min",
            BarPeriod::FIVE_MINUTES => "5min",
            BarPeriod::TEN_MINUTES => "10min",
            BarPeriod::FIFTEEN_MINUTES => "15min",
            BarPeriod::HALF_HOUR => "30min",
            BarPeriod::FORTY_FIVE_MINUTES => "45min",
            BarPeriod::ONE_HOUR => "60min",
            BarPeriod::TWO_HOURS => "2hour",
            BarPeriod::THREE_HOURS => "3hours",
            BarPeriod::FOUR_HOURS => "4hour",
            BarPeriod::SIX_HOURS => "6hour",
        }
    }
}

#[derive(Debug)]
pub enum CapitalPeriod {
    INTRADAY,
    DAY,
    WEEK,
    MONTH,
    YEAR,
    QUARTER,
    HALFAYEAR,
}

impl CapitalPeriod {
    pub fn to_str(&self) -> &'static str {
        match self {
            CapitalPeriod::INTRADAY => "intraday",
            CapitalPeriod::DAY => "day",
            CapitalPeriod::WEEK => "week",
            CapitalPeriod::MONTH => "month",
            CapitalPeriod::YEAR => "year",
            CapitalPeriod::QUARTER => "quarter",
            CapitalPeriod::HALFAYEAR => "6month",
        }
    }
}

/// Field used to sort history orders and interpret their time range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSortBy {
    LATEST_CREATED,
    LATEST_STATUS_UPDATED,
}

impl OrderSortBy {
    pub fn to_str(&self) -> &'static str {
        match self {
            OrderSortBy::LATEST_CREATED => "LATEST_CREATED",
            OrderSortBy::LATEST_STATUS_UPDATED => "LATEST_STATUS_UPDATED",
        }
    }
}

#[derive(Debug)]
pub enum OrderStatus {
    PendingNew,
    PendingSubmit,
    Initial,
    Submitted,
    PartiallyFilled,
    Filled,
    Cancelled,
    PendingCancel,
    Inactive,
    Invalid,
}

impl OrderStatus {
    pub fn to_str(&self) -> &'static str {
        match self {
            OrderStatus::PendingNew => "PendingNew",
            OrderStatus::PendingSubmit => "PendingSubmit",
            OrderStatus::Initial => "Initial",
            OrderStatus::Submitted => "Submitted",
            OrderStatus::PartiallyFilled => "PartiallyFilled",
            OrderStatus::Filled => "Filled",
            OrderStatus::Cancelled => "Cancelled",
            OrderStatus::PendingCancel => "PendingCancel",
            OrderStatus::Inactive => "Inactive",
            OrderStatus::Invalid => "Invalid",
        }
    }
}

#[derive(Debug)]
pub enum TickSizeType {
    OPEN,
    OPEN_CLOSED,
    CLOSED_OPEN,
    CLOSED,
}

impl TickSizeType {
    pub fn to_str(&self) -> &'static str {
        match self {
            TickSizeType::OPEN => "OPEN",
            TickSizeType::OPEN_CLOSED => "OPEN_CLOSED",
            TickSizeType::CLOSED_OPEN => "CLOSED_OPEN",
            TickSizeType::CLOSED => "CLOSED",
        }
    }
}

#[derive(Debug)]
pub enum ResponseType {
    GET_ORDER_NO_END = 1,
    PREVIEW_ORDER_END = 2,
    PLACE_ORDER_END = 3,
    CANCEL_ORDER_END = 4,
    MODIFY_ORDER_END = 5,
    GET_ASSET_END = 6,
    GET_POSITION_END = 7,
    GET_ACCOUNT_END = 8,
    SUBSCRIBE_ORDER_STATUS = 9,
    SUBSCRIBE_POSITION = 10,
    SUBSCRIBE_ASSET = 11,
    SUBSCRIBE_TRADE_EXECUTION = 12,

    GET_MARKET_STATE_END = 101,
    GET_ALL_SYMBOLS_END = 102,
    GET_ALL_SYMBOL_NAMES_END = 103,
    GET_BRIEF_INFO_END = 104,
    GET_STOCK_DETAIL_END = 105,
    GET_TIME_LINE_END = 106,
    GET_HOUR_TRADING_TIME_LINE_END = 107,
    GET_KLINE_END = 108,
    GET_TRADING_TICK_END = 109,
    GET_QUOTE_CHANGE_END = 110,

    GET_SUB_SYMBOLS_END = 111,
    GET_SUBSCRIBE_END = 112,
    GET_CANCEL_SUBSCRIBE_END = 113,

    ERROR_END = 200,
}

lazy_static! {
    pub static ref PART_CODE_NAME_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("a", "NYSE American, LLC (NYSE American)");
        m.insert("b", "NASDAQ OMX BX, Inc. (NASDAQ OMX BX)");
        m.insert("c", "NYSE National, Inc. (NYSE National)");
        m.insert("d", "FINRA Alternative Display Facility (ADF)");
        m.insert("h", "MIAX Pearl Exchange, LLC (MIAX)");
        m.insert("i", "International Securities Exchange, LLC (ISE)");
        m.insert("j", "Cboe EDGA Exchange, Inc. (Cboe EDGA)");
        m.insert("k", "Cboe EDGX Exchange, Inc. (Cboe EDGX)");
        m.insert("l", "Long-Term Stock Exchange, Inc. (LTSE)");
        m.insert("m", "NYSE Chicago, Inc. (NYSE Chicago)");
        m.insert("n", "New York Stock Exchange, LLC (NYSE)");
        m.insert("p", "NYSE Arca, Inc. (NYSE Arca)");
        m.insert("s", "Consolidated Tape System (CTS)");
        m.insert("t", "NASDAQ Stock Market, LLC (NASDAQ)");
        m.insert("u", "Members Exchange, LLC (MEMX)");
        m.insert("v", "Investors' Exchange, LLC. (IEX)");
        m.insert("w", "CBOE Stock Exchange, Inc. (CBSX)");
        m.insert("x", "NASDAQ OMX PSX, Inc. (NASDAQ OMX PSX)");
        m.insert("y", "Cboe BYX Exchange, Inc. (Cboe BYX)");
        m.insert("z", "Cboe BZX Exchange, Inc. (Cboe BZX)");
        m
    };
    pub static ref PART_CODE_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("a", "AMEX");
        m.insert("b", "BX");
        m.insert("c", "NSX");
        m.insert("d", "ADF");
        m.insert("h", "MIAX");
        m.insert("i", "ISE");
        m.insert("j", "EDGA");
        m.insert("k", "EDGX");
        m.insert("l", "LTSE");
        m.insert("m", "CHO");
        m.insert("n", "NYSE");
        m.insert("p", "ARCA");
        m.insert("s", "CTS");
        m.insert("t", "NSDQ");
        m.insert("u", "MEMX");
        m.insert("v", "IEX");
        m.insert("w", "CBSX");
        m.insert("x", "PSX");
        m.insert("y", "BYX");
        m.insert("z", "BZX");
        m
    };
    pub static ref US_TRADE_COND_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(" ", "US_REGULAR_SALE");
        m.insert("B", "US_BUNCHED_TRADE");
        m.insert("C", "US_CASH_TRADE");
        m.insert("F", "US_INTERMARKET_SWEEP");
        m.insert("G", "US_BUNCHED_SOLD_TRADE");
        m.insert("H", "US_PRICE_VARIATION_TRADE");
        m.insert("I", "US_ODD_LOT_TRADE");
        m.insert("K", "US_RULE_127_OR_155_TRADE");
        m.insert("L", "US_SOLD_LAST");
        m.insert("M", "US_MARKET_CENTER_CLOSE_PRICE");
        m.insert("N", "US_NEXT_DAY_TRADE");
        m.insert("O", "US_MARKET_CENTER_OPENING_TRADE");
        m.insert("P", "US_PRIOR_REFERENCE_PRICE");
        m.insert("Q", "US_MARKET_CENTER_OPEN_PRICE");
        m.insert("R", "US_SELLER");
        m.insert("T", "US_FORM_T");
        m.insert("U", "US_EXTENDED_TRADING_HOURS");
        m.insert("V", "US_CONTINGENT_TRADE");
        m.insert("W", "US_AVERAGE_PRICE_TRADE");
        m.insert("X", "US_CROSS_TRADE");
        m.insert("Z", "US_SOLD_OUT_OF_SEQUENCE");
        m.insert("0", "US_ODD_LOST_CROSS_TRADE");
        m.insert("4", "US_DERIVATIVELY_PRICED");
        m.insert("5", "US_MARKET_CENTER_RE_OPENING_TRADE");
        m.insert("6", "US_MARKET_CENTER_CLOSING_TRADE");
        m.insert("7", "US_QUALIFIED_CONTINGENT_TRADE");
        m.insert("9", "US_CONSOLIDATED_LAST_PRICE_PER_LISTING_PACKET");
        m
    };
    pub static ref HK_TRADE_COND_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(" ", "HK_AUTOMATCH_NORMAL");
        m.insert("D", "HK_ODD_LOT_TRADE");
        m.insert("U", "HK_AUCTION_TRADE");
        m.insert("*", "HK_OVERSEAS_TRADE");
        m.insert("P", "HK_LATE_TRADE_OFF_EXCHG");
        m.insert("M", "HK_NON_DIRECT_OFF_EXCHG_TRADE");
        m.insert("X", "HK_DIRECT_OFF_EXCHG_TRADE");
        m.insert("Y", "HK_AUTOMATIC_INTERNALIZED");
        m
    };
}
