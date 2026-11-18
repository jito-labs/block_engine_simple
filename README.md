# Half Baked Block Engine

## About
This is a legacy mock block engine for testing bundles with older Jito-Solana revisions.

## Shortcomings
- The bare minimum methods are implemented for a block engine to forward bundles to a jito-solana validator.
- Bundles are forwarded to all connected leaders instead of the current leader.
- Untested, unaudited, and definitely buggy.

## Local bundle testing on current Jito-Solana

Current Jito-Solana calls `GetBlockEngineEndpoints` before subscribing to bundles.
This mock predates that RPC. Use the in-tree tests to exercise bundle execution
and `simulateBundle` locally on current Jito-Solana.

From the root of a [Jito-Solana checkout](https://github.com/jito-foundation/jito-solana), run the following commands.

```bash
git submodule update --init --recursive
./cargo test --locked -p solana-core --features agave-unstable-api,dev-context-only-utils --lib bundle_stage::tests
./cargo test --locked -p solana-rpc-test --features agave-unstable-api --test simulate_bundle
```

## Running the legacy tools
### Startup the block engine:
```bash
cargo b --release && RUST_LOG=info ./target/release/jito-block-engine
```

### Startup bundle blaster:
```bash
solana-keygen new --no-bip39-passphrase --outfile keypair.json
cargo b --release && RUST_LOG=info ./target/release/jito-searcher-client
```
