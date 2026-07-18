use chrono::{DateTime, TimeZone, Utc};
use serde::Serialize;
use std::sync::Arc;
use tigeropen::push::{Callbacks, PushClient, SubjectType, connect, pb};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize)]
pub struct RealtimeQuote {
    pub symbol: String,
    pub received_at: DateTime<Utc>,
    pub source_timestamp: Option<DateTime<Utc>>,
    pub latest_price: Option<f64>,
    pub volume: Option<i64>,
    pub amount: Option<f64>,
    pub bid_price: Option<f64>,
    pub bid_size: Option<i64>,
    pub ask_price: Option<f64>,
    pub ask_size: Option<i64>,
    pub market_status: Option<String>,
}

impl From<pb::QuoteData> for RealtimeQuote {
    fn from(value: pb::QuoteData) -> Self {
        Self {
            symbol: value.symbol,
            received_at: Utc::now(),
            source_timestamp: i64::try_from(value.timestamp)
                .ok()
                .and_then(|ts| Utc.timestamp_millis_opt(ts).single()),
            latest_price: value.latest_price,
            volume: value.volume,
            amount: value.amount,
            bid_price: value.bid_price,
            bid_size: value.bid_size,
            ask_price: value.ask_price,
            ask_size: value.ask_size,
            market_status: value.market_status,
        }
    }
}

#[derive(Clone)]
pub struct RealtimeHub {
    client: Arc<PushClient>,
    sender: broadcast::Sender<RealtimeQuote>,
}

impl RealtimeHub {
    pub async fn connect(config: tigeropen::config::ClientConfig) -> anyhow::Result<Self> {
        let client = Arc::new(PushClient::new(config, None));
        let (sender, _) = broadcast::channel(4096);
        let quote_sender = sender.clone();
        let bbo_sender = sender.clone();
        client.set_callbacks(Callbacks {
            on_quote: Some(Arc::new(move |quote| {
                let _ = quote_sender.send(quote.into());
            })),
            on_quote_bbo: Some(Arc::new(move |quote| {
                let _ = bbo_sender.send(quote.into());
            })),
            on_error: Some(Arc::new(
                |message| tracing::error!(%message, "Tiger push error"),
            )),
            on_kickout: Some(Arc::new(
                |message| tracing::error!(%message, "Tiger push kicked out"),
            )),
            ..Default::default()
        });
        connect(&client)
            .await
            .map_err(|message| anyhow::anyhow!("Tiger push connection failed: {message}"))?;
        Ok(Self { client, sender })
    }

    pub fn subscribe(
        &self,
        symbols: &[String],
    ) -> anyhow::Result<broadcast::Receiver<RealtimeQuote>> {
        let joined = symbols.join(",");
        if !self
            .client
            .subscribe(&SubjectType::Quote, Some(&joined), None, None)
        {
            anyhow::bail!("Tiger push subscription could not be sent");
        }
        Ok(self.sender.subscribe())
    }
}
