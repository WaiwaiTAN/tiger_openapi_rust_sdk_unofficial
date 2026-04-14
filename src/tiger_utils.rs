use base64::{Engine as _, engine::general_purpose};
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::sign::{Signer, Verifier};
use rsa::signature;
use anyhow::{Result, anyhow};

/// 等价于 C++ 的 hmac_sha1，返回 HMAC-SHA1 的原始字节
pub fn hmac_sha1(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let pkey = PKey::hmac(key)?;
    let mut signer = Signer::new(MessageDigest::sha1(), &pkey)?;
    signer.update(data)?;
    let mac = signer.sign_to_vec()?;
    Ok(mac)
}

/// 从 PEM 读取 RSA 私钥或公钥（PEM 为 UTF-8 文本）
/// is_private=true 读取私钥；false 读取公钥
pub fn create_rsa(
    key_pem: &str,
    is_private: bool,
) -> Result<Rsa<openssl::pkey::Private>, ErrorStack> {
    if is_private {
        Rsa::private_key_from_pem(key_pem.as_bytes())
    } else {
        // 注意：C++ 使用 PEM_read_bio_RSA_PUBKEY（PKCS#1 SubjectPublicKeyInfo）
        // 对应 Rust 的从公钥 PEM 创建 PKey，再提取 RSA 不如直接使用 Verifier 的 PKey。
        // 这里为了贴近原代码，仍返回 Rsa<Private> 的类型需求可以变更。
        // 如果只用于验签，建议直接用 PKey::public_key_from_pem。
        // 为了接口一致性，这里返回错误，提示用另一个函数。
        Err(ErrorStack::get())
    }
}

/// 用 SHA1 对 context 进行 RSA PKCS#1 v1.5 签名，返回原始签名字节
pub fn sha1_sign(
    context: &str,
    private_key_pem: &str,
) -> Result<Vec<u8>> {
    let content_bytes = match "utf-8" {
        "utf-8" => context.as_bytes(),
        _ => return Err(anyhow!("Unsupported charset")),
    };
    // 读取私钥
    let rsa = Rsa::private_key_from_pem(private_key_pem.as_bytes())
        .map_err(|_| anyhow!("RSA creation failed, please check your private key"))?;
    let pkey = PKey::from_rsa(rsa)?;

    // 使用 Signer + MessageDigest::sha1 等价于 C++ 的 RSA_sign(NID_sha1, ...)
    let mut signer = Signer::new(MessageDigest::sha1(), &pkey)?;
    signer.update(content_bytes)?;
    let signature = signer.sign_to_vec()?;

    Ok(signature)
}

/// 使用 SHA1 与 RSA PKCS#1 v1.5 验签。sign 为 Base64 编码的签名
pub fn sha1_verify(
    context: &str,
    sign_base64: &str,
    public_key_pem: &str,
) -> Result<bool> {
    // 读取公钥（SubjectPublicKeyInfo）
    let filled = fill_public_key_marker(public_key_pem);
    let pkey = PKey::public_key_from_pem(filled.as_bytes())
        .map_err(|_| anyhow!("RSA creation failed, please check your public key"))?;

    // Base64 解码签名
    let signature = general_purpose::STANDARD
        .decode(sign_base64)
        .map_err(|_| anyhow!("Base64 decode signature failed"))?;

    let mut verifier = Verifier::new(MessageDigest::sha1(), &pkey)?;
    verifier
        .update(context.as_bytes())
        .map_err(|_| anyhow!("failed to verify"))?;

    let ok = verifier.verify(&signature).map_err(|_| anyhow!("failed to verify signature"))?;
    Ok(if ok { true } else { false })
}

/// 获取 Base64 编码的签名
pub fn get_sign(
    private_key_pem: &str,
    content: &str,
) -> Result<String> {
    let filled = fill_private_key_marker(private_key_pem);
    let signature = sha1_sign(content, &filled)?;
    Ok(general_purpose::STANDARD.encode(signature))
}

/// 自动补全 PEM 格式私钥（如果缺少头尾）
pub fn fill_private_key_marker(key: &str) -> String {
    if key.contains("BEGIN") {
        key.to_string()
    } else {
        format!(
            "-----BEGIN RSA PRIVATE KEY-----\n{}\n-----END RSA PRIVATE KEY-----",
            key
        )
    }
}

/// 自动补全 PEM 格式公钥（如果缺少头尾）
pub fn fill_public_key_marker(key: &str) -> String {
    if key.contains("BEGIN") {
        key.to_string()
    } else {
        format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
            key
        )
    }
}

// pub fn get_sign(private_key_pem: &str, content: &str) -> Result<String, Box<dyn std::error::Error>> {
//     let filled = fill_private_key_marker(private_key_pem);
//     let private_key = RsaPrivateKey::from_pkcs1_pem(&filled)?;

//     let digest = Sha1::digest(content.as_bytes());
//     let signature = private_key.sign(Pkcs1v15Sign::new::<Sha1>(), &digest)?;

//     Ok(general_purpose::STANDARD.encode(signature))
// }

/// 验证签名 (SHA1withRSA)
pub fn verify_sign(
    public_key_pem: &str,
    content: &str,
    encoded_signature: &str,
) -> Result<bool> {
   sha1_verify(content, encoded_signature, public_key_pem)
}


use chrono::Local;

pub fn get_timestamp() -> String {
    // 获取当前时间（本地时区）
    let now = Local::now();
    // 格式化为 "YYYY-MM-DD HH:MM:SS"
    let formatted = now.format("%Y-%m-%d %H:%M:%S").to_string();
    formatted
}

use chrono::{NaiveDate, NaiveDateTime};

pub fn date_string_to_timestamp(date_string: &str) -> i64 {
    // 解析日期字符串，格式为 "YYYY-MM-DD"
    let date = NaiveDate::parse_from_str(date_string, "%Y-%m-%d")
        .expect("日期解析失败");

    // 设置时间为当天的 00:00:00
    let datetime = NaiveDateTime::new(date, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());

    // 转换为时间戳（秒），再乘以 1000 得到毫秒
    datetime.and_utc().timestamp() * 1000
}
