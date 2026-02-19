//! omega-quant CLI — standalone quantitative trading tool.
//!
//! Subcommands:
//! - `check`   — verify IB Gateway connectivity
//! - `analyze` — stream trading signals as JSONL
//! - `order`   — place an order via IBKR

use clap::{Parser, Subcommand};
use omega_quant::execution::{ImmediatePlan, Side};
use omega_quant::executor::{CircuitBreaker, DailyLimits, Executor};
use omega_quant::market_data::IbkrConfig;

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
    /// Stream trading signals as JSONL (one JSON object per line).
    Analyze {
        /// Stock symbol (e.g. AAPL).
        symbol: String,
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
    /// Place an order via IBKR.
    Order {
        /// Stock symbol (e.g. AAPL).
        symbol: String,
        /// Order side: buy or sell.
        side: String,
        /// Quantity (number of shares).
        quantity: f64,
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
        Commands::Check { port, host } => {
            let config = IbkrConfig {
                host: host.clone(),
                port,
                client_id: 1,
            };
            let connected = omega_quant::market_data::check_connection(&config).await;
            let result = serde_json::json!({
                "connected": connected,
                "host": host,
                "port": port,
            });
            println!("{}", serde_json::to_string(&result)?);
        }
        Commands::Analyze {
            symbol,
            portfolio,
            port,
            host,
            bars,
        } => {
            let config = IbkrConfig {
                host,
                port,
                client_id: 1,
            };

            // Verify connectivity first.
            if !omega_quant::market_data::check_connection(&config).await {
                let err = serde_json::json!({
                    "error": format!("IB Gateway not reachable at {}:{}", config.host, config.port),
                });
                println!("{}", serde_json::to_string(&err)?);
                std::process::exit(1);
            }

            let mut engine = omega_quant::QuantEngine::new(&symbol, portfolio);
            let mut rx = omega_quant::market_data::start_price_feed(&symbol, &config);
            let mut count: u32 = 0;

            while let Ok(tick) = rx.recv().await {
                let signal = engine.process_price(tick.price);
                println!("{}", serde_json::to_string(&signal)?);
                count += 1;
                if count >= bars {
                    break;
                }
            }
        }
        Commands::Order {
            symbol,
            side,
            quantity,
            port,
            host,
        } => {
            let config = IbkrConfig {
                host,
                port,
                client_id: 1,
            };

            // Verify connectivity first.
            if !omega_quant::market_data::check_connection(&config).await {
                let err = serde_json::json!({
                    "error": format!("IB Gateway not reachable at {}:{}", config.host, config.port),
                });
                println!("{}", serde_json::to_string(&err)?);
                std::process::exit(1);
            }

            let order_side = match side.to_lowercase().as_str() {
                "buy" => Side::Buy,
                "sell" => Side::Sell,
                _ => {
                    anyhow::bail!("Invalid side '{}'. Use 'buy' or 'sell'.", side);
                }
            };

            let plan = omega_quant::execution::ExecutionPlan::Immediate(ImmediatePlan {
                symbol: symbol.clone(),
                side: order_side,
                quantity,
                estimated_price: 0.0,
                estimated_usd: 0.0,
            });

            let circuit_breaker = CircuitBreaker::default();
            let daily_limits = DailyLimits::new(10, 50_000.0, 5);
            let mut executor = Executor::new(config, circuit_breaker, daily_limits);
            let state = executor.execute(&plan).await;

            let result = serde_json::json!({
                "status": format!("{:?}", state.status),
                "filled_qty": state.total_filled_qty,
                "avg_price": if state.total_filled_qty > 0.0 {
                    state.total_filled_usd / state.total_filled_qty
                } else {
                    0.0
                },
                "filled_usd": state.total_filled_usd,
                "errors": state.errors,
                "abort_reason": state.abort_reason,
            });
            println!("{}", serde_json::to_string(&result)?);
        }
    }

    Ok(())
}
