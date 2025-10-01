// src/constants.rs

/// HTTP Methods
pub const GET: &str = "GET";
pub const POST: &str = "POST";

/// Common Parameters
pub const P_TIGER_ID: &str = "tiger_id";
pub const P_METHOD: &str = "method";
pub const P_CHARSET: &str = "charset";
pub const P_SIGN_TYPE: &str = "sign_type";
pub const P_SIGN: &str = "sign";
pub const P_LANG: &str = "lang";
pub const P_TIMESTAMP: &str = "timestamp";
pub const P_VERSION: &str = "version";
pub const P_ITEMS: &str = "items";
pub const P_DATA: &str = "data";
pub const P_CODE: &str = "code";
pub const P_NOTIFY_URL: &str = "notify_url";
pub const P_DEVICE_ID: &str = "device_id";
pub const P_SDK_VERSION_PREFIX: &str = "openapi-cpp-sdk-";
pub const P_USER_AGENT: &str = "User-Agent";
pub const P_BIZ_CONTENT: &str = "biz_content";
pub const P_ACCOUNT: &str = "account";
pub const P_SECRET_KEY: &str = "secret_key";
pub const P_MARKET: &str = "market";
pub const P_SYMBOLS: &str = "symbols";
pub const P_SYMBOL: &str = "symbol";
pub const P_INCLUDE_OTC: &str = "include_otc";
pub const P_CONTRACT_CODES: &str = "contract_codes";
pub const P_CONTRACT_CODE: &str = "contract_code";
pub const P_PERIOD: &str = "period";
pub const P_BEGIN_TIME: &str = "begin_time";
pub const P_START_TIME: &str = "start_time";
pub const P_END_TIME: &str = "end_time";
pub const P_START_DATE: &str = "start_date";
pub const P_BEGIN_DATE: &str = "begin_date";
pub const P_END_DATE: &str = "end_date";
pub const P_BEGIN_INDEX: &str = "begin_index";
pub const P_END_INDEX: &str = "end_index";
pub const P_TRADE_SESSION: &str = "trade_session";
pub const P_EXPIRY: &str = "expiry";
pub const P_STRIKE: &str = "strike";
pub const P_RIGHT: &str = "right";
pub const P_DATE: &str = "date";
pub const P_TRADING_DATE: &str = "trading_date";
pub const P_LIMIT: &str = "limit";
pub const P_PAGE_TOKEN: &str = "page_token";
pub const P_SEC_TYPE: &str = "sec_type";
pub const P_SEG_TYPE: &str = "seg_type";
pub const P_CURRENCY: &str = "currency";
pub const P_EXCHANGE: &str = "exchange";
pub const P_TYPE: &str = "type";
pub const P_BEGIN: &str = "begin";
pub const P_END: &str = "end";
pub const P_TICK_SIZE: &str = "tick_size";
pub const P_EXCHANGE_CODE: &str = "exchange_code";
pub const P_PAGE: &str = "page";
pub const P_PAGE_SIZE: &str = "page_size";
pub const P_SORT_DIR: &str = "sort_dir";
pub const P_SORT_FIELD_NAME: &str = "sort_field_name";
pub const P_WITH_DETAILS: &str = "with_details";

pub const PROJECT_VERSION: &str = "v1.1.0";
pub const DEFAULT_CHAR_SET: &str = "UTF-8";
pub const DEFAULT_SIGN_TYPE: &str = "RSA";

/// API Version
pub const OPEN_API_SERVICE_VERSION: &str = "2.0";

/// Tiger Brokers Public Keys
pub const TIGER_PUBLIC_KEY: &str = "MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDNF3G8SoEcCZh2rshUbayDgLLrj6rKgzNMxDL2HS\
nKcB0+GPOsndqSv+a4IBu9+I3fyBp5hkyMMG2+AXugd9pMpy6VxJxlNjhX1MYbNTZJUT4nudki4uh+LM\
OkIBHOceGNXjgB+cXqmlUnjlqha/HgboeHSnSgpM3dKSJQlIOsDwIDAQAB";

pub const SANDBOX_TIGER_PUBLIC_KEY: &str = "MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCbm21i11hgAENGd3/f280PSe4g9YGkS3TEXBY\
MidihTvHHf+tJ0PYD0o3PruI0hl3qhEjHTAxb75T5YD3SGK4IBhHn/Rk6mhqlGgI+bBrBVYaXixm\
HfRo75RpUUuWACyeqQkZckgR0McxuW9xRMIa2cXZOoL1E4SL4lXKGhKoWbwIDAQAB";

/// Hosts
pub const TIGER_HOST: &str = "openapi.tigerfintech.com";
pub const SANDBOX_TIGER_HOST: &str = "openapi-sandbox.tigerfintech.com";
pub const US_TIGER_HOST: &str = "openapi.tradeup.com";

/// HTTP Interface Service URLs
pub const TIGER_SERVER_URL: &str = "https://openapi.tigerfintech.com/gateway";
pub const SANDBOX_TIGER_SERVER_URL: &str = "https://openapi-sandbox.tigerfintech.com/gateway";
pub const US_TIGER_SERVER_URL: &str = "https://openapi.tradeup.com/gateway";

/// Push Service
pub const TIGER_SOCKET_HOST: &str = TIGER_HOST;
pub const SANDBOX_TIGER_SOCKET_HOST: &str = SANDBOX_TIGER_HOST;
pub const US_TIGER_SOCKET_HOST: &str = US_TIGER_HOST;

pub const TIGER_SOCKET_PORT: &str = "9883";
pub const SANDBOX_TIGER_SOCKET_PORT: &str = "9885";
pub const US_TIGER_SOCKET_PORT: &str = "9983";

/// Default Files
pub const DEFAULT_TOKEN_FILE: &str = "tiger_openapi_token.properties";
pub const DEFAULT_PROPS_FILE: &str = "tiger_openapi_config.properties";
