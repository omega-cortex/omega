# omega-quant — Technical Specification

## Overview

Quantitative trading engine crate providing real-time market analysis, advisory signals, and order execution. Connects to Interactive Brokers (IBKR) via the TWS API (ibapi crate). Exposed as a standalone CLI binary (`omega-quant`) invoked by the AI through the `ibkr-quant` skill — zero gateway coupling.

## Architecture

```
skills/ibkr-quant/SKILL.md          ← teaches the AI when/how to use omega-quant
         │
         │ AI reads instructions, invokes via bash:
         ▼
omega-quant check --port 4002       ← TCP connectivity check
omega-quant analyze AAPL ...        ← stream signals (Kalman→HMM→Kelly→JSON)
omega-quant order AAPL buy 100 ...  ← place order via IBKR
         │
         │ uses library:
         ▼
crates/omega-quant/src/lib.rs       ← math core (QuantEngine)
crates/omega-quant/src/market_data.rs ← IBKR TWS feed
crates/omega-quant/src/executor.rs  ← order execution
```

## Modules

| Module | Purpose |
|--------|---------|
| `bin/main.rs` | CLI binary: `check`, `analyze`, `order` subcommands via clap |
| `signal.rs` | Output types: `QuantSignal`, `Regime`, `Direction`, `Action`, `ExecutionStrategy` |
| `kalman.rs` | Kalman filter (2D state: price + trend, plain f64 math, no nalgebra) |
| `hmm.rs` | Hidden Markov Model (3-state: Bull/Bear/Lateral, 5 observations, Baum-Welch training) |
| `kelly.rs` | Fractional Kelly criterion (position sizing with safety clamps) |
| `market_data.rs` | IBKR TWS real-time price feed via `ibapi` crate with auto-reconnect |
| `execution.rs` | TWAP + Immediate execution plan types |
| `executor.rs` | Live order execution with circuit breaker, daily limits, crash recovery |
| `lib.rs` | `QuantEngine` orchestrator + inline Merton allocation |

## CLI Subcommands

### `omega-quant check`
TCP connectivity check to IB Gateway.
```
omega-quant check --port 4002 --host 127.0.0.1
→ {"connected": true, "host": "127.0.0.1", "port": 4002}
```

### `omega-quant analyze`
Stream trading signals as JSONL (one JSON object per 5-second bar).
```
omega-quant analyze AAPL --portfolio 50000 --port 4002 --bars 30
→ {"timestamp":"...","symbol":"AAPL","raw_price":185.50,"regime":"Bull",...}
```
Uses `QuantEngine.process_price()` for each bar. Stops after `--bars` count.

### `omega-quant order`
Place a market order via IBKR TWS API.
```
omega-quant order AAPL buy 100 --port 4002
→ {"status":"Completed","filled_qty":100.0,"avg_price":185.52,...}
```
Uses `Executor` with default safety limits (circuit breaker, daily caps).

## Pipeline

```
Price tick → Kalman filter → Returns → EWMA volatility
                                    → HMM regime detection
                                    → Merton optimal allocation (inlined)
                                    → Kelly sizing
                                    → Action + Direction
                                    → Execution strategy
                                    → QuantSignal output
```

## Safety Invariants

1. Paper trading by default (port 4002)
2. Human confirms every trade (`require_confirmation` always true)
3. Kelly fraction <= 1.0 (clamped in `KellyCriterion::new()`)
4. Max allocation <= 50% (clamped in `KellyCriterion::new()`)
5. Daily trade limit checked in `Executor::execute()`
6. Daily USD limit checked in `Executor::execute()`
7. Cooldown enforced via `DailyLimits::check()`
8. Circuit breaker at 2% deviation in `execute_twap()`
9. Crash recovery via `ExecutionState` JSON serde
10. Disclaimer in `format_signal()` includes "NOT FINANCIAL ADVICE"

## Skill Integration

The `ibkr-quant` skill (`skills/ibkr-quant/SKILL.md`) teaches the AI:
- When to use omega-quant (trigger keywords: trading, quant, ibkr, signal, etc.)
- How to check IB Gateway connectivity
- How to get and interpret trading signals
- How to place orders with safety rules
- No gateway wiring needed — removing the skill folder removes quant entirely

## Prerequisites

IB Gateway or TWS must be running locally:
- Paper trading: port 4002
- Live trading: port 4001
- Auth handled by IB Gateway app (no API keys in code)

## Dependencies

tokio, serde, serde_json, tracing, anyhow, chrono, uuid, clap, ibapi, rand
