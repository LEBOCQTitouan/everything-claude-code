---
name: clickhouse-io
description: ClickHouse database patterns, query optimization, analytics, and data engineering best practices for high-performance analytical workloads.
origin: ECC
---

# ClickHouse Analytics Patterns

## When to Activate

- Designing ClickHouse schemas, writing analytical queries
- Optimizing performance (partition pruning, projections, materialized views)
- Ingesting data (batch inserts, Kafka, CDC)

## Table Design

### MergeTree (Most Common)

```sql
CREATE TABLE markets_analytics (
    date Date, market_id String, market_name String,
    volume UInt64, trades UInt32, unique_traders UInt32,
    avg_trade_size Float64, created_at DateTime
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, market_id)
SETTINGS index_granularity = 8192;
```

### ReplacingMergeTree (Deduplication)

```sql
CREATE TABLE user_events (
    event_id String, user_id String, event_type String,
    timestamp DateTime, properties String
) ENGINE = ReplacingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (user_id, event_id, timestamp);
```

### AggregatingMergeTree (Pre-aggregation)

```sql
CREATE TABLE market_stats_hourly (
    hour DateTime, market_id String,
    total_volume AggregateFunction(sum, UInt64),
    total_trades AggregateFunction(count, UInt32),
    unique_users AggregateFunction(uniq, String)
) ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(hour)
ORDER BY (hour, market_id);

SELECT hour, market_id, sumMerge(total_volume) AS volume,
    countMerge(total_trades) AS trades, uniqMerge(unique_users) AS users
FROM market_stats_hourly
WHERE hour >= toStartOfHour(now() - INTERVAL 24 HOUR)
GROUP BY hour, market_id;
```

## Query Patterns

```sql
-- Filter on indexed columns first
SELECT * FROM markets_analytics
WHERE date >= '2025-01-01' AND market_id = 'market-123' AND volume > 1000
ORDER BY date DESC LIMIT 100;

-- Aggregations with quantiles
SELECT toStartOfDay(created_at) AS day, market_id,
    sum(volume) AS total_volume, uniq(trader_id) AS unique_traders,
    quantile(0.95)(trade_size) AS p95
FROM trades
WHERE created_at >= today() - INTERVAL 7 DAY
GROUP BY day, market_id;

-- Window functions
SELECT date, market_id, volume,
    sum(volume) OVER (PARTITION BY market_id ORDER BY date
        ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS cumulative_volume
FROM markets_analytics;
```

## Data Insertion

```typescript
// Batch insert (always prefer over individual inserts)
async function bulkInsertTrades(trades: Trade[]) {
  const values = trades.map(t => `('${t.id}','${t.market_id}','${t.user_id}',${t.amount},'${t.timestamp.toISOString()}')`).join(',')
  await clickhouse.query(`INSERT INTO trades (id, market_id, user_id, amount, timestamp) VALUES ${values}`).toPromise()
}
```

## Materialized Views

```sql
CREATE MATERIALIZED VIEW market_stats_hourly_mv TO market_stats_hourly AS
SELECT toStartOfHour(timestamp) AS hour, market_id,
    sumState(amount) AS total_volume, countState() AS total_trades,
    uniqState(user_id) AS unique_users
FROM trades GROUP BY hour, market_id;
```

## Analytics Queries

```sql
-- Daily active users
SELECT toDate(timestamp) AS date, uniq(user_id) AS dau
FROM events WHERE timestamp >= today() - INTERVAL 30 DAY GROUP BY date ORDER BY date;

-- Conversion funnel
SELECT countIf(step = 'viewed_market') AS viewed,
    countIf(step = 'clicked_trade') AS clicked,
    countIf(step = 'completed_trade') AS completed
FROM (SELECT user_id, event_type AS step FROM events WHERE event_date = today());
```

## Performance Monitoring

```sql
SELECT query_id, query, query_duration_ms, read_rows, memory_usage
FROM system.query_log
WHERE type = 'QueryFinish' AND query_duration_ms > 1000 AND event_time >= now() - INTERVAL 1 HOUR
ORDER BY query_duration_ms DESC LIMIT 10;
```

## Best Practices

- **Partitioning**: By time (month/day), avoid too many partitions
- **Ordering key**: Most-filtered columns first, high cardinality first
- **Data types**: Smallest appropriate type, LowCardinality for repeated strings, Enum for categories
- **Avoid**: `SELECT *`, `FINAL`, too many JOINs (denormalize instead), small frequent inserts (batch)
