# GMX-Solana

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana v1.18.26](https://docs.anza.xyz/cli/install)
- [Anchor v0.30.1](https://www.anchor-lang.com/docs/installation)
- [Node](https://nodejs.org/en/download)
- [Just](https://github.com/casey/just?tab=readme-ov-file#installation)

### Commands

To run all tests:

```bash
just
```

To install the `gmsol` CLI:

```bash
cargo install-gmsol
```

Use the following command to verify the CLI is installed properly:

```bash
gmsol --version
```

### Troubleshooting

#### 1. Failed to start `test-validator` on MacOS

**Error Message:**

```bash
Error: failed to start validator: Failed to create ledger at test-ledger: io error: Error checking to unpack genesis archive: Archive error: extra entry found: "._genesis.bin" Regular
```

**Posssible Solution:**

Check [this comment](https://github.com/solana-labs/solana/issues/35629#issuecomment-2501133871).

