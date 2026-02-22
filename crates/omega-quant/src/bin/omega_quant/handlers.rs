//! CLI command handlers for omega-quant subcommands.

use omega_quant::execution::{ImmediatePlan, Side};
use omega_quant::executor::{
    cancel_all_orders, cancel_order_by_id, check_daily_pnl_cutoff, check_max_positions,
    close_position, get_daily_pnl, get_ibkr_price, get_open_orders, get_positions,
    place_bracket_order, CircuitBreaker, DailyLimits, Executor,
};
use omega_quant::market_data::{build_contract, AssetClass, IbkrConfig};

/// Print a JSON connectivity error and exit.
pub fn connectivity_error(host: &str, port: u16) -> ! {
    let err = serde_json::json!({
        "error": format!("IB Gateway not reachable at {host}:{port}"),
    });
    println!("{}", serde_json::to_string(&err).unwrap());
    std::process::exit(1);
}

/// Parse a side string into a `Side` enum.
pub fn parse_side(s: &str) -> anyhow::Result<Side> {
    match s.to_lowercase().as_str() {
        "buy" => Ok(Side::Buy),
        "sell" => Ok(Side::Sell),
        _ => anyhow::bail!("Invalid side '{s}'. Use 'buy' or 'sell'."),
    }
}

/// Handle the `check` subcommand.
pub async fn handle_check(host: String, port: u16) -> anyhow::Result<()> {
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
    Ok(())
}

/// Handle the `scan` subcommand.
#[allow(clippy::too_many_arguments)]
pub async fn handle_scan(
    scan_code: &str,
    instrument: &str,
    location: &str,
    count: i32,
    min_price: Option<f64>,
    min_volume: Option<i32>,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let config = IbkrConfig {
        host: host.clone(),
        port,
        client_id: 1,
    };
    if !omega_quant::market_data::check_connection(&config).await {
        connectivity_error(&host, port);
    }

    let results = omega_quant::market_data::run_scanner(
        &config, scan_code, instrument, location, count, min_price, min_volume,
    )
    .await?;
    println!("{}", serde_json::to_string(&results)?);
    Ok(())
}

/// Handle the `analyze` subcommand.
pub async fn handle_analyze(
    symbol: &str,
    asset_class: &str,
    portfolio: f64,
    host: String,
    port: u16,
    bars: u32,
) -> anyhow::Result<()> {
    let config = IbkrConfig {
        host: host.clone(),
        port,
        client_id: 1,
    };
    if !omega_quant::market_data::check_connection(&config).await {
        connectivity_error(&host, port);
    }

    let parsed_class: AssetClass = asset_class.parse()?;
    let mut engine = omega_quant::QuantEngine::new(symbol, portfolio);
    let mut rx = omega_quant::market_data::start_price_feed(symbol, &config, parsed_class);
    let mut count: u32 = 0;

    // Timeout for first bar — if no data within 15s, the market is likely closed.
    let first = tokio::time::timeout(std::time::Duration::from_secs(15), rx.recv()).await;
    match first {
        Ok(Ok(tick)) => {
            let signal = engine.process_price(tick.price);
            println!("{}", serde_json::to_string(&signal)?);
            count += 1;
        }
        _ => {
            let err = serde_json::json!({
                "error": format!("No data received for {symbol} ({parsed_class}) within 15s — market may be closed or data subscription missing"),
            });
            println!("{}", serde_json::to_string(&err)?);
            std::process::exit(1);
        }
    }

    while count < bars {
        match tokio::time::timeout(std::time::Duration::from_secs(10), rx.recv()).await {
            Ok(Ok(tick)) => {
                let signal = engine.process_price(tick.price);
                println!("{}", serde_json::to_string(&signal)?);
                count += 1;
            }
            _ => break,
        }
    }
    Ok(())
}

/// Handle the `order` subcommand.
#[allow(clippy::too_many_arguments)]
pub async fn handle_order(
    symbol: &str,
    side: &str,
    quantity: f64,
    asset_class: &str,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
    account: Option<&str>,
    portfolio: Option<f64>,
    max_positions: usize,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let config = IbkrConfig {
        host: host.clone(),
        port,
        client_id: 1,
    };
    if !omega_quant::market_data::check_connection(&config).await {
        connectivity_error(&host, port);
    }

    let order_side = parse_side(side)?;
    let parsed_class: AssetClass = asset_class.parse()?;
    let contract = build_contract(symbol, parsed_class)?;

    // Safety: check position count.
    if let Ok(positions) = get_positions(&config).await {
        check_max_positions(positions.len(), max_positions)?;
    }

    // Safety: check daily P&L cutoff.
    if let (Some(acct), Some(port_val)) = (account, portfolio) {
        if let Ok(pnl) = get_daily_pnl(&config, acct).await {
            check_daily_pnl_cutoff(pnl.daily_pnl, port_val, 5.0)?;
        }
    }

    if let (Some(sl_pct), Some(tp_pct)) = (stop_loss, take_profit) {
        // Bracket order: fetch entry price, calculate SL/TP levels.
        let entry_price = get_ibkr_price(&config, &contract).await?;
        let (sl_price, tp_price) = match order_side {
            Side::Buy => (
                entry_price * (1.0 - sl_pct / 100.0),
                entry_price * (1.0 + tp_pct / 100.0),
            ),
            Side::Sell => (
                entry_price * (1.0 + sl_pct / 100.0),
                entry_price * (1.0 - tp_pct / 100.0),
            ),
        };

        let state =
            place_bracket_order(&config, &contract, order_side, quantity, tp_price, sl_price)
                .await?;

        let result = serde_json::json!({
            "type": "bracket",
            "status": format!("{:?}", state.status),
            "entry_price": entry_price,
            "stop_loss_price": sl_price,
            "take_profit_price": tp_price,
            "filled_qty": state.total_filled_qty,
            "filled_usd": state.total_filled_usd,
            "order_ids": state.order_ids,
            "errors": state.errors,
        });
        println!("{}", serde_json::to_string(&result)?);
    } else {
        // Simple market order.
        let plan = omega_quant::execution::ExecutionPlan::Immediate(ImmediatePlan {
            symbol: symbol.to_string(),
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
            "type": "market",
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
    Ok(())
}

/// Handle the `positions` subcommand.
pub async fn handle_positions(host: String, port: u16) -> anyhow::Result<()> {
    let config = IbkrConfig {
        host: host.clone(),
        port,
        client_id: 1,
    };
    if !omega_quant::market_data::check_connection(&config).await {
        connectivity_error(&host, port);
    }

    let positions = get_positions(&config).await?;
    println!("{}", serde_json::to_string(&positions)?);
    Ok(())
}

/// Handle the `pnl` subcommand.
pub async fn handle_pnl(account: &str, host: String, port: u16) -> anyhow::Result<()> {
    let config = IbkrConfig {
        host: host.clone(),
        port,
        client_id: 1,
    };
    if !omega_quant::market_data::check_connection(&config).await {
        connectivity_error(&host, port);
    }

    let pnl = get_daily_pnl(&config, account).await?;
    println!("{}", serde_json::to_string(&pnl)?);
    Ok(())
}

/// Handle the `close` subcommand.
pub async fn handle_close(
    symbol: &str,
    asset_class: &str,
    quantity: Option<f64>,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let config = IbkrConfig {
        host: host.clone(),
        port,
        client_id: 1,
    };
    if !omega_quant::market_data::check_connection(&config).await {
        connectivity_error(&host, port);
    }

    let parsed_class: AssetClass = asset_class.parse()?;
    let contract = build_contract(symbol, parsed_class)?;

    // Determine side and quantity from current position.
    let positions = get_positions(&config).await?;
    let match_symbol = match parsed_class {
        AssetClass::Forex => symbol.split('/').next().unwrap_or(symbol).to_string(),
        _ => symbol.to_string(),
    };
    let pos = positions
        .iter()
        .find(|p| p.symbol == match_symbol)
        .ok_or_else(|| anyhow::anyhow!("No open position found for {symbol}"))?;

    let close_qty = quantity.unwrap_or(pos.quantity.abs());
    let close_side = if pos.quantity > 0.0 {
        Side::Sell
    } else {
        Side::Buy
    };

    let state = close_position(&config, &contract, close_qty, close_side).await?;

    let result = serde_json::json!({
        "status": format!("{:?}", state.status),
        "side": format!("{close_side}"),
        "closed_qty": state.total_filled_qty,
        "filled_usd": state.total_filled_usd,
        "errors": state.errors,
    });
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}

/// Handle the `orders` subcommand.
pub async fn handle_orders(host: String, port: u16) -> anyhow::Result<()> {
    let config = IbkrConfig {
        host: host.clone(),
        port,
        client_id: 1,
    };
    if !omega_quant::market_data::check_connection(&config).await {
        connectivity_error(&host, port);
    }

    let orders = get_open_orders(&config).await?;
    println!("{}", serde_json::to_string(&orders)?);
    Ok(())
}

/// Handle the `cancel` subcommand.
pub async fn handle_cancel(order_id: Option<i32>, host: String, port: u16) -> anyhow::Result<()> {
    let config = IbkrConfig {
        host: host.clone(),
        port,
        client_id: 1,
    };
    if !omega_quant::market_data::check_connection(&config).await {
        connectivity_error(&host, port);
    }

    if let Some(id) = order_id {
        let status = cancel_order_by_id(&config, id).await?;
        let result = serde_json::json!({
            "cancelled": id,
            "status": status,
        });
        println!("{}", serde_json::to_string(&result)?);
    } else {
        cancel_all_orders(&config).await?;
        let result = serde_json::json!({
            "cancelled": "all",
            "status": "global_cancel_sent",
        });
        println!("{}", serde_json::to_string(&result)?);
    }
    Ok(())
}
