use regex::Regex;

#[derive(Debug)]
pub struct Contract {
    pub contract_type: String,
    pub symbol: String,
    pub expiry: Option<String>,
    pub strike: Option<String>,
    pub right: Option<String>,
    pub currency: String,
    pub multiplier: Option<i64>,
    pub local_symbol: Option<String>,
    pub exchange: Option<String>,
    pub contract_id: Option<i64>,
    pub contract_month: Option<String>,
}

pub struct ContractUtil;

impl ContractUtil {
    pub fn stock_contract(
        symbol: &str,
        currency: &str,
        local_symbol: Option<&str>,
        exchange: Option<&str>,
        contract_id: Option<i64>,
    ) -> Contract {
        Contract {
            contract_type: "STK".to_string(),
            symbol: symbol.to_string(),
            expiry: None,
            strike: None,
            right: None,
            currency: currency.to_string(),
            multiplier: None,
            local_symbol: local_symbol.map(|s| s.to_string()),
            exchange: exchange.map(|s| s.to_string()),
            contract_id,
            contract_month: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn option_contract(
        symbol: &str,
        expiry: &str,
        strike: &str,
        right: &str,
        currency: &str,
        multiplier: i64,
        local_symbol: Option<&str>,
        contract_id: Option<i64>,
    ) -> Contract {
        Contract {
            contract_type: "OPT".to_string(),
            symbol: symbol.to_string(),
            expiry: Some(expiry.to_string()),
            strike: Some(strike.to_string()),
            right: Some(right.to_string()),
            currency: currency.to_string(),
            multiplier: Some(multiplier),
            local_symbol: local_symbol.map(|s| s.to_string()),
            exchange: None,
            contract_id,
            contract_month: None,
        }
    }

    pub fn option_contract_from_identifier(
        identifier: &str,
        multiplier: i64,
        currency: &str,
    ) -> Contract {
        let (symbol, expiry, right, strike) = Self::extract_option_info(identifier);
        let expiry_clean = expiry.replace("-", "");
        Contract {
            contract_type: "OPT".to_string(),
            symbol,
            expiry: Some(expiry_clean),
            strike: Some(strike),
            right: Some(right),
            currency: currency.to_string(),
            multiplier: Some(multiplier),
            local_symbol: None,
            exchange: None,
            contract_id: None,
            contract_month: None,
        }
    }

    pub fn future_contract(
        symbol: &str,
        currency: &str,
        expiry: Option<&str>,
        exchange: Option<&str>,
        contract_month: Option<&str>,
        multiplier: Option<i64>,
        local_symbol: Option<&str>,
    ) -> Contract {
        Contract {
            contract_type: "FUT".to_string(),
            symbol: symbol.to_string(),
            expiry: expiry.map(|s| s.to_string()),
            strike: None,
            right: None,
            currency: currency.to_string(),
            multiplier,
            local_symbol: local_symbol.map(|s| s.to_string()),
            exchange: exchange.map(|s| s.to_string()),
            contract_id: None,
            contract_month: contract_month.map(|s| s.to_string()),
        }
    }

    /// 提取期权信息：返回 (symbol, expiry, right, strike)
    pub fn extract_option_info(identifier: &str) -> (String, String, String, String) {
        let re = Regex::new(r"(\w+(?:\.\w+)?)\s*(\d{6})([CP])(\d+)").unwrap();
        if let Some(caps) = re.captures(identifier) {
            let symbol = caps[1].to_string();
            let mut expiry = format!("20{}", &caps[2]);
            if expiry.len() == 8 {
                expiry = format!("{}-{}-{}", &expiry[0..4], &expiry[4..6], &expiry[6..8]);
            }
            let right = if &caps[3] == "C" { "CALL" } else { "PUT" }.to_string();
            let strike_val: f64 = caps[4].parse::<f64>().unwrap_or(0.0) / 1000.0;
            let strike = format!("{}", strike_val);
            return (symbol, expiry, right, strike);
        }
        (
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        )
    }
}
