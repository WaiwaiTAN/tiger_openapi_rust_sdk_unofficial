use super::models::Bar;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use std::{path::Path, sync::Mutex};
pub struct MarketCache {
    connection: Mutex<Connection>,
}
impl MarketCache {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let c = Connection::open(path)?;
        let s = Self {
            connection: Mutex::new(c),
        };
        s.initialize()?;
        Ok(s)
    }
    pub fn memory() -> rusqlite::Result<Self> {
        let s = Self {
            connection: Mutex::new(Connection::open_in_memory()?),
        };
        s.initialize()?;
        Ok(s)
    }
    fn initialize(&self) -> rusqlite::Result<()> {
        self.connection.lock().unwrap().execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS schema_version(version INTEGER NOT NULL); INSERT INTO schema_version SELECT 1 WHERE NOT EXISTS(SELECT 1 FROM schema_version); CREATE TABLE IF NOT EXISTS bars(provider TEXT NOT NULL,symbol TEXT NOT NULL,interval TEXT NOT NULL,adjustment TEXT NOT NULL,timestamp TEXT NOT NULL,open TEXT NOT NULL,high TEXT NOT NULL,low TEXT NOT NULL,close TEXT NOT NULL,volume INTEGER NOT NULL,amount TEXT,is_complete INTEGER,PRIMARY KEY(provider,symbol,interval,adjustment,timestamp)); CREATE TABLE IF NOT EXISTS coverage(provider TEXT NOT NULL,symbol TEXT NOT NULL,interval TEXT NOT NULL,adjustment TEXT NOT NULL,start TEXT NOT NULL,end TEXT NOT NULL,UNIQUE(provider,symbol,interval,adjustment,start,end));")
    }
    pub fn put(
        &self,
        provider: &str,
        symbol: &str,
        interval: &str,
        adjustment: Option<&str>,
        bars: &[Bar],
    ) -> rusqlite::Result<()> {
        let mut c = self.connection.lock().unwrap();
        let tx = c.transaction()?;
        {
            let mut q=tx.prepare("INSERT INTO bars VALUES(?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(provider,symbol,interval,adjustment,timestamp) DO UPDATE SET open=excluded.open,high=excluded.high,low=excluded.low,close=excluded.close,volume=excluded.volume,amount=excluded.amount,is_complete=excluded.is_complete")?;
            for b in bars {
                q.execute(params![
                    provider,
                    symbol,
                    interval,
                    adjustment.unwrap_or(""),
                    b.timestamp.to_rfc3339(),
                    b.open.to_string(),
                    b.high.to_string(),
                    b.low.to_string(),
                    b.close.to_string(),
                    b.volume,
                    b.amount.map(|v| v.to_string()),
                    b.is_complete
                ])?;
            }
        }
        tx.commit()
    }
    pub fn get(
        &self,
        provider: &str,
        symbol: &str,
        interval: &str,
        adjustment: Option<&str>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> rusqlite::Result<Vec<Bar>> {
        let c = self.connection.lock().unwrap();
        let mut q=c.prepare("SELECT timestamp,open,high,low,close,volume,amount,is_complete FROM bars WHERE provider=?1 AND symbol=?2 AND interval=?3 AND adjustment=?4 AND timestamp>=?5 AND timestamp<=?6 ORDER BY timestamp")?;
        q.query_map(
            params![
                provider,
                symbol,
                interval,
                adjustment.unwrap_or(""),
                start.to_rfc3339(),
                end.to_rfc3339()
            ],
            |r| {
                let timestamp: String = r.get(0)?;
                let parse_dec = |i| -> rusqlite::Result<Decimal> {
                    r.get::<_, String>(i)?.parse().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            i,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })
                };
                let amount: Option<String> = r.get(6)?;
                Ok(Bar {
                    timestamp: timestamp.parse().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?,
                    open: parse_dec(1)?,
                    high: parse_dec(2)?,
                    low: parse_dec(3)?,
                    close: parse_dec(4)?,
                    volume: r.get(5)?,
                    amount: amount.and_then(|v| v.parse().ok()),
                    is_complete: r.get(7)?,
                })
            },
        )?
        .collect()
    }
    pub fn mark_coverage(
        &self,
        provider: &str,
        symbol: &str,
        interval: &str,
        adjustment: Option<&str>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> rusqlite::Result<()> {
        self.connection.lock().unwrap().execute(
            "INSERT OR IGNORE INTO coverage VALUES(?,?,?,?,?,?)",
            params![
                provider,
                symbol,
                interval,
                adjustment.unwrap_or(""),
                start.to_rfc3339(),
                end.to_rfc3339()
            ],
        )?;
        Ok(())
    }
    pub fn is_covered(
        &self,
        provider: &str,
        symbol: &str,
        interval: &str,
        adjustment: Option<&str>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> rusqlite::Result<bool> {
        let c = self.connection.lock().unwrap();
        let count:i64=c.query_row("SELECT COUNT(*) FROM coverage WHERE provider=?1 AND symbol=?2 AND interval=?3 AND adjustment=?4 AND start<=?5 AND end>=?6",params![provider,symbol,interval,adjustment.unwrap_or(""),start.to_rfc3339(),end.to_rfc3339()],|r|r.get(0))?;
        Ok(count > 0)
    }
    pub fn missing_ranges(
        &self,
        provider: &str,
        symbol: &str,
        interval: &str,
        adjustment: Option<&str>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> rusqlite::Result<Vec<(DateTime<Utc>, DateTime<Utc>)>> {
        let c = self.connection.lock().unwrap();
        let mut q = c.prepare("SELECT start,end FROM coverage WHERE provider=?1 AND symbol=?2 AND interval=?3 AND adjustment=?4 AND end>=?5 AND start<=?6 ORDER BY start")?;
        let covered: Vec<(DateTime<Utc>, DateTime<Utc>)> = q
            .query_map(
                params![
                    provider,
                    symbol,
                    interval,
                    adjustment.unwrap_or(""),
                    start.to_rfc3339(),
                    end.to_rfc3339()
                ],
                |r| {
                    let a: String = r.get(0)?;
                    let b: String = r.get(1)?;
                    Ok((
                        a.parse().map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?,
                        b.parse().map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                1,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?,
                    ))
                },
            )?
            .collect::<rusqlite::Result<_>>()?;
        let mut cursor = start;
        let mut missing = Vec::new();
        for (a, b) in covered {
            if a > cursor {
                missing.push((cursor, a));
            }
            if b > cursor {
                cursor = b;
            }
            if cursor >= end {
                break;
            }
        }
        if cursor < end {
            missing.push((cursor, end));
        }
        Ok(missing)
    }
    pub fn ready(&self) -> bool {
        self.connection.lock().is_ok()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    fn bar() -> Bar {
        Bar {
            timestamp: "2026-01-01T00:00:00Z".parse().unwrap(),
            open: Decimal::ONE,
            high: Decimal::ONE,
            low: Decimal::ONE,
            close: Decimal::ONE,
            volume: 1,
            amount: None,
            is_complete: None,
        }
    }
    #[test]
    fn schema_and_idempotency() {
        let c = MarketCache::memory().unwrap();
        c.put("mock", "X", "1d", None, &[bar(), bar()]).unwrap();
        assert_eq!(
            c.get(
                "mock",
                "X",
                "1d",
                None,
                "2025-01-01T00:00:00Z".parse().unwrap(),
                "2027-01-01T00:00:00Z".parse().unwrap()
            )
            .unwrap()
            .len(),
            1
        );
    }
    #[test]
    fn partial_coverage_returns_only_missing_edges() {
        let c = MarketCache::memory().unwrap();
        let start = "2026-01-01T00:00:00Z".parse().unwrap();
        let middle_start = "2026-01-03T00:00:00Z".parse().unwrap();
        let middle_end = "2026-01-07T00:00:00Z".parse().unwrap();
        let end = "2026-01-10T00:00:00Z".parse().unwrap();
        c.mark_coverage("mock", "X", "1d", None, middle_start, middle_end)
            .unwrap();
        let missing = c
            .missing_ranges("mock", "X", "1d", None, start, end)
            .unwrap();
        assert_eq!(missing, vec![(start, middle_start), (middle_end, end)]);
    }
}
