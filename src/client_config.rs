// src/client_config.rs

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use openssl::string;
use serde::{Deserialize, Serialize};

use crate::constants;
use crate::properties::Properties;

use rand::Rng;

pub fn get_device_id() -> String {
    let mut rng = rand::rng();
    let mut mac_bytes = [0u8; 6];

    rng.fill(&mut mac_bytes);
    mac_bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<String>>()
        .join(":")
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientConfig {
    // 业务字段（非本文重点）
    pub tiger_id: String,
    #[serde(skip)]
    pub private_key: String,

    pub charset: String,
    #[serde(skip)]
    pub version: String,
    pub sign_type: String,
    pub device_id: String,

    #[serde(skip)]
    pub account: String,
    #[serde(skip)]
    pub license: String, // 用于 is_us 判断（TBUS）
    #[serde(skip)]
    pub token: String,

    #[serde(skip)]
    pub lang: String,

    // 环境与服务器
    #[serde(skip)]
    pub sandbox_debug: bool,
    #[serde(skip)]
    pub server_url: String,
    #[serde(skip)]
    pub server_public_key: String,
    #[serde(skip)]
    pub socket_url: String,
    #[serde(skip)]
    pub socket_port: String,

    #[serde(skip)]
    // 文件路径根目录（C++ 中 props_path 若为目录则拼接文件名）
    pub props_path: Option<PathBuf>,
}

impl ClientConfig {
    // 仅示例：初始化时用正式环境默认
    pub fn new() -> Self {
        Self {
            sandbox_debug: false,
            server_url: constants::TIGER_SERVER_URL.to_string(),
            server_public_key: constants::TIGER_PUBLIC_KEY.to_string(),
            socket_url: constants::TIGER_SOCKET_HOST.to_string(),
            socket_port: constants::TIGER_SOCKET_PORT.to_string(),
            charset: constants::DEFAULT_CHAR_SET.to_string(),
            sign_type: constants::DEFAULT_SIGN_TYPE.to_string(),
            device_id: get_device_id(),
            lang: "en_US".to_string(),
            ..Default::default()
        }
    }

    // C++: get_props_path(DEFAULT_PROPS_FILE)
    pub fn get_props_path(&self, filename: &str) -> Option<PathBuf> {
        let root = self.props_path.as_ref()?;
        if root.is_dir() {
            let mut p = root.clone();
            p.push(filename);
            Some(p)
        } else {
            None
        }
    }

    // C++: get_token_path() => get_props_path(DEFAULT_TOKEN_FILE)
    pub fn get_token_path(&self) -> Option<PathBuf> {
        self.get_props_path(constants::DEFAULT_TOKEN_FILE)
    }

    // C++: load_props()
    pub fn load_props(&mut self) {
        let Some(full_path) = self.get_props_path(constants::DEFAULT_PROPS_FILE) else {
            return;
        };

        // 打开文件并解析
        let file = match File::open(&full_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Failed to open properties file: {} ({e})",
                    full_path.display()
                );
                return;
            }
        };
        let mut props = Properties::default();
        if let Err(e) = props.load(BufReader::new(file)) {
            eprintln!("Failed to load properties: {} ({e})", full_path.display());
            return;
        }

        // 同步关键字段（仅在当前字段为空时覆盖，贴近 C++ 逻辑）
        if self.tiger_id.is_empty() {
            if let Some(v) = props.get_property("tiger_id") {
                self.tiger_id = v;
            }
        }
        if self.private_key.is_empty() {
            // C++ 使用 private_key_pk1
            if let Some(v) = props.get_property("private_key_pk1") {
                self.private_key = v;
            }
        }
        if self.account.is_empty() {
            if let Some(v) = props.get_property("account") {
                self.account = v;
            }
        }
        if self.license.is_empty() {
            if let Some(v) = props.get_property("license") {
                self.license = v;
            }
        }

        // 读取 env 并处理 SANDBOX
        if let Some(mut env) = props.get_property("env") {
            env.make_ascii_uppercase();
            if env == "SANDBOX" {
                self.sandbox_debug = true;
                self.server_url = constants::SANDBOX_TIGER_SERVER_URL.to_string();
                self.server_public_key = constants::SANDBOX_TIGER_PUBLIC_KEY.to_string();
                self.socket_url = constants::SANDBOX_TIGER_SOCKET_HOST.to_string();
                self.socket_port = constants::SANDBOX_TIGER_SOCKET_PORT.to_string();
            }
        }

        // 根据 license 判断 US 环境（与 C++ is_us() 一致）
        if self.is_us() {
            self.server_url = constants::US_TIGER_SERVER_URL.to_string();
            self.socket_url = constants::US_TIGER_SOCKET_HOST.to_string();
            self.socket_port = constants::US_TIGER_SOCKET_PORT.to_string();
            // 注：C++ 中 server_public_key 在 US 环境下未覆盖，保持默认或 sandbox 值
        }

        // println!(
        //     "Loaded properties successfully, tiger_id: {} account: {}",
        //     self.tiger_id, self.account
        // );
    }

    // C++: load_token()
    pub fn load_token(&mut self) {
        let Some(full_path) = self.get_token_path() else {
            return;
        };
        let file = match File::open(&full_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open token file: {} ({e})", full_path.display());
                return;
            }
        };
        let mut props = Properties::default();
        if let Err(e) = props.load(BufReader::new(file)) {
            eprintln!("Failed to load token file: {} ({e})", full_path.display());
            return;
        }
        if let Some(v) = props.get_property("token") {
            self.token = v;
            // println!("Loaded token successfully, token: {}", self.token);
        }
    }

    // C++: save_token(new_token)
    pub fn save_token(&mut self, new_token: &str) {
        let Some(full_path) = self.get_token_path() else {
            return;
        };

        // 确保父目录存在（如果 props_path 是目录一般已存在）
        if let Some(parent) = full_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let file = match File::create(&full_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Failed to open token file for writing: {} ({e})",
                    full_path.display()
                );
                return;
            }
        };
        let mut props = Properties::default();
        props.set_property("token", new_token);

        if let Err(e) = props.store(BufWriter::new(file)) {
            eprintln!("Failed to save token file: {} ({e})", full_path.display());
            return;
        }
        self.token = new_token.to_string();
        println!("Saved token successfully, token: {}", self.token);
    }

    // C++: is_us() => license == TBUS
    pub fn is_us(&self) -> bool {
        !self.license.is_empty() && self.license == "TBUS"
    }
}
