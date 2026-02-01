-- Fund Portfolio Management Tables

-- Portfolio: represents a collection of fund entries (e.g., "天天基金", "蚂蚁财富")
CREATE TABLE fund_portfolios (
    id SERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    description TEXT,
    total_amount DECIMAL(18, 2) NOT NULL DEFAULT 0
);

-- Fund entries: individual fund records within a portfolio
CREATE TABLE fund_entries (
    id SERIAL PRIMARY KEY,
    portfolio_id INT NOT NULL REFERENCES fund_portfolios(id) ON DELETE CASCADE,
    -- Category hierarchy
    major_category VARCHAR(64) NOT NULL,           -- 大类资产 (股票, 债券, 大宗商品, 现金)
    minor_category VARCHAR(64),                     -- 小类 (标普500, 纳斯达克100, etc.)
    fund_type VARCHAR(64),                          -- 基金类别 (标普500, 3-7年国债, etc.)
    -- Fund details
    fund_name VARCHAR(256) NOT NULL,                -- 基金名称 (博时标普500ETF联接A(050025))
    target_ratio DECIMAL(8, 4) NOT NULL DEFAULT 0,  -- 计划比例 (0.01 = 1%)
    amount DECIMAL(18, 2) NOT NULL DEFAULT 0,       -- 金额
    sort_index INT NOT NULL DEFAULT 0
);

CREATE INDEX idx_fund_entries_portfolio ON fund_entries(portfolio_id);
CREATE INDEX idx_fund_entries_major_category ON fund_entries(major_category);
CREATE INDEX idx_fund_entries_sort_index ON fund_entries(sort_index);
