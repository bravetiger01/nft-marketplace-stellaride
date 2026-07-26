# Soroban NFT Marketplace

A full-stack NFT marketplace built on the **Stellar** blockchain using **Soroban** smart contracts and **Next.js**. Users can mint NFTs, create collections, list NFTs for sale, make offers, and trade with built-in royalty support and platform fees.

## Features

- **Mint NFTs** with metadata URIs, collection grouping, and configurable royalties (up to 10%)
- **Create Collections** to organize NFTs with name, description, and image
- **List & Buy** — list NFTs for sale, buyers purchase with a SPL-like payment token
- **Offers** — make offers on any NFT, sellers accept or buyers cancel
- **Royalties** — creators automatically receive royalties on secondary sales
- **Platform Fees** — configurable fee (up to 10%) paid to the marketplace admin on every sale
- **Transfer** — send NFTs to any address
- **Freighter Wallet** integration for seamless Testnet signing

## Architecture

```
project/
├── contract/                  # Soroban smart contract (Rust)
│   ├── Cargo.toml
│   └── contracts/contract/
│       └── src/
│           ├── lib.rs         # Contract logic (NftMarketplace)
│           └── test.rs        # 39 passing tests
└── client/                    # Next.js 16 frontend
    └── src/
        ├── app/               # App Router pages
        ├── components/        # Navbar, Contract (marketplace UI)
        ├── hooks/             # Contract interaction (ScVal wrappers)
        └── lib/               # Utilities
```

**Smart Contract:** `NftMarketplace` on Soroban Testnet  
**Frontend:** Next.js 16 + Tailwind CSS v4  
**Wallet:** Freighter browser extension  
**Network:** Stellar Testnet

## Setup Instructions

### Prerequisites

- [Rust](https://rustup.rs/) with `wasm32v1-none` target
- [Stellar CLI](https://soroban.stellar.org/docs/getting-started/installation) (`stellar`)
- [Node.js](https://nodejs.org/) 18+ and [Bun](https://bun.sh/)
- [Freyer](https://freighter.app) browser extension (Chrome/Brave)

### 1. Clone & Install

```bash
# Install Rust target
rustup target add wasm32v1-none

# Install client dependencies
cd client
bun install
```

### 2. Deploy the Smart Contract

```bash
cd contract

# Build the WASM
stellar contract build

# Generate a funded testnet account
stellar keys generate dev --network testnet --fund

# Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/hello_world.wasm \
  --source-account dev --network testnet
# Save the returned C... contract address

# Create a payment token (SAC)
stellar keys generate token-admin --network testnet --fund
stellar contract asset id --asset USD:token-admin --network testnet
# Save the token C... address

# Initialize the marketplace
stellar contract invoke \
  --id <CONTRACT_ADDRESS> \
  --source-account dev --network testnet \
  -- initialize \
  --admin <DEV_PUBLIC_KEY> \
  --fee-bps 250 \
  --payment-token <TOKEN_ADDRESS>
```

### 3. Configure the Frontend

Update the contract address in `client/src/hooks/contract.ts`:

```ts
export const CONTRACT_ADDRESS = "C...your_deployed_contract_address";

const PAYMENT_TOKEN = "C...your_payment_token_address";
```

### 4. Run the Frontend

```bash
cd client
bun run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

### 5. Run Contract Tests

```bash
cd contract
cargo test
# All 39 tests should pass
```

## Usage

1. **Connect Wallet** — Click "Connect Wallet" in the navbar, approve in Freighter
2. **Mint** — Go to the "Mint NFT" tab, fill in Token ID, Metadata URI, Collection, Royalty
3. **List** — Go to "My NFTs", enter Token ID and price, click List
4. **Buy** — Browse marketplace, click "Buy Now" on any listing
5. **Transfer** — Go to "My NFTs", enter Token ID and recipient address
6. **Collections** — Create and search collections in the "Collections" tab

## Screenshots

### Wallet Connected State

<img width="1469" height="878" alt="Screenshot 2026-07-26 at 8 16 08 PM" src="https://github.com/user-attachments/assets/efd7261d-46c4-4fde-a0f0-0dd3a6c2996a" />


The navbar displays the connected wallet address (truncated) with a green connection indicator and the user's payment token balance.

### Balance Displayed
<img width="1469" height="878" alt="Screenshot 2026-07-26 at 8 16 08 PM" src="https://github.com/user-attachments/assets/e13c3ece-755b-4c20-b006-9a8e97e95240" />


When a wallet is connected, the payment token balance is shown in the navbar alongside the wallet address. Balances update on page load and after transactions.

### Successful Testnet Transaction
<img width="1469" height="878" alt="Screenshot 2026-07-26 at 8 16 08 PM" src="https://github.com/user-attachments/assets/595233d1-709d-433d-80a0-aa7f0e190903" />


After a successful on-chain transaction (mint, list, buy, or transfer), a toast notification appears in the top-right corner showing the operation result with a clickable link to view the transaction on Stellar Expert.

## Smart Contract API

| Function | Description |
|---|---|
| `initialize(admin, fee_bps, payment_token)` | Set up the marketplace |
| `mint(caller, token_id, metadata_uri, collection, royalty_bps)` | Mint a new NFT |
| `get_nft(token_id)` | Get NFT details |
| `transfer_nft(from, to, token_id)` | Transfer an NFT |
| `list(seller, token_id, price)` | List an NFT for sale |
| `delist(seller, token_id)` | Remove a listing |
| `buy(buyer, token_id)` | Purchase a listed NFT |
| `make_offer(buyer, token_id, amount, expires_at)` | Make an offer |
| `accept_offer(seller, token_id, buyer)` | Accept an offer |
| `cancel_offer(buyer, token_id)` | Cancel an offer |
| `create_collection(creator, name, description, image_uri)` | Create a collection |
| `get_collection(name)` | Get collection details |
| `get_listings(offset, limit)` | Browse marketplace (paginated) |
| `get_user_nfts(user)` | Get NFTs owned by a user |
| `get_collection_nfts(name)` | Get NFTs in a collection |
| `set_fee(new_fee)` | Admin: update platform fee |

## Tech Stack

- **Smart Contract:** Rust + Soroban SDK v25
- **Frontend:** Next.js 16 (App Router) + React 19 + TypeScript
- **Styling:** Tailwind CSS v4
- **Blockchain SDK:** @stellar/stellar-sdk v16
- **Wallet:** @stellar/freighter-api v6
- **Runtime:** Bun

## License

MIT
