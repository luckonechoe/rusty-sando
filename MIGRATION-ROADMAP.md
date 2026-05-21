# rusty-sando → alloy v1 + revm v33 Migration Roadmap

**Strategy:** Option C — drop `foundry-evm` and `cfmms-rs` entirely. Reimplement the
~100 LOC of CFMM math we actually use; build directly on `revm` v33 + `alloy` v1.

**Reference docs (canonical patterns):**
- `C:\Users\stale\OneDrive\Documents\MEV\skill-updates\references\alloy-v1.md`
- `C:\Users\stale\OneDrive\Documents\MEV\skill-updates\references\ethers-to-alloy-migration.md`
- `C:\Users\stale\OneDrive\Documents\MEV\skill-updates\references\revm-simulation.md`
- `C:\Users\stale\OneDrive\Documents\MEV\skill-updates\references\token-safety.md`
- `C:\Users\stale\OneDrive\Documents\MEV\skill-updates\references\eip-4844-2930.md`
- `C:\Users\stale\OneDrive\Documents\MEV\skill-updates\references\recovery-and-reorg.md`
- `flashbots-mev.md` (cited for Phase 5 — not yet locally available; pull from skill bundle).

---

## 1. Codebase Map

Workspace root: `bot/Cargo.toml` (3-crate workspace).

### 1.1 `artemis-core` (`bot/crates/artemis-core`)

- **LOC:** 471
- **Files:** 10
- **Role:** Generic event-bus engine (collectors → strategies → executors).
- **Public API surface (consumed by `strategy` and `sando-bin`):**
  - Traits: `Collector<E>`, `Executor<A>`, `Strategy<E, A>`
  - Types: `Engine<E, A>`, `CollectorMap`, `ExecutorMap`, `CollectorStream<E>`
  - Event/action structs: `BlockInfo`, `MempoolTx`, `NewBlock`, `SubmitTxToFlashbots`,
    `BundleRequest` re-exports.
  - Built-in collectors: `BlockCollector`, `MempoolCollector`, `LogCollector`.
  - Built-in executors: `MempoolExecutor`, `FlashbotsExecutor`.
- **Entry points:** `lib.rs` re-exports `engine`, `collectors`, `executors`, `types`.
- **Direct dep call sites:**
  - `ethers`: ~14 sites — `Provider<Ws>`, `Middleware`, `Block`, `Transaction`,
    `H256`, `Filter`, `Log`, `BlockId`, `BlockNumber`.
  - `ethers-flashbots`: 1 module (`executors/flashbots_executor.rs`) —
    `FlashbotsMiddleware`, `BundleRequest`, `SignerMiddleware<Provider, LocalWallet>`.
  - `anvil`: 1 site — Anvil-style state-override RPC plumbing inside the
    block-collector helper (`anvil_setStorageAt` / `anvil_impersonateAccount` patterns
    used in tests).
  - `foundry-evm`: 0.
  - `cfmms`: 0.

### 1.2 `strategy` (`bot/crates/strategy`)

- **LOC:** 2 142
- **Files:** 22
- **Role:** All sandwich logic — pool sync, mempool filtering, simulation,
  bundle assembly, salmonella detection.
- **Public API surface (consumed by `sando-bin`):**
  - `SandoBot::new(provider, sando_address, searcher_signer) -> Self`
  - `SandoBot: Strategy<Event, Action>` impl
  - Re-exported types: `BlockInfo`, `RawIngredients`, `SandoRecipe`, `Pool` (cfmms re-export),
    `PoolManager`, `BlockManager`, `SandoState`.
  - `helpers::access_list_to_revm`, `helpers::eth_to_wei`, `constants::*`.
- **Entry points:** `lib.rs`, `bot.rs` (`SandoBot` struct), `simulator/mod.rs`.
- **Direct dep call sites:**
  - `foundry_evm`: **17 imports** across 3 files
    - `simulator/lil_router.rs`: `executor::{fork::SharedBackend, Bytecode, ExecutionResult, Output, TransactTo}`, `revm::{db::CacheDB, primitives::{keccak256, AccountInfo, Address as rAddress, U256 as rU256}, EVM}`.
    - `simulator/huff_sando.rs`: same `executor::*`, `executor::TxEnv`, `executor::inspector::AccessListTracer`, `revm::*`.
    - `simulator/salmonella_inspector.rs`: `executor::InstructionResult`, `revm::{interpreter::{opcode, Interpreter}, Database, EVMData, Inspector}`.
  - `cfmms`: **11 imports**
    - `pool::Pool::{UniswapV2, UniswapV3}` pattern matching (lil_router.rs, huff_sando.rs, bot.rs, types.rs).
    - `pool::{UniswapV2Pool, UniswapV3Pool}` field access (`token_a`, `token_b`, `address()`, `fee`).
    - `sync` and `checkpoint` modules in `managers/pool_manager.rs`.
  - `ethers`: **34 imports** — `types::{U256, U64, H160, H256, Bytes, Address, Log, Transaction, BlockId}`, `abi::{self, parse_abi, Token, ParamType}`, `prelude::BaseContract`, `providers::{Provider, Ws, Middleware}`, `signers::LocalWallet`.
  - `anvil`: 1 site — `anvil::eth::util::get_precompiles_for(spec_id)` in `simulator/huff_sando.rs:1`.
  - `revm` (transitively via `foundry_evm::revm`): 14 sites — `primitives::{Address, U256, Bytes, AccountInfo, Bytecode, keccak256}`, `db::CacheDB`, `Database`, `Inspector`, `interpreter::{opcode, Interpreter}`.

### 1.3 `sando-bin` (`bot/sando-bin`)

- **LOC:** 167
- **Files:** 4 (`main.rs`, `runner.rs`, `prelude.rs`, `relayer.rs`)
- **Role:** Wires env config → providers → signers → `SandoBot` → `artemis-core::Engine`.
- **Entry point:** `main.rs::main()`.
- **Direct dep call sites:**
  - `ethers`: ~12 sites — `providers::{Provider, Ws, Middleware}`, `signers::{LocalWallet, Signer}`,
    `middleware::SignerMiddleware`, `types::{Address, U256}`.
  - `ethers-flashbots`: re-uses `artemis-core::executors::FlashbotsExecutor`.
  - `foundry-evm`, `cfmms`, `anvil`: 0.

---

## 2. Dependency Impact Matrix

| Old symbol (ethers / foundry-evm / cfmms / anvil) | New (alloy v1 / revm v33 / local) | Reference |
|---|---|---|
| `ethers::providers::Provider<Ws>` | `alloy::providers::RootProvider<PubSubFrontend>` via `ProviderBuilder::new().on_ws(...)` | alloy-v1.md §Provider |
| `ethers::providers::Middleware` trait | `alloy::providers::Provider` trait | ethers-to-alloy-migration.md §Provider |
| `ethers::types::U256` | `alloy::primitives::U256` (drop `.as_u128()`, use `U256::from`/`.to::<u128>()`) | ethers-to-alloy-migration.md §Primitives |
| `ethers::types::H160` / `Address` | `alloy::primitives::Address` (20-byte) | ethers-to-alloy-migration.md §Primitives |
| `ethers::types::H256` | `alloy::primitives::B256` | ethers-to-alloy-migration.md §Primitives |
| `ethers::types::Bytes` | `alloy::primitives::Bytes` | ethers-to-alloy-migration.md §Primitives |
| `ethers::types::U64` | `u64` directly OR `alloy::primitives::U64` | ethers-to-alloy-migration.md §Primitives |
| `ethers::types::{Block, Transaction, Log, Filter, BlockId, BlockNumber}` | `alloy::rpc::types::eth::{Block, Transaction, Log, Filter, BlockId, BlockNumberOrTag}` | alloy-v1.md §RPC types |
| `ethers::types::transaction::eip2930::AccessList` | `alloy::eips::eip2930::AccessList` | eip-4844-2930.md |
| `ethers::abi::{self, encode, parse_abi, Token, ParamType}` | `alloy::sol!` macro + `alloy::sol_types::{SolCall, SolValue}` | alloy-v1.md §ABI |
| `ethers::prelude::BaseContract` | `alloy::sol!` generated bindings or `alloy::contract::Interface` | alloy-v1.md §Contract bindings |
| `ethers::signers::LocalWallet` | `alloy::signers::local::PrivateKeySigner` | ethers-to-alloy-migration.md §Signers |
| `ethers::signers::Signer` trait | `alloy::signers::Signer` trait | ethers-to-alloy-migration.md §Signers |
| `ethers::middleware::SignerMiddleware` | `ProviderBuilder::new().wallet(signer).on_*` (wallet filler) | alloy-v1.md §Wallet filler |
| `ethers_flashbots::FlashbotsMiddleware` | Custom `FlashbotsClient` — alloy `PrivateKeySigner` signs `X-Flashbots-Signature`, plain `reqwest` POST to relay | flashbots-mev.md §Bundle submission |
| `ethers_flashbots::BundleRequest` | Local `FlashbotsBundleRequest` struct serialised with serde | flashbots-mev.md §Bundle JSON |
| `foundry_evm::executor::fork::SharedBackend` | Custom `AlloyDb<P>: revm::Database` wrapping `RootProvider`; or use `revm::db::AlloyDB` if available in revm v33 helpers | revm-simulation.md §Forking |
| `foundry_evm::revm::EVM::new() / .database() / .transact_commit()` | `revm::Evm::builder().with_db(db).with_external_context(...).modify_block_env(...).build().transact_commit()` | revm-simulation.md §ContextBuilder |
| `foundry_evm::revm::db::CacheDB` | `revm::db::CacheDB` (same crate, re-imported direct) | revm-simulation.md §CacheDB |
| `foundry_evm::executor::{ExecutionResult, Output, TransactTo}` | `revm::primitives::{ExecutionResult, Output, TxKind}` (`TransactTo::Call(addr)` → `TxKind::Call(addr)`) | revm-simulation.md §Tx env |
| `foundry_evm::executor::TxEnv` | `revm::primitives::TxEnv` (`access_list` field type changed to `Vec<AccessListItem>`) | revm-simulation.md §TxEnv |
| `foundry_evm::executor::Bytecode` | `revm::primitives::Bytecode::new_raw(Bytes)` | revm-simulation.md §AccountInfo |
| `foundry_evm::executor::inspector::AccessListTracer` | `revm-inspectors::access_list::AccessListInspector` (separate crate) or roll our own (~80 LOC) | revm-simulation.md §Inspectors |
| `foundry_evm::revm::Inspector<DB>` w/ `step(&mut self, interp, EVMData, bool)` | `revm::Inspector<DB>` w/ `step(&mut self, interp: &mut Interpreter, ctx: &mut EvmContext<DB>)` — **signature change** | revm-simulation.md §Inspector trait |
| `foundry_evm::revm::EVMData<'_, DB>` | `revm::EvmContext<DB>` | revm-simulation.md §Inspector trait |
| `foundry_evm::revm::interpreter::{opcode, Interpreter}` | `revm::interpreter::{opcode, Interpreter}` | revm-simulation.md §Inspector |
| `foundry_evm::revm::primitives::{keccak256, AccountInfo, Address, U256}` | `revm::primitives::{keccak256, AccountInfo, Address, U256}` (alloy types under the hood) | revm-simulation.md §Primitives |
| `foundry_evm::revm::primitives::InstructionResult` | `revm::interpreter::InstructionResult` | revm-simulation.md §Inspector |
| `anvil::eth::util::get_precompiles_for(spec_id)` | `revm::precompile::Precompiles::new(PrecompileSpecId::from_spec_id(spec_id)).addresses().copied().collect::<HashSet<_>>()` | revm-simulation.md §Precompiles |
| `anvil_setStorageAt` / `anvil_impersonateAccount` (RPC) | `db.insert_account_storage(addr, slot, value)` / `db.insert_account_info(addr, info)` directly on `CacheDB` | revm-simulation.md §State override |
| `cfmms::pool::Pool::UniswapV2(p)` enum | Local `enum SandoPool { V2(SandoV2Pool), V3(SandoV3Pool) }` | (new) §4 below |
| `cfmms::pool::UniswapV2Pool { token_a, token_b, address(), fee, .. }` | Local `SandoV2Pool { token0, token1, address, fee_bps }` | (new) §4 below |
| `cfmms::pool::UniswapV3Pool { token_a, token_b, address(), fee, .. }` | Local `SandoV3Pool { token0, token1, address, fee_pips }` | (new) §4 below |
| `cfmms::sync::sync_pairs` / discovery | New `pool_discovery::sync_v2_pairs(provider, factory, from_block, to_block)` using alloy `PairCreated` log filter; same for V3 `PoolCreated` | (new) §4 below |
| `cfmms::checkpoint::*` (JSON pool-cache) | Plain serde JSON read/write of `Vec<SandoPool>` | (new) §4 below |

---

## 3. CFMM Math Inventory

The bot uses far less off-chain CFMM math than `cfmms-rs` provides. Concrete usage:

| Location | Math / pool data | Reimplementation |
|---|---|---|
| `simulator/huff_sando.rs::v2_get_amount_out` (lines 372–437) | V2 constant-product `amount_out = (a_in*997*r_out) / (r_in*1000 + a_in*997)`; reserves fetched live via `getReserves()` EVM call | Keep as-is, swap `ethers::abi::decode` → `alloy::sol!` `getReserves()Call::abi_decode_returns` and `U256` types → alloy. ~30 LOC. |
| `simulator/huff_sando.rs` and `lil_router.rs` — pattern match `cfmms::pool::Pool::{UniswapV2(p), UniswapV3(p)}` | Pool variant dispatch | Local `SandoPool` enum with same two variants. |
| `tx_utils/huff_sando_interface/v2.rs`, `v3.rs` (build_swap_v2_data / build_swap_v3_data) | Builds calldata from `pool.address()`, `pool.token_a/b`, `pool.fee` | Read fields from `SandoV2Pool` / `SandoV3Pool` directly. No math change. |
| `tx_utils/lil_router_interface.rs` | `decode_swap_v2_result` / `decode_swap_v3_result` ABI decoders | Replace `ethers::abi::decode` with `alloy::sol!` `decode_returns`. |
| `managers/pool_manager.rs` | Pool discovery + caching (uses `cfmms::sync` + `cfmms::checkpoint`) | Reimplement: alloy log subscription on factory `PairCreated(token0,token1,pair,uint256)` and V3 `PoolCreated(token0,token1,fee,tickSpacing,pool)`. ~80 LOC. |
| `simulator/lil_router.rs` | WETH balance slot encoding `keccak256(abi.encode(addr, U256::from(3)))` | Replace `ethers::abi::encode([Token::Address(...), Token::Uint(3)])` with `alloy::sol_types::SolValue::abi_encode(&(addr, U256::from(3)))` or manual `[address.into_word(), B256::from(U256::from(3))].concat()`. |

**Verdict:** No `sqrtPriceX96`/tick-math is used off-chain — V3 amounts are obtained by
running `swap` inside revm. **Total reimplementation surface ≈ 100–150 LOC** (one `SandoPool`
module + two ABI decoders + one factory-log poller).

**No external CFMM library required.** Optional: pull `uniswap-v3-math` crate later if/when
we want off-chain V3 quotes; not needed for migration.

---

## 4. Migration Phases

Effort key: **S** ≤ 0.5 day · **M** 1–2 days · **L** 3–5 days · **XL** 1+ week.

### Phase 1 — Workspace + dep skeleton (S)

**Goal:** Cargo.toml updated; `cargo check` fails with real type errors, not "crate not found".

**Files touched:**
- `bot/Cargo.toml` (workspace deps)
- `bot/crates/artemis-core/Cargo.toml`
- `bot/crates/strategy/Cargo.toml`
- `bot/sando-bin/Cargo.toml`

**Changes:**
- Remove: `ethers = ...`, `ethers-flashbots = ...`, `foundry-evm = { git = ... }`, `cfmms = { git = ... }`, `anvil = { git = ... }`.
- Add (workspace):
  ```toml
  alloy = { version = "1", features = ["full", "pubsub", "ws", "rpc-types-eth", "signer-local", "sol-types", "contract"] }
  alloy-primitives = "1"
  alloy-sol-types = "1"
  revm = { version = "33", default-features = false, features = ["std", "serde"] }
  revm-inspectors = "0.x"
  reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
  ```
- Pin all `revm`/`alloy`/`reqwest` versions in workspace `[workspace.dependencies]`.

**Acceptance:** `cargo check -p artemis-core 2>&1 | grep -c "could not find"` → `0`.
Errors are now `unresolved import alloy::providers::...` style — i.e. real migration TODOs.

---

### Phase 2 — Provider + types layer (M)

**Goal:** All `ethers::types::*` and `ethers::providers::*` replaced. Code compiles up to
the simulator boundary.

**Files touched:**
- `artemis-core/src/types.rs`, `engine.rs`
- `artemis-core/src/collectors/*.rs` (block, mempool, log)
- `artemis-core/src/executors/mempool_executor.rs`
- `strategy/src/types.rs`, `bot.rs`, `helpers.rs`, `constants.rs`
- `strategy/src/managers/{block_manager,sando_state_manager}.rs`
- `sando-bin/src/{main,prelude,runner,relayer}.rs`

**Substeps:**
1. Replace primitives: `U256`/`H160`/`H256`/`Bytes`/`Address` → alloy. Use sed-style
   rg-and-edit; `as_u128()` → `.to::<u128>()`, `.from(u64)` patterns may stay as
   `U256::from`.
2. Provider: `Provider<Ws>` → `RootProvider<PubSubFrontend>`. Build via
   `ProviderBuilder::new().on_ws(WsConnect::new(url)).await?`.
3. Subscriptions:
   - `provider.subscribe_blocks()` → `provider.subscribe_blocks().await?.into_stream()`.
   - `provider.subscribe_pending_txs()` → `provider.subscribe_pending_transactions().await?` +
     fetch full tx via `provider.get_transaction_by_hash(hash).await?`.
4. RPC types: `Block<TxHash>` → `alloy::rpc::types::Block`, `Transaction` → `alloy::rpc::types::Transaction` etc. Field names mostly identical; `tx.from`, `tx.to`, `tx.input` (now `Bytes`), `tx.value`.
5. AccessList: `helpers::access_list_to_revm` rewritten — alloy `AccessList` already
   matches revm's `Vec<AccessListItem>` 1:1; the helper may collapse to a direct
   `.into()`.
6. Constants: `LIL_ROUTER_ADDRESS: Address = address!("...")` via `alloy::primitives::address!`.

**Test strategy:**
- Unit: round-trip `serde_json` of `BlockInfo`, `MempoolTx`.
- Integration: `cargo build -p artemis-core` fully green.

**Acceptance:**
- `cargo build -p artemis-core` succeeds.
- `strategy` errors confined to `simulator/*.rs`, `tx_utils/*.rs`, `managers/pool_manager.rs`.
- `sando-bin` connects to a WS endpoint in a smoke test (`cargo run -- --check-rpc`).

---

### Phase 3 — Simulation layer (revm v33) (L)

**Goal:** All `foundry_evm::*` and the old `EVM::new()` API replaced with revm v33
`ContextBuilder`/`Evm::builder()`. Salmonella inspector ported to new `Inspector` trait.

**Files touched:**
- `strategy/src/simulator/mod.rs` — `setup_block_state` rewritten against `BlockEnv`.
- `strategy/src/simulator/lil_router.rs` — full rewrite of `evaluate_sandwich_revenue`.
- `strategy/src/simulator/huff_sando.rs` — full rewrite of `create_recipe`,
  `get_erc20_balance`, `v2_get_amount_out`.
- `strategy/src/simulator/salmonella_inspector.rs` — Inspector trait migration.
- `strategy/src/helpers.rs` — `access_list_to_revm` simplified.
- New: `strategy/src/simulator/db.rs` — `AlloyForkDb<P>: revm::Database` (replaces
  `foundry_evm::SharedBackend`).
- New: `strategy/src/simulator/precompiles.rs` — replaces `anvil::get_precompiles_for`.
- New: `strategy/src/simulator/access_list.rs` — replaces foundry's `AccessListTracer`
  (or use `revm-inspectors`).

**Critical changes:**
1. **Backend:** `SharedBackend` → custom `AlloyForkDb<P>` impl `revm::Database`:
   ```rust
   impl<P: Provider> revm::Database for AlloyForkDb<P> {
       type Error = anyhow::Error;
       fn basic(&mut self, addr: Address) -> Result<Option<AccountInfo>> { /* eth_getBalance + eth_getCode + eth_getTransactionCount via tokio::block_on */ }
       fn code_by_hash(&mut self, hash: B256) -> Result<Bytecode> { /* cache */ }
       fn storage(&mut self, addr: Address, slot: U256) -> Result<U256> { /* eth_getStorageAt */ }
       fn block_hash(&mut self, n: u64) -> Result<B256> { /* eth_getBlockByNumber */ }
   }
   ```
   Cache via `Mutex<HashMap<...>>` or wrap with `revm::db::CacheDB`.
2. **EVM construction:** `EVM::new()` + `evm.database(db)` → `Evm::builder().with_db(db).with_block_env(block_env).build()`.
3. **Tx env mutation:** `evm.env.tx.* = ...` → `evm.context.evm.env.tx.* = ...` OR rebuild
   per-tx via `.modify_tx_env(|tx| { tx.caller = ...; })` and call `.transact_commit()`.
4. **Inspector trait** (most invasive):
   ```rust
   // OLD
   impl<DB: Database> Inspector<DB> for SalmonellaInspectoooor {
       fn step(&mut self, interp: &mut Interpreter, _data: &mut EVMData<'_, DB>, _is_static: bool) -> InstructionResult { ... }
   }
   // NEW
   impl<DB: Database> Inspector<DB> for SalmonellaInspectoooor {
       fn step(&mut self, interp: &mut Interpreter, _ctx: &mut EvmContext<DB>) { ... }
   }
   ```
   No return value; suspicious-opcode flagging is unchanged. `interp.current_opcode()`
   API is preserved.
5. **AccessList tracer:** prefer `revm-inspectors::access_list::AccessListInspector::new(from, to, precompiles)`. Run with `Evm::builder()...with_external_context(inspector).append_handler_register(inspector_handle_register).build()` then call `.transact_commit()` and read `inspector.access_list()`.
6. **Precompiles:** `anvil::get_precompiles_for(spec_id)` → wrap revm:
   ```rust
   pub fn precompiles_for(spec: SpecId) -> HashSet<Address> {
       Precompiles::new(PrecompileSpecId::from_spec_id(spec)).addresses().copied().collect()
   }
   ```

**Test strategy:**
- New `tests/simulator_smoke.rs`: spin up `MockProvider` (or anvil-rs) at a known block,
  run `evaluate_sandwich_revenue` against a static V2 pool, assert revenue > 0.
- Salmonella: feed bytecode containing `BALANCE`/`COINBASE` and assert `NotSafu`.
- Per `token-safety.md` invariants: assert backrun output is `WETH` (not target token);
  assert balance check uses token of `start_end_token`.

**Acceptance:**
- `cargo test -p strategy --test simulator_smoke` passes.
- `cargo clippy -p strategy -- -D warnings` clean.
- Inspector counts opcodes correctly on a 5-op fixture.

---

### Phase 4 — CFMM math reimplementation (S–M)

**Goal:** `cfmms` symbol fully removed.

**Files touched:**
- New: `strategy/src/pools/mod.rs` — `SandoPool`, `SandoV2Pool`, `SandoV3Pool`.
- New: `strategy/src/pools/discovery.rs` — log-based factory poller (V2 `PairCreated`,
  V3 `PoolCreated`).
- New: `strategy/src/pools/cache.rs` — JSON checkpoint read/write.
- Modify: `strategy/src/managers/pool_manager.rs` — use new types.
- Modify: `strategy/src/simulator/huff_sando.rs`, `lil_router.rs`, `bot.rs`,
  `tx_utils/huff_sando_interface/v2.rs`, `v3.rs`, `tx_utils/lil_router_interface.rs` —
  pattern match local enum.

**Substeps:**
1. Define `SandoPool` enum (V2/V3 variants) mirroring exact field names used by
   `tx_utils` (`token_a`, `token_b`, `address()`, `fee`).
2. V2 reserves call: `alloy::sol! { interface IUniswapV2Pair { function getReserves() external view returns (uint112, uint112, uint32); } }`.
3. V3 quoter: not needed off-chain (we simulate).
4. Discovery: subscribe `provider.get_logs(Filter::new().address(factory).event_signature(PAIR_CREATED_TOPIC))`, parse via `sol!`-generated `PairCreated::decode_log`.
5. Cache: replace cfmms-checkpoint JSON with `serde_json::to_writer_pretty(&Vec<SandoPool>)`.

**Test strategy:**
- Unit: golden V2 `getAmountOut` math against known reserves (Uniswap V2 docs example:
  `1000` in, reserves `100k/100k` → `996`).
- Discovery: replay 100 historical `PairCreated` logs and assert pool count.

**Acceptance:**
- `rg -n "cfmms" bot/` returns 0 hits.
- `cargo build -p strategy` clean.

---

### Phase 5 — Flashbots bundle layer (M)

**Goal:** `ethers-flashbots` removed; we sign + POST bundles ourselves.

**Files touched:**
- Replace: `artemis-core/src/executors/flashbots_executor.rs` (full rewrite).
- New: `artemis-core/src/flashbots/{client.rs, types.rs, signer.rs}`.
- Modify: `strategy/src/bot.rs` (action emission) and `sando-bin/src/relayer.rs`.

**Implementation:**
- `FlashbotsBundleRequest` struct (raw transactions, target block, optional
  `minTimestamp`, `maxTimestamp`, `revertingTxHashes`). serde-serialised to JSON-RPC
  `eth_sendBundle` per Flashbots spec.
- Signer: alloy `PrivateKeySigner` signs `keccak256(json_body)` → `X-Flashbots-Signature: 0x{addr}:0x{sig}` header.
- HTTP: `reqwest::Client` POST to `https://relay.flashbots.net` (and configurable additional
  relays).
- Multi-relay: parallel `futures::future::join_all` POST to N relays; aggregate errors.
- Per `flashbots-mev.md`: include `eth_callBundle` simulation pre-flight, and respect
  EIP-4844 blob-tx exclusion (sandwich bundles contain no blob txs).

**Test strategy:**
- Integration: dry-run against Goerli/Sepolia Flashbots staging relay; assert 200 OK.
- Signature: golden test against known body+key → known sig (cross-check
  ethers-flashbots output using a fixed fixture).

**Acceptance:**
- `rg -n "ethers" bot/crates/artemis-core` returns 0 hits.
- Bundle round-trip on Sepolia returns relay-acked `bundleHash`.

---

### Phase 6 — Strategy / searcher logic glue (S)

**Goal:** Tie everything together; `SandoBot::process_event` works end-to-end.

**Files touched:**
- `strategy/src/bot.rs` — pattern matches over local `SandoPool`, alloy types
  throughout.
- `strategy/src/managers/{block_manager.rs, pool_manager.rs, sando_state_manager.rs}`.
- `sando-bin/src/runner.rs` wires the new `Engine`.

**Test strategy:**
- E2E replay test: against an archive node, take a known historical mempool tx that
  was sandwiched, run `SandoBot::process_event`, assert recipe profit > 0 and
  recipe `frontrun_data` matches historical bundle (modulo gas-price drift).
- Per `recovery-and-reorg.md`: assert reorg-detection logic still triggers
  `SandoState::on_reorg`.

**Acceptance:**
- `cargo test --workspace` green.
- `cargo run -p sando-bin -- --dry-run` connects, ingests blocks, emits no
  bundles (no profit) without panicking, for 100 mainnet blocks.

---

### Phase 7 — Huff executor contract (deferred) (S, conditional)

**Goal:** Decide whether `contract/src/sando.huff` needs updating.

**Conditions to update:**
- New EVM features (PUSH0 / EIP-3855 / Cancun opcodes) would reduce gas — check `huff-neo` skill.
- Deployment script changes due to alloy.

**Files touched:** `contract/src/sando.huff`, `contract/test/*.t.sol`, deploy script in
`sando-bin` (bytecode constant in `constants.rs`).

**Test strategy:** existing Foundry tests in `contract/`.

**Acceptance:** `forge test` passes; bytecode in `LIL_ROUTER_CODE` regenerated.

---

## 5. Risks & Open Questions (need user decision)

1. **Is `artemis-core` a fork of paradigmxyz/artemis or custom?**
   The dir layout (`engine.rs`, `collectors/`, `executors/`, `types.rs`) matches
   paradigm artemis exactly, but vendored locally. Decide:
   - (a) Replace with upstream `artemis-core` crate (does upstream support alloy v1 yet? — check before committing).
   - (b) Keep fork, migrate in place.
   Recommendation: **(b) keep fork** — upstream paradigm artemis was not actively
   alloy-ported as of last public commit; vendoring is cheaper than chasing upstream.

2. **Keep the 3-crate split?**
   `artemis-core` (471 LOC) + `strategy` (2 142 LOC) + `sando-bin` (167 LOC).
   Decide:
   - (a) Keep — clean dep direction, easy to test in isolation.
   - (b) Collapse `artemis-core` into `strategy` (we're the only consumer).
   Recommendation: **(a) keep** — `artemis-core` traits are reused by `FlashbotsExecutor`
   and would otherwise leak into `strategy`.

3. **Huff executor contract (`contract/src/sando.huff`) — update it?**
   Migration does **not** require it. But:
   - If we want to leverage PUSH0 / TLOAD-TSTORE for cheaper sandwich, Phase 7 fires.
   - If we leave it alone, `LIL_ROUTER_CODE` in `constants.rs` stays as a hex
     literal — no change.
   **Defer to user.**

4. **`uniswap-v3-math` crate — do we want off-chain V3 quoting?**
   Current bot simulates V3 in revm. Adding off-chain math (`uniswap-v3-math` crate
   on crates.io) would be optional and unblock faster optimal-input search. Out of
   scope for the migration but worth flagging.

5. **`AlloyForkDb` async-in-sync gymnastics**
   `revm::Database` is sync. Alloy provider calls are async. Standard solutions:
   - (a) `tokio::task::block_in_place(|| handle.block_on(provider.get_storage_at(...)))` — needs multi-thread runtime.
   - (b) Pre-fetch all touched slots via dry-run trace, then sync-only.
   - (c) Use `revm::db::AlloyDB` (if shipped in revm v33 helpers — verify).
   Recommendation: **start with (a), measure**, fall back to (c) if revm ships it.

6. **`ethers-flashbots` replacement effort estimate**
   `flashbots-mev.md` reference was cited but not yet read into this audit. Phase 5
   effort is **M** assuming the doc covers `eth_sendBundle` JSON shape, signature
   header format, and multi-relay strategy. If it doesn't, bump to **L**.

7. **Mempool subscription parity**
   `ethers::Provider<Ws>::subscribe_pending_txs()` returns just hashes; we re-fetch
   full tx. Some providers (Erigon, Reth) support `newPendingTransactionsWithBody`
   which alloy exposes via `provider.subscribe_full_pending_transactions()`. Worth
   using to halve RPC calls in the mempool collector — flag for Phase 2 follow-up.

8. **EIP-4844 blob handling**
   Per `eip-4844-2930.md` invariants: sandwich bundles must not include type-3 (blob)
   transactions, and we must skip blob-tx victims (their `maxFeePerBlobGas` field
   makes ROI math meaningless until blob-fee oracle is in). Add a filter in
   `MempoolCollector` to drop blob txs early. Confirm with user.

9. **Token-safety invariants** (from `token-safety.md`)
   The audit found no current rebasing-token / fee-on-transfer guard in the bot.
   Migration is a good time to add `assert(post_balance >= pre_balance + expected_out)`
   guards in `huff_sando.rs::create_recipe`. Out of strict scope but flag as
   follow-up.

---

## Summary — scope & ordering

```
Phase 1 (S)  ─►  Phase 2 (M)  ─►  Phase 3 (L)  ─►  Phase 4 (S-M)  ─►  Phase 5 (M)  ─►  Phase 6 (S)
                                       │
                                       └──► Phase 7 (S, conditional)
```

Total estimated effort (no Phase 7): **~10–14 working days** for one engineer.
Phase 3 is the long pole; everything else is mechanical.

Critical-path blockers needing user answers **before Phase 1**:
- Q1 (artemis fork-vs-upstream)
- Q3 (huff contract update y/n)
- Q6 (`flashbots-mev.md` content sufficient?)
