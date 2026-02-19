# Quantitative Trading Engine

## Overview

The omega-quant crate provides real-time quantitative trading analysis and order execution via Interactive Brokers (IBKR). It is exposed as a standalone CLI binary (`omega-quant`) that the AI invokes through the `ibkr-quant` skill — no gateway wiring, no config.toml section needed.

## How It Works

The AI learns about omega-quant from the `ibkr-quant` skill (`skills/ibkr-quant/SKILL.md`). When a user asks about trading, stocks, or market analysis, the AI invokes the CLI tool via bash:

1. **Check connectivity**: `omega-quant check --port 4002`
2. **Analyze signals**: `omega-quant analyze AAPL --portfolio 50000 --bars 10`
3. **Place orders**: `omega-quant order AAPL buy 100 --port 4002`

## CLI Commands

### Check IB Gateway connectivity

```bash
omega-quant check --port 4002
# → {"connected": true, "host": "127.0.0.1", "port": 4002}
```

- Port 4002 = paper trading (default, safe)
- Port 4001 = live trading (real money)

### Analyze — stream trading signals

```bash
omega-quant analyze AAPL --portfolio 50000 --port 4002 --bars 10
```

Streams JSONL (one JSON signal per 5-second bar). Each signal contains:
- `regime`: Bull / Bear / Lateral (HMM-detected market state)
- `filtered_price`: Kalman-filtered price (noise removed)
- `merton_allocation`: optimal portfolio allocation [-0.5, 1.5]
- `kelly_fraction`: fractional Kelly bet size
- `kelly_position_usd`: recommended position in dollars
- `direction`: Long / Short / Hold
- `action`: Long/Short/Hold/ReducePosition/Exit with urgency
- `confidence`: signal confidence score [0, 1]
- `reasoning`: human-readable summary

### Order — place a trade

```bash
omega-quant order AAPL buy 100 --port 4002
# → {"status": "Completed", "filled_qty": 100.0, "avg_price": 185.52, ...}
```

## Signal Interpretation

- **Bull regime + Long direction + kelly_should_trade=true** → strong buy signal
- **Bear regime + Short direction** → consider reducing exposure
- **Lateral regime** → typically Hold, wait for regime change
- **confidence > 0.5** → higher conviction signal
- **merton_allocation > 0.1** → math says go long; < -0.1 → go short

## Prerequisites

IB Gateway or TWS must be running locally:
- Paper trading: port 4002 (default)
- Live trading: port 4001
- Docker: `docker run -d -p 4002:4002 ghcr.io/gnzsnz/ib-gateway:latest`
- Auth is handled by the IB Gateway app — no API keys in code

## Safety Guardrails

| Guardrail | Default | Purpose |
|-----------|---------|---------|
| Mode | paper | Paper trading by default (port 4002) |
| Daily trades | 10 | Cap on number of trades per day |
| Daily USD | $50,000 | Cap on total USD traded per day |
| Cooldown | 5 min | Minimum wait between trades |
| Circuit breaker | 2% | Abort TWAP if price deviates >2% |
| Signals | advisory | Marked "NOT FINANCIAL ADVICE" |

## Removing Quant

To completely remove quant from Omega, simply delete `skills/ibkr-quant/`. Zero gateway changes needed.
