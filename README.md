# Development

Clone the `solana-playground` repository, if not cloned:

```bash
git clone https://github.com/regolith-labs/solana-playground ../solana-playground
```

Install `dioxus-cli`, if not installed:

```bash
cargo install dioxus-cli
```

Run the following command in the root of the project to start the Dioxus dev server:

```bash
dx serve --hot-reload
```

- Open the browser to http://localhost:8080

To rebuild the wallet adapter:
```
cd wallet-adapter && npm run build
```
