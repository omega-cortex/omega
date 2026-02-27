---
name = "doli-miner"
description = "DOLI blockchain producer node management — wallet, mining, rewards, node health, service setup."
trigger = "doli|miner|producer|bond|wallet|rewards|epoch|node|block|mining"
---

# DOLI Miner

Day-to-day operations for a DOLI blockchain producer (miner).

## What it covers

- **Wallet** — create, addresses, balance, send, sign/verify, backup/restore
- **Producer** — register, add bonds, status, list producers, withdrawals, exit, slashing
- **Rewards** — automatic via coinbase (Bitcoin-like), check with `doli balance`
- **Node health** — chain status, peer count, sync state, process check
- **Node management** — init, status, import/export blocks, recover state
- **Updates** — check, vote, apply, rollback (3/5 maintainer multisig)
- **Service** — create and manage systemd (Linux) or launchd (macOS) service
- **Troubleshooting** — common issues, clock drift, RocksDB locks, resync

## Install

Download pre-built binaries from [GitHub Releases](https://github.com/e-weil/doli/releases/latest). Each package includes `doli-node` (full node) and `doli` (wallet CLI).

**macOS (Apple Silicon):**
```bash
sudo installer -pkg doli-VERSION-aarch64-apple-darwin.pkg -target /
```

**Ubuntu/Debian:**
```bash
curl -LO https://github.com/e-weil/doli/releases/latest/download/doli-VERSION-x86_64-unknown-linux-gnu.deb
sudo dpkg -i doli-VERSION-*.deb
```

**Fedora/RHEL:**
```bash
curl -LO https://github.com/e-weil/doli/releases/latest/download/doli-VERSION-x86_64-unknown-linux-gnu.rpm
sudo rpm -i doli-VERSION-*.rpm
```

**From tarball (any platform):**
```bash
tar -xzf doli-VERSION-*.tar.gz
sudo cp doli-VERSION-*/doli-node doli-VERSION-*/doli /usr/local/bin/
```

Verify: `doli --version && doli-node --version`

Data directories: `~/.doli/mainnet/`, `~/.doli/testnet/`, `~/.doli/devnet/`.

### First run

```bash
doli new                  # 1. create wallet (write down the 24-word seed phrase!)
doli-node run --yes       # 2. start node (syncs mainnet)
doli chain                # 3. check sync progress
doli balance              # 4. check balance once synced
```

## Address format

Bech32m: `doli1...` (mainnet), `tdoli1...` (testnet), `ddoli1...` (devnet).

Both `doli1...` addresses and 64-char hex pubkey hashes are accepted everywhere.

## Global options

Global options go **BEFORE** the subcommand:

```bash
# CORRECT
doli -w /path/to/wallet.json balance
doli-node --data-dir /path run --producer

# WRONG
doli balance -w /path/to/wallet.json
doli-node run --data-dir /path
```

| Option | Description | Default |
|--------|-------------|---------|
| `-w, --wallet <PATH>` | Wallet file path | `~/.doli/wallet.json` |
| `-r, --rpc <URL>` | Node RPC endpoint | `http://127.0.0.1:8545` |

## Wallet

### Create wallet

```bash
doli new [--name NAME]    # creates wallet + 24-word BIP-39 seed phrase
```

**After creation, always show the user:**

1. The **24-word recovery phrase** (numbered, in a clear grid)
2. The **primary address** (`doli1...`)
3. The **wallet file path** (e.g. `~/.doli/wallet.json`)
4. The **seed file path** (e.g. `~/.doli/wallet.seed.txt`)
5. **Critical warning**: Write down the 24 words on paper, then delete the seed file:
   ```bash
   rm ~/.doli/wallet.seed.txt
   ```

The seed phrase is saved to a separate `.seed.txt` file — it is **not** stored in the wallet JSON. If you lose both the wallet file and the seed words, your funds are unrecoverable.

### Addresses

A wallet can hold **multiple addresses**. The primary address is derived from the seed phrase. Additional addresses are random keypairs (not seed-derived).

```bash
doli info                            # show wallet metadata (name, version, all keys) — READ-ONLY
doli addresses                       # list all addresses in the wallet
doli address [--label "savings"]     # generate NEW address (WARNING: mutates the wallet file!)
```

**Important**: `doli address` **generates** a new address and appends it to the wallet. Use `doli info` or `doli addresses` to inspect without mutating.

### Balance

```bash
doli balance                         # show balance for ALL addresses in the wallet
doli balance --address doli1...      # check specific address
doli balance --address a1b2c3d4...   # hex pubkey hash also works
```

Shows 4 balance types per address:

| Type | Description |
|------|-------------|
| **Confirmed** | Spendable balance (mature UTXOs) |
| **Unconfirmed** | Pending transactions in mempool |
| **Immature** | Coinbase/epoch rewards pending 100-block maturity |
| **Total** | Sum of all balances |

### Send

```bash
doli send doli1recipient... 20       # fee auto-calculated
doli send doli1recipient... 20 --fee 0.001   # explicit fee override
```

Fee is auto-calculated as `max(1000, inputs × 500)` units.

### Transaction history

```bash
doli history                         # last 10 transactions
doli history --limit 20              # custom limit
```

### Backup & restore

```bash
doli export ~/backup/wallet.json     # backup wallet
doli import ~/backup/wallet.json     # restore wallet
```

### Sign & verify messages

```bash
doli sign "Hello, world!" [--address doli1...]    # sign a message
doli verify "Hello, world!" <signature> <pubkey>  # verify signature
```

### Multi-node operations

Use `-w` to select which key signs, `-r` to target a specific node's RPC:

```bash
doli -w ~/.doli/mainnet/keys/producer_1.json balance
doli -w ~/.doli/mainnet/keys/producer_2.json -r http://127.0.0.1:8546 send doli1... 50
```

Producer key files are wallet-compatible — use directly with `-w`.

## Producer

### Register

```bash
doli producer register               # 1 bond (default)
doli producer register --bonds 5     # 5 bonds (more bonds = more block slots)
```

Bond costs: **10 DOLI/bond** (mainnet), **1 DOLI/bond** (devnet). Max: 10,000 bonds.

More bonds = proportionally more block production slots in the deterministic rotation.

### Status & list

```bash
doli producer status                 # your status: active/unbonding/exited, bonds, withdrawals
doli producer status --pubkey <PK>   # check another producer
doli producer list                   # all producers
doli producer list --active          # active only
```

### Add bonds (bond stacking)

```bash
doli producer add-bond --count 3     # add 3 more bonds
```

### Withdrawal

```bash
doli producer request-withdrawal --count 2                     # start 7-day unbonding
doli producer request-withdrawal --count 2 --destination doli1...  # specify destination
doli producer claim-withdrawal                                 # claim after delay
doli producer claim-withdrawal --index 1                       # claim specific withdrawal
```

### Exit

```bash
doli producer exit                   # normal exit (after commitment period)
doli producer exit --force           # early exit with penalty
```

Early exit penalty: `<1yr = 75%`, `1-2yr = 50%`, `2-3yr = 25%`, `3yr+ = 0%`.

### Report equivocation (slashing)

```bash
doli producer slash --block1 <HASH> --block2 <HASH>   # same slot, different blocks
```

100% of the offender's bond is burned permanently.

## Rewards

Rewards work like **Bitcoin**: producers receive 1 DOLI/block automatically via coinbase transaction. **No claiming needed.**

- Rewards appear as **Immature** until 100 confirmations, then become **Confirmed** and spendable
- Halving every ~4 years (12,614,400 blocks)
- Check your rewards with `doli balance`

```bash
doli rewards info                    # current epoch progress and config
```

The `rewards list`, `rewards claim`, `rewards claim-all`, `rewards history` commands are **deprecated** (from a removed presence-based system). They return empty results.

## Chain status

```bash
doli chain                           # height, hash, slot, network info
```

## Node management (doli-node)

### Run

```bash
# Non-producer (sync only)
doli-node run --yes

# Producer node
doli-node --data-dir ~/.doli/mainnet/data run \
  --producer --producer-key ~/.doli/mainnet/keys/producer.json \
  --yes --force-start

# Testnet
doli-node --network testnet run
```

Key run options:

| Option | Description |
|--------|-------------|
| `--producer` | Enable block production |
| `--producer-key <PATH>` | Producer key file (required with --producer) |
| `--no-auto-update` | Disable automatic updates |
| `--p2p-port <PORT>` | P2P listen port |
| `--rpc-port <PORT>` | RPC listen port |
| `--bootstrap <ADDR>` | Bootstrap node multiaddr |
| `--no-dht` | Disable DHT discovery |
| `--chainspec <PATH>` | Path to chainspec JSON file |

### Other node commands

```bash
doli-node init --network mainnet     # initialize data directory
doli-node status                     # show node status
doli-node import <path>              # import blocks from file
doli-node export <path> --from 0 --to 1000   # export blocks
doli-node recover [--yes]            # recover chain state from blocks
```

Network ports:

| Network | P2P | RPC | Metrics |
|---------|-----|-----|---------|
| Mainnet | 30303 | 8545 | 9090 |
| Testnet | 40303 | 18545 | 19090 |
| Devnet | 50303 | 28545 | 29090 |

## Updates & governance (doli-node)

3/5 maintainer multisig with 7-day veto period and 40% stake veto threshold.

```bash
doli-node update check                                    # check for updates
doli-node update status                                   # pending update status
doli-node update vote --approve --key producer.json       # approve update
doli-node update vote --veto --key producer.json          # veto update
doli-node update votes [--version VERSION]                # view votes
doli-node update apply [--force]                          # apply approved update
doli-node update rollback                                 # rollback to previous version
doli-node update verify --version VERSION                 # verify release

doli-node maintainer list                                 # list maintainers
doli-node maintainer add --target <PK> --key maint.json   # propose add (3/5 sig)
doli-node maintainer remove --target <PK> --key maint.json  # propose remove
doli-node maintainer sign --proposal-id <ID> --key maint.json  # sign proposal
doli-node maintainer verify --pubkey <PK>                 # verify status
```

## Service setup

**Linux (systemd):**
```bash
sudo systemctl start doli-mainnet     # start
sudo systemctl stop doli-mainnet      # stop
sudo systemctl status doli-mainnet    # status
journalctl -u doli-mainnet -f         # logs
```

**macOS (launchd):**
```bash
launchctl load ~/Library/LaunchAgents/network.doli.mainnet.plist    # start
launchctl stop network.doli.mainnet                                 # stop
launchctl list | grep doli                                          # status
tail -f ~/.doli/mainnet/node.log                                    # logs
```

## Troubleshooting

**Node not producing blocks:**
1. `doli producer status` — verify active
2. Check process has `--producer` flag: `pgrep -la doli-node`
3. Check peer count > 0: `doli chain`
4. Check clock sync: `date -u` vs `ntpdate -q pool.ntp.org`

**No peers:** Open P2P port (30303), verify bootstrap node.

**RocksDB LOCK error:** Another `doli-node` process is running. Kill it first — never delete the LOCK file.

**Clock drift:** Max allowed is 1 second. Fix: `sudo sntp -sS pool.ntp.org` (macOS) or `sudo timedatectl set-ntp true` (Linux).

**Stuck sync:** Restart node. If stuck at height 0, check bootstrap and firewall.

**Fork/corruption:** Stop node, delete state files (keep keys!), restart to resync:
```bash
rm -f chain_state.bin producers.bin utxo.bin
rm -rf blocks/ signed_slots.db/
```
