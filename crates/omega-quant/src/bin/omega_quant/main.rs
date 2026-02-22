//! omega-quant CLI — standalone quantitative trading tool.
//!
//! Subcommands:
//! - `check`     — verify IB Gateway connectivity
//! - `scan`      — scan market for instruments by volume/activity
//! - `analyze`   — stream trading signals as JSONL
//! - `order`     — place an order (market or bracket) via IBKR
//! - `positions` — list open positions
//! - `pnl`       — get daily P&L for an account
//! - `close`     — close a position
//! - `orders`    — list open/pending orders
//! - `cancel`    — cancel an order or all orders

mod handlers;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "omega-quant",
    version,
    about = "Quantitative trading engine — Kalman filter, HMM regime detection, Kelly sizing, IBKR execution"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check IB Gateway connectivity.
    Check {
        /// TWS/Gateway port (paper: 4002, live: 4001).
        #[arg(long, default_value_t = 4002)]
        port: u16,
        /// TWS/Gateway host.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Scan market for instruments by volume/activity.
    Scan {
        /// Scanner code (e.g. MOST_ACTIVE, HOT_BY_VOLUME, TOP_PERC_GAIN).
        #[arg(long, default_value = "MOST_ACTIVE")]
        scan_code: String,
        /// Instrument type (e.g. STK, CRYPTO, CASH.IDEALPRO).
        #[arg(long, default_value = "STK")]
        instrument: String,
        /// Location code (e.g. STK.US.MAJOR, CRYPTO.PAXOS).
        #[arg(long, default_value = "STK.US.MAJOR")]
        location: String,
        /// Number of results to return.
        #[arg(long, default_value_t = 10)]
        count: i32,
        /// Minimum price filter.
        #[arg(long)]
        min_price: Option<f64>,
        /// Minimum volume filter.
        #[arg(long)]
        min_volume: Option<i32>,
        /// TWS/Gateway port.
        #[arg(long, default_value_t = 4002)]
        port: u16,
        /// TWS/Gateway host.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Stream trading signals as JSONL (one JSON object per line).
    Analyze {
        /// Symbol (e.g. AAPL, EUR/USD, BTC).
        symbol: String,
        /// Asset class: stock, forex/fx, crypto.
        #[arg(long, default_value = "stock")]
        asset_class: String,
        /// Portfolio value in USD.
        #[arg(long, default_value_t = 10_000.0)]
        portfolio: f64,
        /// TWS/Gateway port.
        #[arg(long, default_value_t = 4002)]
        port: u16,
        /// TWS/Gateway host.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Number of 5-second bars to process before stopping.
        #[arg(long, default_value_t = 30)]
        bars: u32,
    },
    /// Place an order via IBKR (market or bracket with SL/TP).
    Order {
        /// Symbol (e.g. AAPL, EUR/USD, BTC).
        symbol: String,
        /// Order side: buy or sell.
        side: String,
        /// Quantity.
        quantity: f64,
        /// Asset class: stock, forex/fx, crypto.
        #[arg(long, default_value = "stock")]
        asset_class: String,
        /// Stop loss percentage (e.g. 1.5 = 1.5% below/above entry).
        #[arg(long)]
        stop_loss: Option<f64>,
        /// Take profit percentage (e.g. 3.0 = 3% above/below entry).
        #[arg(long)]
        take_profit: Option<f64>,
        /// IBKR account ID (for P&L cutoff check).
        #[arg(long)]
        account: Option<String>,
        /// Portfolio value in USD (for P&L cutoff check).
        #[arg(long)]
        portfolio: Option<f64>,
        /// Maximum simultaneous positions allowed.
        #[arg(long, default_value_t = 3)]
        max_positions: usize,
        /// TWS/Gateway port.
        #[arg(long, default_value_t = 4002)]
        port: u16,
        /// TWS/Gateway host.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// List open positions from IBKR.
    Positions {
        /// TWS/Gateway port.
        #[arg(long, default_value_t = 4002)]
        port: u16,
        /// TWS/Gateway host.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Get daily P&L for an IBKR account.
    Pnl {
        /// IBKR account ID (e.g. DU1234567).
        account: String,
        /// TWS/Gateway port.
        #[arg(long, default_value_t = 4002)]
        port: u16,
        /// TWS/Gateway host.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Close an open position.
    Close {
        /// Symbol (e.g. AAPL, EUR/USD, BTC).
        symbol: String,
        /// Asset class: stock, forex/fx, crypto.
        #[arg(long, default_value = "stock")]
        asset_class: String,
        /// Quantity to close (omit to close entire position).
        #[arg(long)]
        quantity: Option<f64>,
        /// TWS/Gateway port.
        #[arg(long, default_value_t = 4002)]
        port: u16,
        /// TWS/Gateway host.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// List all open/pending orders.
    Orders {
        /// TWS/Gateway port.
        #[arg(long, default_value_t = 4002)]
        port: u16,
        /// TWS/Gateway host.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Cancel an order by ID, or cancel all open orders.
    Cancel {
        /// Order ID to cancel (omit to cancel ALL open orders).
        #[arg(long)]
        order_id: Option<i32>,
        /// TWS/Gateway port.
        #[arg(long, default_value_t = 4002)]
        port: u16,
        /// TWS/Gateway host.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { port, host } => handlers::handle_check(host, port).await,
        Commands::Scan {
            scan_code,
            instrument,
            location,
            count,
            min_price,
            min_volume,
            port,
            host,
        } => {
            handlers::handle_scan(
                &scan_code,
                &instrument,
                &location,
                count,
                min_price,
                min_volume,
                host,
                port,
            )
            .await
        }
        Commands::Analyze {
            symbol,
            asset_class,
            portfolio,
            port,
            host,
            bars,
        } => handlers::handle_analyze(&symbol, &asset_class, portfolio, host, port, bars).await,
        Commands::Order {
            symbol,
            side,
            quantity,
            asset_class,
            stop_loss,
            take_profit,
            account,
            portfolio,
            max_positions,
            port,
            host,
        } => {
            handlers::handle_order(
                &symbol,
                &side,
                quantity,
                &asset_class,
                stop_loss,
                take_profit,
                account.as_deref(),
                portfolio,
                max_positions,
                host,
                port,
            )
            .await
        }
        Commands::Positions { port, host } => handlers::handle_positions(host, port).await,
        Commands::Pnl {
            account,
            port,
            host,
        } => handlers::handle_pnl(&account, host, port).await,
        Commands::Close {
            symbol,
            asset_class,
            quantity,
            port,
            host,
        } => handlers::handle_close(&symbol, &asset_class, quantity, host, port).await,
        Commands::Orders { port, host } => handlers::handle_orders(host, port).await,
        Commands::Cancel {
            order_id,
            port,
            host,
        } => handlers::handle_cancel(order_id, host, port).await,
    }
}
