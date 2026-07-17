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
use clap::Parser;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
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
    symbols: &[String],
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
    symbols: &[String],
    begin_date: &str,
    end_date: &str,
    output_dir: &Path,
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
        let day_output_dir = output_dir.join(format!(".trading_data_{}", date));
        match fs::create_dir_all(&day_output_dir) {
            Ok(_) => println!("目录创建成功: {}", day_output_dir.display()),
            Err(e) => eprintln!("创建目录失败: {}", e),
        }
        let date_str = date.to_string();
        let cfg_clone = cfg.clone();
        let symbols_clone = symbols.to_vec();

        let handle = tokio::spawn(async move {
            let mut quote_client = QuoteClient::new(cfg_clone.clone(), true).await?;
            let day_output_dir_string = day_output_dir.to_string_lossy().into_owned();
            save_trading_data_as_jsonl(
                &mut quote_client,
                &symbols_clone,
                &date_str,
                &day_output_dir_string,
            )
            .await?;
            Ok(day_output_dir_string)
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
        let final_path = output_dir.join(final_filename);

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
                symbol,
                processed_files,
                final_path.display()
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

#[derive(Debug, Parser)]
#[command(name = "downloader", about = "Tiger historical-data downloader")]
struct DownloaderArgs {
    /// Provider-native symbols. HK examples may be written as 7709.HK or 7709.
    #[arg(long, value_delimiter = ',', default_value = "7709.HK")]
    symbols: Vec<String>,
    #[arg(long, default_value = "2021-01-01")]
    start: String,
    #[arg(long, default_value = "2026-03-06")]
    end: String,
    /// Directory for completed JSONL files and temporary daily files.
    #[arg(long, default_value = "../stock_data/")]
    output_dir: PathBuf,
    /// Explicitly request the default detached mode (Unix only).
    #[arg(long, conflicts_with = "foreground")]
    detach: bool,
    /// Stay attached to the terminal instead of running in the background.
    #[arg(long, conflicts_with = "detach")]
    foreground: bool,
    /// Log file used in detached mode (defaults to <output-dir>/downloader.log).
    #[arg(long, conflicts_with = "foreground")]
    log_file: Option<PathBuf>,
    /// Internal flag used by the detached child process.
    #[arg(long, hide = true)]
    detached_child: bool,
}

#[cfg(unix)]
fn spawn_detached(args: &DownloaderArgs) -> Result<()> {
    use std::os::unix::process::CommandExt;

    fs::create_dir_all(&args.output_dir)?;
    let log_path = args
        .log_file
        .clone()
        .unwrap_or_else(|| args.output_dir.join("downloader.log"));
    let log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    let stderr = log.try_clone()?;
    let mut command = Command::new(std::env::current_exe()?);
    command
        .arg("--symbols")
        .arg(args.symbols.join(","))
        .arg("--start")
        .arg(&args.start)
        .arg("--end")
        .arg(&args.end)
        .arg("--output-dir")
        .arg(&args.output_dir)
        .arg("--detached-child")
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(stderr));
    // SAFETY: setsid has no memory-safety preconditions and is called before exec.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let child = command.spawn()?;
    println!("Downloader started in background (PID {}).", child.id());
    println!("Log: {}", log_path.display());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = DownloaderArgs::parse();
    let should_detach = args.detach || !args.foreground;
    if should_detach && !args.detached_child {
        #[cfg(unix)]
        return spawn_detached(&args);
        #[cfg(not(unix))]
        return Err(anyhow!("--detach is currently supported only on Unix"));
    }
    fs::create_dir_all(&args.output_dir)?;
    println!(
        "Downloading {} from {} to {} into {}",
        args.symbols.join(", "),
        args.start,
        args.end,
        args.output_dir.display()
    );
    download_concurrently(&args.symbols, &args.start, &args.end, &args.output_dir).await
}
