---
name: "ibkr-quant"
description: "Autonomous trading via IBKR — multi-asset (stocks, forex, crypto), bracket orders, scanner, positions, P&L monitoring."
trigger: "quant|trading|signal|ibkr|market|regime|kelly|position|interactive brokers|stock|portfolio|analyze|forex|crypto|scanner|bracket|pnl|close"
---

# IBKR Quant Engine — Autonomous Trading

You have `omega-quant`, a standalone CLI for quantitative trading via Interactive Brokers. You are the strategist — omega-quant provides the tools; you make the decisions.

## Commands Reference

### 1. `check` — Verify connectivity

```bash
omega-quant check --port 4002
```

Returns: `{"connected": true, "host": "127.0.0.1", "port": 4002}`

**Always check connectivity before any other command.**

### 2. `scan` — Find instruments by volume/activity

```bash
# Most active US stocks
omega-quant scan --scan-code MOST_ACTIVE --instrument STK --location STK.US.MAJOR --count 10 --port 4002

# Hot crypto by volume
omega-quant scan --scan-code HOT_BY_VOLUME --instrument CRYPTO --location CRYPTO.PAXOS --count 5 --port 4002

# Stocks above $10 with high volume
omega-quant scan --scan-code HOT_BY_VOLUME --instrument STK --location STK.US.MAJOR --count 10 --min-price 10 --min-volume 1000000 --port 4002
```

Scan codes: `MOST_ACTIVE`, `HOT_BY_VOLUME`, `TOP_PERC_GAIN`, `TOP_PERC_LOSE`, `HIGH_OPEN_GAP`, `LOW_OPEN_GAP`

Returns: JSON array of `{rank, symbol, security_type, exchange, currency}`

### 3. `analyze` — Stream trading signals

```bash
# Stock
omega-quant analyze AAPL --asset-class stock --portfolio 50000 --bars 10 --port 4002

# Forex
omega-quant analyze EUR/USD --asset-class forex --portfolio 50000 --bars 10 --port 4002

# Crypto
omega-quant analyze BTC --asset-class crypto --portfolio 50000 --bars 10 --port 4002
```

Each signal contains: `regime`, `regime_probabilities`, `filtered_price`, `trend`, `merton_allocation`, `kelly_fraction`, `kelly_position_usd`, `kelly_should_trade`, `direction`, `action`, `execution`, `confidence`, `reasoning`

**Signal interpretation:**
- Bull regime + Long + kelly_should_trade=true + confidence > 0.5 → **strong buy**
- Bear regime + Short + confidence > 0.5 → **strong sell/short**
- Lateral regime → **hold, wait for regime change**
- merton_allocation > 0.1 → math says long; < -0.1 → short

### 4. `order` — Place trades (market or bracket)

```bash
# Simple market order
omega-quant order AAPL buy 100 --asset-class stock --port 4002

# Bracket order with SL/TP (percentages from entry)
omega-quant order AAPL buy 100 --asset-class stock --stop-loss 1.5 --take-profit 3.0 --port 4002

# Bracket with safety checks (P&L cutoff + max positions)
omega-quant order AAPL buy 100 --asset-class stock --stop-loss 1.5 --take-profit 3.0 --account DU1234567 --portfolio 50000 --max-positions 3 --port 4002

# Forex bracket
omega-quant order EUR/USD buy 20000 --asset-class forex --stop-loss 0.5 --take-profit 1.0 --port 4002

# Crypto bracket
omega-quant order BTC buy 0.1 --asset-class crypto --stop-loss 2.0 --take-profit 5.0 --port 4002
```

Safety checks (automatic when flags provided):
- `--max-positions N`: blocks if current positions >= N (default: 3)
- `--account` + `--portfolio`: blocks if daily P&L < -5% of portfolio

Bracket orders create 3 linked orders: MKT entry → LMT take-profit → STP stop-loss.

### 5. `positions` — List open positions

```bash
omega-quant positions --port 4002
```

Returns: JSON array of `{account, symbol, security_type, quantity, avg_cost}`
- Positive quantity = long position
- Negative quantity = short position

### 6. `pnl` — Daily P&L

```bash
omega-quant pnl DU1234567 --port 4002
```

Returns: `{daily_pnl, unrealized_pnl, realized_pnl}`

### 7. `close` — Close a position

```bash
# Close entire position (auto-detects side and quantity)
omega-quant close AAPL --asset-class stock --port 4002

# Partial close
omega-quant close AAPL --asset-class stock --quantity 50 --port 4002

# Close forex
omega-quant close EUR/USD --asset-class forex --port 4002
```

## Strategy Rules (YOU MUST FOLLOW)

1. **Always use bracket orders**: Every entry must have `--stop-loss` and `--take-profit`. Default: SL 1.5%, TP 3.0% unless the user specifies otherwise.

2. **Max 3 simultaneous positions**: Always check `positions` before entering. Use `--max-positions 3`.

3. **Never same instrument 2 days in a row**: Track via conversation memory. If you traded AAPL yesterday, skip AAPL today.

4. **Time-based asset selection**:
   - US market hours (9:30am-4:00pm ET): stocks
   - Outside US hours: prioritize crypto and forex

5. **Pre-entry checklist** (every single trade):
   - `check` → connectivity OK
   - `positions` → count < 3
   - `pnl ACCOUNT` → daily P&L > -5%
   - `analyze SYMBOL` → confidence > 0.5 AND kelly_should_trade = true
   - Only then → `order` with bracket

6. **Exit discipline**:
   - Let bracket orders handle exits (SL/TP)
   - Manual close only if regime changes dramatically (Bull → Bear)
   - Check positions every monitoring cycle

## Reporting Format

After every action, report to the user:

```
[TRADE] BUY 100 AAPL @ $185.50
  SL: $182.72 (-1.5%) | TP: $191.07 (+3.0%)
  Confidence: 72% | Regime: Bull | Kelly: $2,400
  Daily P&L: +$150.30 | Positions: 2/3
```

## Autonomous Loop (via SCHEDULE_ACTION)

When the user activates autonomous trading:

1. **Every 5 minutes**: `scan` → `analyze` top candidates → enter if criteria met
2. **Every 1 minute**: `positions` + `pnl` → monitor, close if regime flipped
3. Report every action via Telegram

Use `SCHEDULE_ACTION` markers to set up these loops:
```
SCHEDULE_ACTION: 5m | Scan and analyze top instruments for trading opportunities
SCHEDULE_ACTION: 1m | Monitor open positions and P&L, close if regime changed
```

## Safety

- **Paper first**: Always `--port 4002` unless user explicitly says "live" or "real money"
- **Not financial advice**: Always include disclaimer that signals are advisory
- **Circuit breaker**: Auto-aborts if price deviates >2% during execution
- **Daily limits**: Max 10 trades/day, $50k/day, 5-min cooldown (enforced in Rust)
- **P&L cutoff**: Halt all trading if daily loss exceeds 5% of portfolio

## Prerequisites

IB Gateway or TWS must be running locally. If `check` shows `"connected": false`:
- Docker: `docker run -d -p 4002:4002 ghcr.io/gnzsnz/ib-gateway:latest`
- Or launch IB Gateway / TWS manually
