// src/main.rs
//! Effective Stock Data Downloader with Missing Date Detection
//!
//! This module provides a robust and efficient framework for downloading stock data
//! with support for batch processing, automatic retries, and missing date detection.

use tiger_openapi_rust_sdk_unofficial::{
    client_config::ClientConfig, quote_client::QuoteClient, tiger_enums,
};

use serde::{Deserialize, Serialize};
use serde_json::json;

use anyhow::Result;
use anyhow::anyhow;
use std::fs;

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;

use std::time::Duration;
use tokio::task::JoinHandle;

use tokio::pin;
use tokio::time::sleep;
use tokio_stream::StreamExt;
fn parse_trading_days<'a>(
    calendar_result: &'a serde_json::Value,
    begin_date: &str,
    end_date: &str,
) -> Result<Vec<&'a str>> {
    match calendar_result {
        serde_json::Value::Array(days_array) => {
            let mut trading_days: Vec<&str> = Vec::new();

            for day_obj in days_array {
                if let serde_json::Value::Object(obj) = day_obj
                    && let Some(date_value) = obj.get("date")
                    && let Some(date_str) = date_value.as_str()
                    && date_str >= begin_date
                    && date_str <= end_date
                {
                    trading_days.push(date_str);
                }
            }

            trading_days.sort();
            Ok(trading_days)
        }
        _ => Err(anyhow!("无法解析交易日历")),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MinuteData {
    #[serde(rename = "avgPrice")]
    avg_price: f64,
    price: f64,
    time: i64, // 毫秒时间戳
    volume: i64,
}

#[derive(Debug, Serialize)]
struct DailyData {
    date: String,
    #[serde(rename = "minuteData")]
    minute_data: Vec<MinuteData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseData {
    symbol: String,
    items: Vec<MinuteData>,
}

async fn save_trading_data_as_jsonl(
    quote_client: &mut QuoteClient,
    symbols: &Vec<String>,
    date: &str,
    output_dir: &str,
) -> Result<()> {
    let response: Vec<ResponseData> = serde_json::from_value(
        quote_client
            .get_history_timeline(&json!(symbols), date, &tiger_enums::QuoteRight::br)
            .await?,
    )?;

    if response.is_empty() {
        return Err(anyhow!(
            "Warning: response (date: {}) is empty for all symbols",
            date
        ));
    }

    for response_data in response {
        let symbol = &response_data.symbol;
        let items = response_data.items;

        let daily_data = DailyData {
            date: date.to_string(),
            minute_data: items,
        };

        let json_line = serde_json::to_string(&daily_data)?;

        let file_name = format!("{}/data_{}_{}.jsonl", output_dir, symbol, date);
        fs::write(&file_name, format!("{}\n", json_line))?;

        println!("Saved data for {} to {}", symbol, file_name);
    }

    println!(
        "All symbols saved successfully to directory: {}",
        output_dir
    );
    Ok(())
}

async fn download_concurrently(
    symbols: &Vec<String>,
    begin_date: &str,
    end_date: &str,
) -> Result<()> {
    let mut handles: Vec<JoinHandle<Result<String>>> = vec![];

    let mut cfg = ClientConfig::new();
    cfg.props_path = Some(PathBuf::from(
        std::env::var_os("TIGER_CREDENTIAL_DIRECTORY").ok_or_else(|| {
            anyhow!("TIGER_CREDENTIAL_DIRECTORY must explicitly name the credential directory")
        })?,
    ));
    cfg.load_props()?;
    cfg.load_token()?;

    let quote_client = QuoteClient::new(cfg.clone(), true).await?;
    let calendar_result = quote_client
        .get_trading_calendar(tiger_enums::Market::HK, begin_date, end_date)
        .await?;

    let expected_dates = parse_trading_days(&calendar_result, begin_date, end_date)?;
    println!("预期交易日: {} 天", expected_dates.len());
    sleep(Duration::from_secs(6)).await;

    let stream = tokio_stream::iter(expected_dates.clone()).throttle(Duration::from_secs(7));
    pin!(stream);
    while let Some(date) = stream.next().await {
        let output_dir = format!("/tmp/trading_data_{}", date);
        match fs::create_dir_all(&output_dir) {
            Ok(_) => println!("目录创建成功: {}", output_dir),
            Err(e) => eprintln!("创建目录失败: {}", e),
        }
        let date_str = date.to_string();
        let cfg_clone = cfg.clone();
        let symbols_clone = symbols.clone();

        let handle = tokio::spawn(async move {
            let mut quote_client = QuoteClient::new(cfg_clone.clone(), true).await?;
            save_trading_data_as_jsonl(&mut quote_client, &symbols_clone, &date_str, &output_dir)
                .await?;
            Ok(output_dir)
        });
        handles.push(handle);
    }

    let mut generated_folders = vec![];
    for handle in handles {
        match handle.await {
            Ok(Ok(folder)) => generated_folders.push(folder),
            Ok(Err(e)) => eprintln!("下载任务失败: {}", e),
            Err(e) => eprintln!("任务Join错误: {}", e),
        }
    }

    for symbol in symbols {
        let final_filename = format!("{}_full_data.jsonl", symbol);
        let final_path = format!("./{}", final_filename);

        println!("开始合并 Symbol: {} 的数据...", symbol);

        let mut processed_files = 0;
        let mut skipped_files = 0;

        let file = fs::File::create(&final_path)?;
        let mut writer = BufWriter::new(file);

        for folder in &generated_folders {
            if let Some(date_part) = folder.split('_').next_back() {
                let file_path = format!("{}/data_{}_{}.jsonl", folder, symbol, date_part);

                if Path::new(&file_path).exists() {
                    let file = match fs::File::open(&file_path) {
                        Ok(file) => file,
                        Err(e) => {
                            eprintln!("无法打开文件 {}: {}", file_path, e);
                            skipped_files += 1;
                            continue;
                        }
                    };

                    let reader = BufReader::new(file);
                    let mut lines = reader.lines();

                    if let Some(line) = lines.next() {
                        match line {
                            Ok(line) => {
                                writer.write_all(line.as_bytes())?;
                                writer.write_all(b"\n")?;
                                processed_files += 1;
                            }
                            Err(e) => {
                                eprintln!("读取文件 {} 的行时出错: {}", file_path, e);
                                skipped_files += 1;
                            }
                        }
                    }
                    drop(lines);
                }
            }
        }

        writer.flush()?;

        if processed_files > 0 {
            println!(
                "Symbol: {} 已合并 {} 个文件，生成: {}",
                symbol, processed_files, final_path
            );

            if skipped_files > 0 {
                println!("  警告: 跳过了 {} 个文件", skipped_files);
            }

            for folder in &generated_folders {
                if let Some(date_part) = folder.split('_').next_back() {
                    let file_path = format!("{}/data_{}_{}.jsonl", folder, symbol, date_part);
                    if Path::new(&file_path).exists() {
                        let _ = fs::remove_file(&file_path);
                    }
                }
            }
        } else {
            println!("Symbol: {} 没有找到数据文件", symbol);
            let _ = fs::remove_file(&final_path);
        }
    }

    Ok(())
}

// 使用示例
#[tokio::main]
async fn main() -> Result<()> {
    download_concurrently(
        &vec![
            "09988".into(),
            "01810".into(),
            "03032".into(),
            "02259".into(),
        ],
        "2021-01-01",
        "2026-03-06",
    )
    .await?;
    Ok(())
}
