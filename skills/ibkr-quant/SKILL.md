---
name: "ibkr-quant"
description: "Quantitative trading via IBKR — Kalman filter, HMM regime detection, Kelly sizing, order execution."
trigger: "quant|trading|signal|ibkr|market|regime|kelly|position|interactive brokers|stock|portfolio|analyze"
---

# IBKR Quant Engine

You have access to `omega-quant`, a standalone CLI tool for quantitative trading analysis and order execution via Interactive Brokers (IBKR).

## Commands

### 1. Check IB Gateway connectivity

```bash
omega-quant check --port 4002
```

Returns JSON: `{"connected": true, "host": "127.0.0.1", "port": 4002}`

**Always check connectivity before running analyze or order commands.**

- Port 4002 = paper trading (default, safe)
- Port 4001 = live trading (real money)

### 2. Analyze — stream trading signals

```bash
omega-quant analyze AAPL --portfolio 50000 --port 4002 --bars 10
```

Streams JSONL (one JSON signal per 5-second bar). Each signal contains:
- `regime`: Bull / Bear / Lateral (HMM-detected market state)
- `regime_probabilities`: probability distribution across regimes
- `filtered_price`: Kalman-filtered price (noise removed)
- `trend`: price trend direction/magnitude
- `merton_allocation`: optimal portfolio allocation [-0.5, 1.5]
- `kelly_fraction`: fractional Kelly bet size
- `kelly_position_usd`: recommended position in dollars
- `kelly_should_trade`: whether Kelly criterion recommends trading
- `direction`: Long / Short / Hold
- `action`: Long/Short/Hold/ReducePosition/Exit with urgency
- `execution`: TWAP / Immediate / DontTrade
- `confidence`: signal confidence score [0, 1]
- `reasoning`: human-readable summary

**Interpreting signals:**
- **Bull regime + Long direction + kelly_should_trade=true** → strong buy signal
- **Bear regime + Short direction** → consider reducing exposure
- **Lateral regime** → typically Hold, wait for regime change
- **confidence > 0.5** → higher conviction signal
- **merton_allocation > 0.1** → math says go long; < -0.1 → go short

### 3. Order — place a trade via IBKR

```bash
omega-quant order AAPL buy 100 --port 4002
```

Returns JSON with execution result:
```json
{"status": "Completed", "filled_qty": 100.0, "avg_price": 185.52, "filled_usd": 18552.0, "errors": [], "abort_reason": null}
```

## Safety rules

1. **Paper first**: Always use `--port 4002` (paper) unless the user explicitly says "live" or "real money"
2. **Check before trade**: Always run `omega-quant check` before placing orders
3. **Analyze before trade**: Run `omega-quant analyze` to get signals before recommending a trade
4. **Daily limits**: The executor enforces max 10 trades/day, $50k/day, 5-min cooldown
5. **Circuit breaker**: Auto-aborts if price deviates >2% during execution
6. **Not financial advice**: Always include a disclaimer that signals are advisory, not financial advice

## Prerequisites

IB Gateway or TWS must be running locally. If `omega-quant check` shows `"connected": false`:
- Docker: `docker run -d -p 4002:4002 ghcr.io/gnzsnz/ib-gateway:latest`
- Or launch IB Gateway / TWS manually

## Typical workflow

1. User asks about trading or a stock → check connectivity
2. Run analyze with the symbol → interpret the signals
3. Present findings: regime, direction, Kelly sizing, confidence
4. If user wants to trade → confirm side/quantity → execute order on paper
5. Report execution result
