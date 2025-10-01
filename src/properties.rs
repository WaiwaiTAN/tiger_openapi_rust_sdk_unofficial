// src/properties.rs

use std::collections::HashMap;
use std::io::{BufRead, Write};

#[derive(Debug, Default, Clone)]
pub struct Properties {
    data: HashMap<String, String>,
}

impl Properties {
    pub fn load<R: BufRead>(&mut self, mut reader: R) -> std::io::Result<()> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            let trimmed = line.trim();
            // 跳过空行与注释
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                continue;
            }
            // 解析 key=value 或 key: value
            if let Some((k, v)) = split_kv(trimmed) {
                self.data.insert(k.to_string(), v.to_string());
            }
        }
        Ok(())
    }

    pub fn store<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        for (k, v) in &self.data {
            writeln!(writer, "{k}={v}")?;
        }
        Ok(())
    }

    pub fn get_property(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    pub fn set_property(&mut self, key: &str, value: impl Into<String>) {
        self.data.insert(key.to_string(), value.into());
    }
}

fn split_kv(s: &str) -> Option<(&str, &str)> {
    // 优先用 '=' 分隔，其次用 ':'
    if let Some(idx) = s.find('=') {
        let (k, v) = s.split_at(idx);
        let v = &v[1..];
        return Some((k.trim(), v.trim()));
    }
    if let Some(idx) = s.find(':') {
        let (k, v) = s.split_at(idx);
        let v = &v[1..];
        return Some((k.trim(), v.trim()));
    }
    None
}
