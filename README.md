# kalshi-f1

A CLI trading bot (built in Rust) that uses xAI (Grok) + the Kalshi API to automatically identify and place bets on F1 race markets.

The bot uses an agentic loop: it fetches live market data from Kalshi, uses a fast Grok model to decide which trades to make, and uses a reasoning Grok model with web search to generate fair value price estimates for each market. It then places limit orders where the current market price diverges meaningfully from its fair value estimate.

## How It Works

1. The `run` command sends a prompt to `grok-4-1-fast-non-reasoning` (the "trading agent"), giving it access to a set of Kalshi tools.
2. The agent calls tools iteratively (up to 10 times) to gather information and act:
   - `getBalance` — fetch current cash balance
   - `getPortfolioValue` — fetch total portfolio value
   - `getF1Markets` — fetch all open F1 race markets with volume > 1,000 contracts
   - `getOrders` — fetch open orders
   - `getPositions` — fetch current positions
   - `priceMarketsFromTickers` — for each ticker, calls `grok-4-1-fast-reasoning` with web + X search to generate a fair yes-bid price (only use for markets with volume > $500)
   - `createOrder` — place a limit buy or sell order for Yes or No contracts
3. Once the agent produces a final text response, the loop ends and the result is printed.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024)
- A [Kalshi](https://kalshi.com) account with API access (RSA key pair)
- An [xAI](https://x.ai) API key

## Setup

### 1. Clone and build

```sh
git clone https://github.com/yourname/kalshi-f1.git
cd kalshi-f1
cargo build --release
```

The compiled binary will be at `./target/release/fuji`.

### 2. Generate a Kalshi RSA key pair

Follow the [Kalshi API docs](https://trading-api.readme.io/reference/getting-started) to generate an RSA private key and register the public key with your account. Save the private key file somewhere safe (e.g. `~/.kalshi/private_key.pem`).

### 3. Configure credentials

Use the `set-key` subcommand to store your credentials. Config is saved to your OS config directory (`~/.config/fuji/config.toml` on Linux, `~/Library/Application Support/com.fuji.fuji/config.toml` on macOS).

```sh
# xAI / Grok API key
fuji set-key --key grok_key --value "xai-xxxxxxxxxxxx"

# Path to your Kalshi RSA private key file
fuji set-key --key kalshi_key_path --value "/path/to/private_key.pem"

# Your Kalshi API key ID (shown in the Kalshi dashboard)
fuji set-key --key kalshi_id --value "your-kalshi-key-id"
```

### 4. Verify config

```sh
fuji view-config
```

## Usage

### Run with the default prompt

The default prompt instructs the agent to find the best F1 markets to bet on, targeting markets where the current price is off by more than ±6 cents from the fair value estimate.

```sh
fuji run
```

### Run with a custom prompt

```sh
fuji run --prompt "Show me my current balance and open positions."
```

```sh
fuji run --prompt "Find F1 markets where I currently have positions and check if any should be closed."
```

## Commands

| Command | Description |
|---|---|
| `fuji set-key --key <KEY> --value <VALUE>` | Store a config value |
| `fuji view-config` | Print the current config |
| `fuji run` | Run the agent with the default prompt |
| `fuji run --prompt <PROMPT>` | Run the agent with a custom prompt |

### Valid keys for `set-key`

| Key | Description |
|---|---|
| `grok_key` | Your xAI API key |
| `kalshi_key_path` | Path to your Kalshi RSA private key file |
| `kalshi_id` | Your Kalshi API key ID |

## Project Structure

```
src/
├── main.rs              # CLI entry point (clap)
├── config.rs            # Config read/write (stored via `directories` + TOML)
├── kalshi/
│   ├── api.rs           # Authenticated Kalshi HTTP client (RSA-signed requests)
│   ├── balance.rs       # Balance and portfolio value fetching
│   ├── markets.rs       # F1 market fetching and formatting
│   ├── orders.rs        # Open order fetching
│   ├── positions.rs     # Position fetching
│   └── purchase.rs      # Order placement
└── llm/
    ├── llm.rs           # Grok API client, agentic tool-call loop
    └── price_agent.rs   # Fair value pricing via reasoning model + web search
```

## Notes

- All prices on Kalshi are in **cents** (e.g. 60 = $0.60 = 60% implied probability).
- The pricing agent (`price_agent.rs`) uses `grok-4-1-fast-reasoning` with web and X search to estimate fair value. Only pass it tickers with > $500 in volume to avoid low-liquidity edge cases.
- The trading agent loop runs for a maximum of **10 iterations** before stopping.
- Orders are limit orders only.