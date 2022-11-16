# Half Baked Block Engine

## About
This is a half-baked block engine. It can be used for testing bundles running through jito-solana.

## Shortcomings
- The bare minimum methods are implemented for a block engine to forward bundles to a jito-solana validator.
- Bundles are forwarded to all connected leaders instead of the current leader.
- Untested, unaudited, and definitely buggy.

## Running
### Startup the block engine:
```bash
cargo b --release && RUST_LOG=info ./target/release/jito-block-engine
```

### Startup the validator (jito-solana):
Build the validator: `cargo b --release`

In one terminal, run: `./start`

In another termianl, run: `./bootstrap`

### Startup bundle blaster:
```bash
solana-keygen new --no-bip39-passphrase --outfile keypair.json
cargo b --release && RUST_LOG=info ./target/release/jito-searcher-client
```
