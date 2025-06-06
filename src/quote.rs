// quote.rs
use crate::error::QuoteAppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool}; 
use std::path::Path;
use utoipa::ToSchema;
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct JsonQuote {
    pub id: i64,
    pub quote: String,
    pub author: String,
}


#[derive(Clone, Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Quote 
{
    pub id: i64,
    pub quote: String,
    pub author: String,
}

pub fn read_quotes_from_file<P: AsRef<Path>>(quotes_path: P) -> Result<Vec<JsonQuote>, QuoteAppError> {
    let f = std::fs::File::open(quotes_path.as_ref())?;
    let quotes: Vec<JsonQuote> = serde_json::from_reader(f)?;
    Ok(quotes)
}

impl JsonQuote 
{
    pub fn new(quote: &Quote) -> Self {
        Self 
        {
            id: quote.id,
            quote: quote.quote.clone(),
            author: quote.author.clone(),
        }
    }

    pub fn to_quote(&self) -> Quote {
        Quote {
            id: self.id,
            quote: self.quote.clone(),
            author: self.author.clone(),
        }
    }
}

pub async fn get_quote_by_id_from_db(db: &SqlitePool, quote_id: &str) -> Result<Quote, sqlx::Error> {
    let id: i64 = quote_id.parse().map_err(|_| sqlx::Error::Decode("Invalid ID format".into()))?;
    sqlx::query_as! (
        Quote,
        "SELECT id, quote,author FROM quotes WHERE id = ?",
        id
    )
    .fetch_one(db)
    .await
}

pub async fn get_random_quote_id_from_db(db: &SqlitePool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!("SELECT id FROM quotes ORDER BY RANDOM() LIMIT 1;")
    .fetch_one(db)
    .await
}
