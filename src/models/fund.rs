use crate::db_wrapper::get_postgres;
use serde::{Deserialize, Serialize};
use sqlx::Row;

// ============== Portfolio ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundPortfolio {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub total_amount: f64,
}

fn decimal_to_f64(row: &sqlx::postgres::PgRow, col: &str) -> f64 {
    use sqlx::types::BigDecimal;
    row.get::<BigDecimal, _>(col)
        .to_string()
        .parse()
        .unwrap_or(0.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPortfolio {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePortfolio {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
}

impl FundPortfolio {
    /// List all portfolios
    pub async fn list_all() -> Result<Vec<FundPortfolio>, String> {
        sqlx::query("SELECT id, name, description, total_amount FROM fund_portfolios ORDER BY id")
            .map(|row: sqlx::postgres::PgRow| FundPortfolio {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                total_amount: decimal_to_f64(&row, "total_amount"),
            })
            .fetch_all(get_postgres())
            .await
            .map_err(|e| format!("Failed to fetch portfolios: {}", e))
    }

    /// Get a single portfolio by ID
    pub async fn get_by_id(id: i32) -> Result<FundPortfolio, String> {
        sqlx::query("SELECT id, name, description, total_amount FROM fund_portfolios WHERE id = $1")
            .bind(id)
            .map(|row: sqlx::postgres::PgRow| FundPortfolio {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                total_amount: decimal_to_f64(&row, "total_amount"),
            })
            .fetch_one(get_postgres())
            .await
            .map_err(|e| format!("Portfolio not found: {}", e))
    }

    /// Create a new portfolio
    pub async fn create(data: NewPortfolio) -> Result<i32, String> {
        sqlx::query("INSERT INTO fund_portfolios (name, description) VALUES ($1, $2) RETURNING id")
            .bind(&data.name)
            .bind(&data.description)
            .map(|row: sqlx::postgres::PgRow| row.get::<i32, _>(0))
            .fetch_one(get_postgres())
            .await
            .map_err(|e| format!("Failed to create portfolio: {}", e))
    }

    /// Update portfolio
    pub async fn update(data: UpdatePortfolio) -> Result<(), String> {
        let mut set_clauses: Vec<String> = Vec::new();
        let mut param_index = 1;

        if data.name.is_some() {
            set_clauses.push(format!("name = ${}", param_index));
            param_index += 1;
        }
        if data.description.is_some() {
            set_clauses.push(format!("description = ${}", param_index));
            param_index += 1;
        }

        // No fields to update: treat as a no-op success.
        if set_clauses.is_empty() {
            return Ok(());
        }

        let query = format!(
            "UPDATE fund_portfolios SET {} WHERE id = ${}",
            set_clauses.join(", "),
            param_index
        );

        let mut q = sqlx::query(&query);

        if let Some(ref name) = data.name {
            q = q.bind(name);
        }
        if let Some(ref desc) = data.description {
            q = q.bind(desc);
        }
        q = q.bind(data.id);

        q.execute(get_postgres())
            .await
            .map(|_| ())
            .map_err(|e| format!("Failed to update portfolio: {}", e))
    }

    /// Delete portfolio (will cascade delete all entries)
    pub async fn delete(id: i32) -> Result<(), String> {
        sqlx::query("DELETE FROM fund_portfolios WHERE id = $1")
            .bind(id)
            .execute(get_postgres())
            .await
            .map(|_| ())
            .map_err(|e| format!("Failed to delete portfolio: {}", e))
    }

    /// Update total amount (called after entries are modified)
    pub async fn recalculate_total(id: i32) -> Result<(), String> {
        sqlx::query(
            "UPDATE fund_portfolios SET total_amount = COALESCE(
                (SELECT SUM(amount) FROM fund_entries WHERE portfolio_id = $1), 0
             ) WHERE id = $1",
        )
        .bind(id)
        .execute(get_postgres())
        .await
        .map(|_| ())
        .map_err(|e| format!("Failed to recalculate total: {}", e))
    }
}

// ============== Fund Entry ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundEntry {
    pub id: i32,
    pub portfolio_id: i32,
    pub major_category: String,
    pub minor_category: Option<String>,
    pub fund_type: Option<String>,
    pub fund_name: String,
    pub target_ratio: f64,
    pub amount: f64,
    pub sort_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFundEntry {
    pub portfolio_id: i32,
    pub major_category: String,
    pub minor_category: Option<String>,
    pub fund_type: Option<String>,
    pub fund_name: String,
    pub target_ratio: f64,
    pub amount: f64,
    pub sort_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFundEntry {
    pub id: i32,
    pub major_category: Option<String>,
    pub minor_category: Option<String>,
    pub fund_type: Option<String>,
    pub fund_name: Option<String>,
    pub target_ratio: Option<f64>,
    pub amount: Option<f64>,
    pub sort_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateEntry {
    pub id: i32,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateOrderEntry {
    pub id: i32,
    pub sort_index: i32,
}

impl FundEntry {
    /// List all entries for a portfolio
    pub async fn list_by_portfolio(portfolio_id: i32) -> Result<Vec<FundEntry>, String> {
        sqlx::query(
            "SELECT id, portfolio_id, major_category, minor_category, fund_type, 
                fund_name, target_ratio, amount, sort_index
             FROM fund_entries 
             WHERE portfolio_id = $1 
             ORDER BY sort_index ASC, major_category, minor_category, fund_name",
        )
        .bind(portfolio_id)
        .map(|row: sqlx::postgres::PgRow| FundEntry {
            id: row.get("id"),
            portfolio_id: row.get("portfolio_id"),
            major_category: row.get("major_category"),
            minor_category: row.get("minor_category"),
            fund_type: row.get("fund_type"),
            fund_name: row.get("fund_name"),
            target_ratio: decimal_to_f64(&row, "target_ratio"),
            amount: decimal_to_f64(&row, "amount"),
            sort_index: row.get("sort_index"),
        })
        .fetch_all(get_postgres())
        .await
        .map_err(|e| format!("Failed to fetch entries: {}", e))
    }

    /// Create a new entry
    pub async fn create(data: NewFundEntry) -> Result<i32, String> {
        let id = sqlx::query(
            "INSERT INTO fund_entries 
             (portfolio_id, major_category, minor_category, fund_type, fund_name, target_ratio, amount, sort_index)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(data.portfolio_id)
        .bind(&data.major_category)
        .bind(&data.minor_category)
        .bind(&data.fund_type)
        .bind(&data.fund_name)
        .bind(data.target_ratio)
        .bind(data.amount)
        .bind(data.sort_index.unwrap_or(0))
        .map(|row: sqlx::postgres::PgRow| row.get::<i32, _>(0))
        .fetch_one(get_postgres())
        .await
        .map_err(|e| format!("Failed to create entry: {}", e))?;

        // Recalculate portfolio total
        FundPortfolio::recalculate_total(data.portfolio_id).await?;

        Ok(id)
    }

    /// Update an entry
    pub async fn update(data: UpdateFundEntry) -> Result<i32, String> {
        // First get the portfolio_id
        let portfolio_id: i32 = sqlx::query("SELECT portfolio_id FROM fund_entries WHERE id = $1")
            .bind(data.id)
            .map(|row: sqlx::postgres::PgRow| row.get(0))
            .fetch_one(get_postgres())
            .await
            .map_err(|e| format!("Entry not found: {}", e))?;

        // Use a simpler approach with direct SQL
        sqlx::query(
            "UPDATE fund_entries SET 
             major_category = COALESCE($1, major_category),
             minor_category = COALESCE($2, minor_category),
             fund_type = COALESCE($3, fund_type),
             fund_name = COALESCE($4, fund_name),
             target_ratio = COALESCE($5, target_ratio),
             amount = COALESCE($6, amount),
             sort_index = COALESCE($7, sort_index)
             WHERE id = $8",
        )
        .bind(&data.major_category)
        .bind(&data.minor_category)
        .bind(&data.fund_type)
        .bind(&data.fund_name)
        .bind(data.target_ratio)
        .bind(data.amount)
        .bind(data.sort_index)
        .bind(data.id)
        .execute(get_postgres())
        .await
        .map_err(|e| format!("Failed to update entry: {}", e))?;

        // Recalculate portfolio total
        FundPortfolio::recalculate_total(portfolio_id).await?;

        Ok(portfolio_id)
    }

    /// Delete an entry
    pub async fn delete(id: i32) -> Result<i32, String> {
        // First get the portfolio_id
        let portfolio_id: i32 = sqlx::query("SELECT portfolio_id FROM fund_entries WHERE id = $1")
            .bind(id)
            .map(|row: sqlx::postgres::PgRow| row.get(0))
            .fetch_one(get_postgres())
            .await
            .map_err(|e| format!("Entry not found: {}", e))?;

        sqlx::query("DELETE FROM fund_entries WHERE id = $1")
            .bind(id)
            .execute(get_postgres())
            .await
            .map_err(|e| format!("Failed to delete entry: {}", e))?;

        // Recalculate portfolio total
        FundPortfolio::recalculate_total(portfolio_id).await?;

        Ok(portfolio_id)
    }

    /// Batch update amounts (for quick amount editing)
    pub async fn batch_update_amounts(
        portfolio_id: i32,
        updates: Vec<BatchUpdateEntry>,
    ) -> Result<(), String> {
        for update in updates {
            sqlx::query("UPDATE fund_entries SET amount = $1 WHERE id = $2")
                .bind(update.amount)
                .bind(update.id)
                .execute(get_postgres())
                .await
                .map_err(|e| format!("Failed to update entry {}: {}", update.id, e))?;
        }

        // Recalculate portfolio total
        FundPortfolio::recalculate_total(portfolio_id).await?;

        Ok(())
    }

    /// Batch update sort_index for entries
    pub async fn batch_update_order(
        portfolio_id: i32,
        updates: Vec<BatchUpdateOrderEntry>,
    ) -> Result<(), String> {
        for update in updates {
            sqlx::query(
                "UPDATE fund_entries SET sort_index = $1 WHERE id = $2 AND portfolio_id = $3",
            )
            .bind(update.sort_index)
            .bind(update.id)
            .bind(portfolio_id)
            .execute(get_postgres())
            .await
            .map_err(|e| format!("Failed to update order for entry {}: {}", update.id, e))?;
        }

        Ok(())
    }
}

// ============== Simplified Response Structs (for API) ==============

/// 简化的 Portfolio 响应，只包含前端需要的字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSimple {
    pub id: i32,
    pub name: String,
    pub total_amount: f64,
}

impl From<FundPortfolio> for PortfolioSimple {
    fn from(p: FundPortfolio) -> Self {
        Self {
            id: p.id,
            name: p.name,
            total_amount: p.total_amount,
        }
    }
}

/// 简化的 Entry 响应，只包含前端需要的字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrySimple {
    pub id: i32,
    pub major_category: String,
    pub minor_category: Option<String>,
    pub fund_type: Option<String>,
    pub fund_name: String,
    pub target_ratio: f64,
    pub amount: f64,
}

impl From<FundEntry> for EntrySimple {
    fn from(e: FundEntry) -> Self {
        Self {
            id: e.id,
            major_category: e.major_category,
            minor_category: e.minor_category,
            fund_type: e.fund_type,
            fund_name: e.fund_name,
            target_ratio: e.target_ratio,
            amount: e.amount,
        }
    }
}

/// 简化的 Portfolio + Entries 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioWithEntriesSimple {
    pub portfolio: PortfolioSimple,
    pub entries: Vec<EntrySimple>,
}

impl PortfolioWithEntriesSimple {
    pub async fn get(portfolio_id: i32) -> Result<Self, String> {
        let portfolio = FundPortfolio::get_by_id(portfolio_id).await?;
        let entries = FundEntry::list_by_portfolio(portfolio_id).await?;
        Ok(Self {
            portfolio: portfolio.into(),
            entries: entries.into_iter().map(|e| e.into()).collect(),
        })
    }
}

// ============== Portfolio with Entries (full, for internal use) ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioWithEntries {
    pub portfolio: FundPortfolio,
    pub entries: Vec<FundEntry>,
}

impl PortfolioWithEntries {
    pub async fn get(portfolio_id: i32) -> Result<Self, String> {
        let portfolio = FundPortfolio::get_by_id(portfolio_id).await?;
        let entries = FundEntry::list_by_portfolio(portfolio_id).await?;
        Ok(Self { portfolio, entries })
    }
}
