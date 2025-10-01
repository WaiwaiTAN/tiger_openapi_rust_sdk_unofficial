use openssl::rsa::{Rsa};
use openssl::pkey::PKey;
use openssl::sign::{Signer, Verifier};
use openssl::hash::MessageDigest;
use base64::{engine::general_purpose, Engine as _};
use openssl::error::ErrorStack;
use rsa::signature;

/// 等价于 C++ 的 hmac_sha1，返回 HMAC-SHA1 的原始字节
pub fn hmac_sha1(key: &[u8], data: &[u8]) -> Result<Vec<u8>, ErrorStack> {
    let pkey = PKey::hmac(key)?;
    let mut signer = Signer::new(MessageDigest::sha1(), &pkey)?;
    signer.update(data)?;
    let mac = signer.sign_to_vec()?;
    Ok(mac)
}

/// 从 PEM 读取 RSA 私钥或公钥（PEM 为 UTF-8 文本）
/// is_private=true 读取私钥；false 读取公钥
pub fn create_rsa(key_pem: &str, is_private: bool) -> Result<Rsa<openssl::pkey::Private>, ErrorStack> {
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
pub fn sha1_sign(context: &str, private_key_pem: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let content_bytes = match "utf-8" {
        "utf-8" => context.as_bytes(),
        _ => return Err("Unsupported charset".into()),
    };
    // 读取私钥
    let rsa = Rsa::private_key_from_pem(private_key_pem.as_bytes())
        .map_err(|_| "RSA creation failed, please check your private key")?;
    let pkey = PKey::from_rsa(rsa)?;

    // 使用 Signer + MessageDigest::sha1 等价于 C++ 的 RSA_sign(NID_sha1, ...)
    let mut signer = Signer::new(MessageDigest::sha1(), &pkey)?;
    signer.update(content_bytes)?;
    let signature = signer.sign_to_vec()?;

    Ok(signature)
}

/// 使用 SHA1 与 RSA PKCS#1 v1.5 验签。sign 为 Base64 编码的签名（与 C++ 行为一致）
/// 返回 1 表示成功，0 表示失败（与 C++ 的 ret 语义对齐）
pub fn sha1_verify(context: &str, sign_base64: &str, public_key_pem: &str) -> Result<i32, Box<dyn std::error::Error>> {
    // 读取公钥（SubjectPublicKeyInfo）
    let pkey = PKey::public_key_from_pem(public_key_pem.as_bytes())
        .map_err(|_| "RSA creation failed, please check your public key")?;

    // Base64 解码签名
     let signature = general_purpose::STANDARD.decode(sign_base64)?;

    let mut verifier = Verifier::new(MessageDigest::sha1(), &pkey)?;
    verifier.update(context.as_bytes())?;

    let ok = verifier.verify(&signature)?;
    Ok(if ok { 1 } else { 0 })
}


/// 获取 Base64 编码的签名
pub fn get_sign(private_key_pem: &str, content: &str) -> Result<String, Box<dyn std::error::Error>> {
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

// /// 验证签名 (SHA1withRSA)
// pub fn verify_sign(
//     public_key_pem: &str,
//     content: &str,
//     encoded_signature: &str,
// ) -> Result<bool, Box<dyn std::error::Error>> {
//     let filled = fill_public_key_marker(public_key_pem);
//     let public_key = RsaPublicKey::from_public_key_pem(&filled)?;

//     let digest = Sha1::digest(content.as_bytes());
//     let signature = general_purpose::STANDARD.decode(encoded_signature)?;

//     let result = public_key.verify(Pkcs1v15Verify::new::<Sha1>(), &digest, &signature);
//     Ok(result.is_ok())
// }
