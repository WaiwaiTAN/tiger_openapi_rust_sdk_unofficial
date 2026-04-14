// src/main.rs
//! Effective Stock Data Downloader with Missing Date Detection
//!
//! This module provides a robust and efficient framework for downloading stock data
//! with support for batch processing, automatic retries, and missing date detection.

mod client_config;
mod constants;
mod contract_utils;
mod models;
mod properties;
mod quote_client;
mod service_types;
mod tiger_client;
mod tiger_enums;
mod tiger_utils;
mod trade_client;

use client_config::ClientConfig;
use quote_client::QuoteClient;
use serde_json::{Value, json};
use std::path::PathBuf;

use chrono::{Datelike, Days, NaiveDate, Utc};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

/// Download configuration settings
/// Controls retry behavior, request intervals, batch sizes, and storage location
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// Maximum number of retry attempts for failed requests
    pub max_retries: u32,
    /// Delay in milliseconds between retry attempts
    pub retry_delay_ms: u64,
    /// Interval in milliseconds between API requests (rate limiting)
    pub request_interval_ms: u64,
    /// Number of days to download in each batch to manage memory and API limits
    pub batch_size: usize,
    /// Directory path for storing downloaded data
    pub save_dir: String,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 1000,
            request_interval_ms: 300, // 300ms interval to comply with Tiger API rate limits
            batch_size: 10,           // Download 10 days per batch
            save_dir: "stock_data".to_string(),
        }
    }
}

impl DownloadConfig {
    /// Validates the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.batch_size == 0 {
            return Err("batch_size must be greater than 0".to_string());
        }
        if self.max_retries == 0 {
            return Err("max_retries must be greater than 0".to_string());
        }
        if self.request_interval_ms == 0 {
            return Err("request_interval_ms must be greater than 0".to_string());
        }
        Ok(())
    }
}

/// Tracks download progress and statistics
#[derive(Debug)]
pub struct DownloadProgress {
    /// Total number of days to download
    pub total_days: usize,
    /// Number of successfully downloaded days
    pub downloaded: usize,
    /// Number of failed downloads
    pub failed: usize,
    /// Current stock code being processed
    pub current_stock: String,
    /// Path to log file for failures
    pub log_file: String,
    /// Timestamp when download started
    pub start_time: std::time::Instant,
}

impl DownloadProgress {
    /// Creates a new progress tracker
    fn new(stock_code: &str, total_days: usize, log_file: &str) -> Self {
        Self {
            total_days,
            downloaded: 0,
            failed: 0,
            current_stock: stock_code.to_string(),
            log_file: log_file.to_string(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Updates progress with result of downloading a single day
    fn update(&mut self, success: bool, date: &str) {
        if success {
            self.downloaded += 1;
        } else {
            self.failed += 1;
            self.log_failure(date);
        }

        let progress_percent = (self.downloaded as f32 / self.total_days as f32 * 100.0) as u32;
        let elapsed = self.start_time.elapsed().as_secs();

        println!(
            "[{}] 进度: {}/{} ({}%) | 失败: {} | 耗时: {}s",
            self.current_stock,
            self.downloaded,
            self.total_days,
            progress_percent,
            self.failed,
            elapsed
        );
    }

    /// Logs a failed download to the log file
    fn log_failure(&self, date: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .expect("无法打开日志文件");
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(file, "[{}] {}: 下载失败", timestamp, date).expect("写入日志失败");
    }

    /// Returns summary statistics
    fn summary(&self) -> String {
        let success_rate = (self.downloaded as f32 / self.total_days as f32 * 100.0) as u32;
        let elapsed = self.start_time.elapsed().as_secs();

        format!(
            "成功: {} 天 | 失败: {} 天 | 成功率: {}% | 总耗时: {}s",
            self.downloaded, self.failed, success_rate, elapsed
        )
    }
}

/// Result of a single day download
#[derive(Debug, Clone)]
pub struct DayDownloadResult {
    /// Date of the download
    pub date: String,
    /// Result containing data or error message
    pub result: Result<serde_json::Value, String>,
}

/// Result of missing date analysis
#[derive(Debug)]
pub struct MissingDatesAnalysis {
    /// List of expected trading dates
    pub expected_dates: Vec<String>,
    /// List of successfully downloaded dates
    pub downloaded_dates: Vec<String>,
    /// List of missing dates that should have been downloaded
    pub missing_dates: Vec<String>,
    /// Files that were found and analyzed
    pub found_files: Vec<String>,
}

impl MissingDatesAnalysis {
    /// Returns a summary of the analysis
    fn summary(&self) -> String {
        format!(
            "预期日期: {} | 已下载: {} | 缺失: {}",
            self.expected_dates.len(),
            self.downloaded_dates.len(),
            self.missing_dates.len()
        )
    }
}

/// Loads downloaded data and checks for any missing dates
///
/// This function:
/// 1. Loads all JSON files from the stock directory
/// 2. Extracts downloaded dates from the data
/// 3. Compares with expected trading dates
/// 4. Returns a detailed analysis of missing dates
pub async fn check_and_load_downloaded_data(
    quote_client: &mut QuoteClient,
    stock_code: &str,
    market: tiger_enums::Market,
    begin_date: &str,
    end_date: &str,
    config: &DownloadConfig,
) -> Result<MissingDatesAnalysis, Box<dyn Error>> {
    println!("\n===== 检查已下载数据 =====");
    println!("股票: {}, 期间: {} 到 {}", stock_code, begin_date, end_date);

    let stock_dir = format!("{}/{}", config.save_dir, stock_code);

    // Get expected trading dates from API
    println!("获取交易日历...");
    let calendar_result = quote_client
        .get_trading_calendar(market, begin_date, end_date)
        .await?;

    let expected_dates = parse_trading_days(&calendar_result, begin_date, end_date)?;
    println!("预期交易日: {} 天", expected_dates.len());

    // Load downloaded dates from local files
    let mut downloaded_dates = HashSet::new();
    let mut found_files = Vec::new();

    if Path::new(&stock_dir).exists() {
        println!("扫描已保存的文件...");

        if let Ok(entries) = fs::read_dir(&stock_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(filename) = path.file_name() {
                        let filename_str = filename.to_string_lossy();

                        // Look for JSON files
                        if filename_str.ends_with(".json") {
                            found_files.push(filename_str.to_string());

                            match fs::read_to_string(&path) {
                                Ok(content) => {
                                    // Try to parse as array of day data
                                    if let Ok(Value::Array(items)) =
                                        serde_json::from_str::<Value>(&content)
                                    {
                                        for item in items {
                                            if let Some(date) =
                                                item.get("date").and_then(|v| v.as_str())
                                            {
                                                downloaded_dates.insert(date.to_string());
                                            }
                                        }
                                    }
                                    // Also try to parse as single object with nested data
                                    else if let Ok(Value::Object(obj)) =
                                        serde_json::from_str::<Value>(&content)
                                    {
                                        if let Some(date) = obj.get("date").and_then(|v| v.as_str())
                                        {
                                            downloaded_dates.insert(date.to_string());
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("警告: 无法读取文件 {}: {}", filename_str, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("找到文件: {}", found_files.len());
        println!("已下载日期: {} 天", downloaded_dates.len());
    } else {
        println!("警告: 目录不存在: {}", stock_dir);
    }

    // Find missing dates
    let mut missing_dates = Vec::new();
    for date in &expected_dates {
        if !downloaded_dates.contains(date) {
            missing_dates.push(date.clone());
        }
    }

    let analysis = MissingDatesAnalysis {
        expected_dates: expected_dates.clone(),
        downloaded_dates: downloaded_dates.into_iter().collect(),
        missing_dates: missing_dates.clone(),
        found_files,
    };

    // Print results
    println!("\n===== 数据检查结果 =====");
    println!("{}", analysis.summary());

    if !analysis.missing_dates.is_empty() {
        println!("\n缺失的日期 (前20个):");
        let display_count = analysis.missing_dates.len().min(20);
        for date in analysis.missing_dates.iter().take(display_count) {
            println!("  - {}", date);
        }
        if analysis.missing_dates.len() > 20 {
            println!(
                "  ... 还有 {} 个缺失日期",
                analysis.missing_dates.len() - 20
            );
        }
    } else {
        println!("\n✓ 所有交易日数据都已下载！");
    }

    Ok(analysis)
}

/// Downloads missing dates that were not successfully downloaded before
///
/// This function:
/// 1. Takes the analysis result from check_and_load_downloaded_data
/// 2. Downloads only the missing dates with new attempt
/// 3. Saves data in the same monthly organization
/// 4. Merges with existing monthly files to maintain consistency
pub async fn download_missing_dates(
    quote_client: &mut QuoteClient,
    stock_code: &str,
    market: tiger_enums::Market,
    quote_right: tiger_enums::QuoteRight,
    analysis: &MissingDatesAnalysis,
    config: &DownloadConfig,
) -> Result<(), Box<dyn Error>> {
    if analysis.missing_dates.is_empty() {
        println!("\n✓ 没有缺失的日期，无需重新下载");
        return Ok(());
    }

    println!("\n===== 开始下载缺失数据 =====");
    println!("股票: {}", stock_code);
    println!("缺失日期数: {}", analysis.missing_dates.len());

    let stock_dir = format!("{}/{}", config.save_dir, stock_code);

    // Create directory if it doesn't exist
    if !Path::new(&stock_dir).exists() {
        fs::create_dir_all(&stock_dir)?;
    }

    // Setup progress tracking for missing dates
    let log_file = format!("{}/{}_missed_retry.log", config.save_dir, stock_code);
    let mut progress = DownloadProgress::new(stock_code, analysis.missing_dates.len(), &log_file);

    // Download missing dates in batches
    let missing_dates = analysis.missing_dates.clone();
    let batches: Vec<&[String]> = missing_dates.chunks(config.batch_size).collect();
    let total_batches = batches.len();

    for (batch_num, batch) in batches.into_iter().enumerate() {
        println!(
            "\n[批次 {}/{}] 补充下载 {} 到 {} 的缺失数据 ({} 天)",
            batch_num + 1,
            total_batches,
            batch.first().unwrap(),
            batch.last().unwrap(),
            batch.len()
        );

        let batch_results =
            download_batch(quote_client, stock_code, batch, &quote_right, config).await?;

        let results: Vec<Result<serde_json::Value, String>> =
            batch_results.iter().map(|d| d.result.clone()).collect();

        // Save with special batch naming for missed dates
        save_missed_batch_data(&stock_dir, batch, &results)?;

        for day_result in batch_results {
            progress.update(day_result.result.is_ok(), &day_result.date);
        }

        // Delay between batches
        if batch_num + 1 < total_batches {
            sleep(Duration::from_secs(1)).await;
        }
    }

    println!("\n✓ 缺失数据下载完成!");
    println!("数据统计: {}", progress.summary());

    // Merge monthly files to ensure consistency
    println!("\n合并所有月度文件...");
    merge_monthly_files(&stock_dir)?;

    Ok(())
}

/// Loads all downloaded data for a stock into memory
/// Useful for data analysis and post-processing
pub async fn load_all_stock_data(
    stock_code: &str,
    config: &DownloadConfig,
) -> Result<Vec<Value>, Box<dyn Error>> {
    let stock_dir = format!("{}/{}", config.save_dir, stock_code);
    let mut all_data = Vec::new();

    if Path::new(&stock_dir).exists() {
        if let Ok(entries) = fs::read_dir(&stock_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(filename) = path.file_name() {
                        let filename_str = filename.to_string_lossy();

                        if filename_str.ends_with(".json") && !filename_str.contains("failed") {
                            if let Ok(content) = fs::read_to_string(&path) {
                                match serde_json::from_str(&content) {
                                    Ok(Value::Array(items)) => {
                                        all_data.extend(items);
                                    }
                                    Ok(single_item) => {
                                        all_data.push(single_item);
                                    }
                                    Err(e) => {
                                        eprintln!("警告: 解析失败 {}: {}", filename_str, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort by date
        all_data.sort_by(|a, b| {
            let date_a = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
            let date_b = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
            date_a.cmp(date_b)
        });
    }

    Ok(all_data)
}

/// Main function to download stock historical data
///
/// This function orchestrates the entire download process:
/// 1. Creates directories for storing data
/// 2. Fetches trading calendar
/// 3. Downloads data in batches
/// 4. Merges monthly files
/// 5. Reports statistics and failures
pub async fn download_stock_data(
    quote_client: &mut QuoteClient,
    stock_code: &str,
    market: tiger_enums::Market,
    begin_date: &str,
    end_date: &str,
    quote_right: tiger_enums::QuoteRight,
    config: &DownloadConfig,
) -> Result<(), Box<dyn Error>> {
    // Validate configuration
    config.validate()?;

    println!(
        "开始下载 {} ({:?}) 数据: {} 到 {}",
        stock_code, market, begin_date, end_date
    );

    // 1. Create save directory
    let stock_dir = format!("{}/{}", config.save_dir, stock_code);
    if !Path::new(&stock_dir).exists() {
        fs::create_dir_all(&stock_dir)?;
        println!("创建目录: {}", stock_dir);
    }

    // 2. Get trading calendar
    println!("获取交易日历...");
    let calendar_result = quote_client
        .get_trading_calendar(market, begin_date, end_date)
        .await?;

    let trading_days = parse_trading_days(&calendar_result, begin_date, end_date)?;
    println!("共 {} 个交易日", trading_days.len());

    if trading_days.is_empty() {
        println!("没有找到交易日");
        return Ok(());
    }

    // 3. Download data in batches
    let log_file = format!("{}/{}_failed.log", config.save_dir, stock_code);
    let mut progress = DownloadProgress::new(stock_code, trading_days.len(), &log_file);
    let batches: Vec<&[String]> = trading_days.chunks(config.batch_size).collect();
    let total_batches = batches.len();

    for (batch_num, batch) in batches.into_iter().enumerate() {
        println!(
            "\n[批次 {}/{}] 下载 {} 到 {} 的数据 ({} 天)",
            batch_num + 1,
            total_batches,
            batch.first().unwrap(),
            batch.last().unwrap(),
            batch.len()
        );

        let batch_results =
            download_batch(quote_client, stock_code, batch, &quote_right, config).await?;

        let results: Vec<Result<serde_json::Value, String>> =
            batch_results.iter().map(|d| d.result.clone()).collect();
        save_batch_data(&stock_dir, batch, &results, batch_num)?;

        for day_result in batch_results {
            progress.update(day_result.result.is_ok(), &day_result.date);
        }

        // Delay between batches to prevent rate limiting
        if batch_num + 1 < total_batches {
            println!("批次完成，等待1秒继续下一批...");
            sleep(Duration::from_secs(1)).await;
        }
    }

    println!("\n✓ 下载完成!");
    println!("数据统计: {}", progress.summary());

    // 4. Merge small files (optional)
    if config.batch_size < trading_days.len() {
        merge_monthly_files(&stock_dir)?;
    }

    Ok(())
}

/// Downloads a batch of dates with retry and rate limiting
async fn download_batch(
    quote_client: &mut QuoteClient,
    stock_code: &str,
    dates: &[String],
    quote_right: &tiger_enums::QuoteRight,
    config: &DownloadConfig,
) -> Result<Vec<DayDownloadResult>, Box<dyn Error>> {
    let mut results = Vec::new();

    for (i, date) in dates.iter().enumerate() {
        // Request interval for rate limiting
        if i > 0 {
            sleep(Duration::from_millis(config.request_interval_ms)).await;
        }

        let result = download_with_retry(
            quote_client,
            stock_code,
            date,
            quote_right,
            config.max_retries,
            config.retry_delay_ms,
        )
        .await;

        results.push(DayDownloadResult {
            date: date.clone(),
            result,
        });
    }

    Ok(results)
}

/// Downloads data with automatic retry logic
/// Handles rate limiting and transient failures gracefully
async fn download_with_retry(
    quote_client: &mut QuoteClient,
    stock_code: &str,
    date: &str,
    quote_right: &tiger_enums::QuoteRight,
    max_retries: u32,
    retry_delay_ms: u64,
) -> Result<serde_json::Value, String> {
    let mut last_error = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            println!("  重试 {} [{}/{}]...", date, attempt, max_retries);
            sleep(Duration::from_millis(retry_delay_ms * attempt as u64)).await;
        }

        match quote_client
            .get_history_timeline(&json!([stock_code]), date, quote_right)
            .await
        {
            Ok(data) => {
                if is_valid_data(&data) {
                    return Ok(data);
                } else {
                    last_error = Some(format!("{}: 返回数据为空或无效", date));
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                last_error = Some(format!("{}: {}", date, &error_msg));

                // Handle rate limiting
                if error_msg.contains("rate limit") || error_msg.contains("429") {
                    let wait_time = retry_delay_ms * 5;
                    println!("  [频率限制] 等待 {}ms", wait_time);
                    sleep(Duration::from_millis(wait_time)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| format!("{}: 未知错误", date)))
}

/// Saves batch data to files organized by month
fn save_batch_data(
    stock_dir: &str,
    dates: &[String],
    results: &[Result<serde_json::Value, String>],
    batch_num: usize,
) -> Result<(), Box<dyn Error>> {
    // Group data by month
    let mut monthly_data: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

    for (date, result) in dates.iter().zip(results) {
        if let Ok(data) = result {
            // Extract year-month as filename key (format: 2026-02)
            let year_month = date[..7].to_string();
            let day_data = json!({
                "date": date,
                "data": data
            });

            monthly_data
                .entry(year_month)
                .or_insert_with(Vec::new)
                .push(day_data);
        }
    }

    // Save each month's data
    for (year_month, data) in monthly_data {
        let filename = format!("{}/{}_{}.json", stock_dir, batch_num, year_month);
        let json_data = serde_json::to_string_pretty(&data)?;
        fs::write(&filename, json_data)?;
        println!("  保存: {} ({} 天)", filename, data.len());
    }

    Ok(())
}

/// Saves missed batch data to files organized by month
/// Similar to save_batch_data but handles the case of retrying missed dates
fn save_missed_batch_data(
    stock_dir: &str,
    dates: &[String],
    results: &[Result<serde_json::Value, String>],
) -> Result<(), Box<dyn Error>> {
    // Group data by month
    let mut monthly_data: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

    for (date, result) in dates.iter().zip(results) {
        if let Ok(data) = result {
            // Extract year-month as filename key (format: 2026-02)
            let year_month = date[..7].to_string();
            let day_data = json!({
                "date": date,
                "data": data
            });

            monthly_data
                .entry(year_month)
                .or_insert_with(Vec::new)
                .push(day_data);
        }
    }

    // Save each month's data
    for (year_month, data) in monthly_data {
        let filename = format!("{}/{}_missed.json", stock_dir, year_month);
        let json_data = serde_json::to_string_pretty(&data)?;
        fs::write(&filename, json_data)?;
        println!("  保存: {} ({} 天)", filename, data.len());
    }

    Ok(())
}

/// Merges batch files into monthly consolidated files
/// This helps manage multiple small files and improve data organization
/// Handles both regular batch files (_0), (_1), etc. and missed batch files (_missed)
fn merge_monthly_files(stock_dir: &str) -> Result<(), Box<dyn Error>> {
    println!("\n合并月度文件...");

    let paths = fs::read_dir(stock_dir)?;
    let mut monthly_files: HashMap<String, Vec<String>> = HashMap::new();

    // Group files by year-month (includes both regular and missed files)
    for entry in paths {
        let entry = entry?;
        let filename = entry.file_name().to_string_lossy().to_string();

        if filename.ends_with(".json") && filename.contains('_') {
            // Match patterns like: 0_2026-01.json, 1_2026-01.json, 2026-01_missed.json
            let year_month = if filename.contains("_missed") {
                // Pattern: 2026-01_missed.json
                filename.trim_end_matches("_missed.json").to_string()
            } else {
                // Pattern: 0_2026-01.json, 1_2026-01.json
                let parts: Vec<&str> = filename.split('_').collect();
                if parts.len() >= 2 {
                    parts[1].trim_end_matches(".json").to_string()
                } else {
                    continue;
                }
            };

            monthly_files
                .entry(year_month)
                .or_insert_with(Vec::new)
                .push(entry.path().to_string_lossy().to_string());
        }
    }

    // Merge files for each month
    for (year_month, mut file_paths) in monthly_files {
        if file_paths.len() > 1 {
            // Sort file paths to ensure consistent ordering
            file_paths.sort();

            let mut all_data = Vec::new();

            for file_path in &file_paths {
                let content = fs::read_to_string(file_path)?;
                let data: Vec<serde_json::Value> = serde_json::from_str(&content)?;
                all_data.extend(data);
            }

            // Remove duplicates based on date (keep the latest version)
            let mut seen_dates = HashSet::new();
            let mut unique_data = Vec::new();

            // Sort by date in reverse to get the latest data first
            all_data.sort_by(|a, b| {
                let date_a = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
                let date_b = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
                date_b.cmp(date_a) // Reverse sort (newest first)
            });

            for item in all_data {
                if let Some(date) = item.get("date").and_then(|v| v.as_str()) {
                    if !seen_dates.contains(date) {
                        seen_dates.insert(date.to_string());
                        unique_data.push(item);
                    }
                }
            }

            // Sort by date in forward order (oldest first)
            unique_data.sort_by(|a, b| {
                let date_a = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
                let date_b = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
                date_a.cmp(date_b)
            });

            // Save merged file with _merged suffix
            let merged_filename = format!("{}/{}_merged.json", stock_dir, year_month);
            let json_data = serde_json::to_string_pretty(&unique_data)?;
            fs::write(&merged_filename, json_data)?;
            println!(
                "  合并: {} 个文件 -> {} (共 {} 天)",
                file_paths.len(),
                merged_filename,
                unique_data.len()
            );

            // Optional: delete temporary files to save space
            for file_path in file_paths {
                fs::remove_file(file_path)?;
            }
        }
    }

    Ok(())
}

/// Parses trading calendar API response to extract trading dates
/// Falls back to weekday generation if API response is invalid
fn parse_trading_days(
    calendar_result: &serde_json::Value,
    begin_date: &str,
    end_date: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    match calendar_result {
        serde_json::Value::Array(days_array) => {
            let mut trading_days: Vec<String> = Vec::new();

            for day_obj in days_array {
                if let serde_json::Value::Object(obj) = day_obj {
                    if let Some(date_value) = obj.get("date") {
                        if let Some(date_str) = date_value.as_str() {
                            if date_str >= begin_date && date_str <= end_date {
                                trading_days.push(date_str.to_string());
                            }
                        }
                    }
                }
            }

            trading_days.sort();
            Ok(trading_days)
        }
        _ => {
            println!("警告: 无法解析交易日历，使用工作日（周一至周五）作为备选");
            generate_weekdays(begin_date, end_date)
        }
    }
}

/// Generates weekdays between two dates (Mon-Fri)
/// Used as fallback when trading calendar API is unavailable
fn generate_weekdays(begin_date: &str, end_date: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let begin = NaiveDate::parse_from_str(begin_date, "%Y-%m-%d")?;
    let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")?;

    let mut current = begin;
    let mut weekdays = Vec::new();

    while current <= end {
        let weekday = current.weekday();
        if weekday != chrono::Weekday::Sat && weekday != chrono::Weekday::Sun {
            weekdays.push(current.format("%Y-%m-%d").to_string());
        }
        current = current.succ_opt().ok_or("日期溢出")?;
    }

    Ok(weekdays)
}

/// Validates whether API response contains valid data
fn is_valid_data(data: &serde_json::Value) -> bool {
    !data.is_null()
        && !(data.is_array() && data.as_array().map(|arr| arr.is_empty()).unwrap_or(false))
        && !(data.is_object() && data.as_object().map(|obj| obj.is_empty()).unwrap_or(false))
}

/// Example: Downloads Alibaba (09988.HK) historical data
/// Configured with conservative settings for stable downloads
pub async fn download_alibaba(quote_client: &mut QuoteClient) -> Result<(), Box<dyn Error>> {
    let config = DownloadConfig {
        request_interval_ms: 350,           // Slightly longer interval for safety
        batch_size: 5,                      // Small batches to reduce memory usage
        save_dir: "stock_data".to_string(), // Use relative path
        ..DownloadConfig::default()
    };

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let five_years_ago = Utc::now()
        .checked_sub_days(Days::new(5 * 365))
        .unwrap_or(Utc::now())
        .format("%Y-%m-%d")
        .to_string();

    download_stock_data(
        quote_client,
        "09988", // Alibaba Hong Kong stock code
        tiger_enums::Market::HK,
        &five_years_ago,
        &today,
        tiger_enums::QuoteRight::br, // Backward adjusted price
        &config,
    )
    .await?;

    Ok(())
}

/// Example: Downloads Xiaomi (01810.HK) historical data
/// Configured for faster download of recent data
pub async fn download_xiaomi(quote_client: &mut QuoteClient) -> Result<(), Box<dyn Error>> {
    let config = DownloadConfig {
        request_interval_ms: 300,
        batch_size: 20, // Larger batches for faster download
        save_dir: "stock_data".to_string(),
        ..DownloadConfig::default()
    };

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let five_months_ago = Utc::now()
        .checked_sub_days(Days::new(150))
        .unwrap_or(Utc::now())
        .format("%Y-%m-%d")
        .to_string();

    download_stock_data(
        quote_client,
        "01810", // Xiaomi Hong Kong stock code
        tiger_enums::Market::HK,
        &five_months_ago,
        &today,
        tiger_enums::QuoteRight::br,
        &config,
    )
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = ClientConfig::new();
    cfg.props_path = Some(PathBuf::from("properties_sandbox/"));
    cfg.load_props();
    cfg.load_token();

    println!("===== 股票数据下载器 v2.0 =====");
    println!("功能: 批量下载 | 自动重试 | 缺失检测");
    println!();

    // Initialize client
    let mut quote_client = QuoteClient::new(cfg.clone()).await;

    // let ali240906 = quote_client.get_history_timeline(
    //     &json!(["09988"]),
    //     "2024-09-06",
    //     &tiger_enums::QuoteRight::br,
    // ).await?;
    // println!("阿里巴巴 2024-09-06 数据: {:?}", ali240906);

    // === Example 1: Download Alibaba data ===
    // println!("\n===== 示例 1: 下载阿里巴巴(09988.HK)数据 =====");
    // download_alibaba(&mut quote_client).await?;

    // === Example 2: Check for missing dates ===
    println!("\n===== 示例 2: 检查已下载数据 =====");
    let config = DownloadConfig {
        request_interval_ms: 350,
        batch_size: 5,
        save_dir: "../stock_data".to_string(),
        ..DownloadConfig::default()
    };

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let one_month_ago = Utc::now()
        .checked_sub_days(Days::new(5*365))
        .unwrap_or(Utc::now())
        .format("%Y-%m-%d")
        .to_string();

    // Check Alibaba data
    let analysis = check_and_load_downloaded_data(
        &mut quote_client,
        "09988",
        tiger_enums::Market::HK,
        &one_month_ago,
        &today,
        &config,
    )
    .await?;

    // Load all data if needed
    if analysis.missing_dates.is_empty() {
        println!("\n加载所有已下载数据...");
        let all_data = load_all_stock_data("09988", &config).await?;
        println!("已加载 {} 条数据", all_data.len());

        // Show sample data
        if !all_data.is_empty() {
            println!("\n数据样本 (前3条):");
            for (i, item) in all_data.iter().take(3).enumerate() {
                println!("  [{}] {}", i + 1, item);
            }
        }
    } else {
        println!("\n有 {} 个缺失日期", analysis.missing_dates.len());

    // === Example 3: Download missing dates ===
        println!("\n===== 示例 3: 补充下载缺失数据 =====");
        download_missing_dates(
            &mut quote_client,
            "09988",
            tiger_enums::Market::HK,
            tiger_enums::QuoteRight::br,
            &analysis,
            &config,
        )
        .await?;

        println!("\n✓ 缺失数据已补充完成，所有文件已按月份组织！");

        // Load all data after supplementing
        let all_data = load_all_stock_data("09988", &config).await?;
        println!("已加载 {} 条数据", all_data.len());
    }

    // === Example 4: Download other stocks ===
    // Uncomment to download additional stocks
    // println!("\n===== 示例 4: 下载小米(01810.HK)数据 =====");
    // download_xiaomi(&mut quote_client).await?;

    println!("\n✓ 所有任务已完成");
    println!("\n数据组织结构:");
    println!("  stock_data/09988/           - 阿里巴巴数据");
    println!("    0_2026-01.json            - 批次文件");
    println!("    2026-01_missed.json       - 缺失补充文件 (自动生成)");
    println!("    2026-01_merged.json       - 月度合并文件 (自动生成)");
    println!("    09988_failed.log          - 失败日期日志");
    println!("    09988_missed_retry.log    - 缺失补充日志");
    println!("\n功能说明:");
    println!("  ✓ 自动分批下载 (可配置批量大小)");
    println!("  ✓ 断点续传支持 (自动重试失败)");
    println!("  ✓ 速率限制控制 (符合API限制)");
    println!("  ✓ 缺失日期检测 (对账交易日)");
    println!("  ✓ 缺失数据补充 (自动重新下载)");
    println!("  ✓ 自动按月合并 (优化存储,自动去重)");

    Ok(())
}
