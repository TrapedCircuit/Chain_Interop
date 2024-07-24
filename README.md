# IZAR

![iydyuz.png](https://files.catbox.moe/iydyuz.png)


## Build

Before building, Please ensure your machine has `Rust`.

Start by github clone

```sh
git clone https://github.com/izar-bridge/izar.git
```

and build by `cargo`

```sh
cd izar
cargo build --release
```

## Configuration

1. Validator config

   ```toml
   api_dest = "http://127.0.0.1:80" # sequencer destination

   [aleo_config]
   pk = "your-aleo-private-key"
   dest = "http://your-aleo-node-api"
   from_height = 0 # listen from height

   [sepolia_config]
   pk = "your-sepolia-private-key"
   dest = "https://your-sepolia-node-api"
   from_height = 0

   #[zksync_config]
   #....
   #...
   ```

2. Relayer config

   ```toml
   api_dest = "http://127.0.0.1:80" # sequencer destination
   port = 4000 # relayer restful server port

   [aleo_config]
   pk = "your-aleo-private-key"
   dest = "http://your-aleo-node-api"
   from_height = 0 # listen from height but not need

   [sepolia_config]
   pk = "your-sepolia-private-key"
   dest = "https://your-sepolia-node-api"

   #[scroll_config]
   #....
   #....
   ```



## Run

```sh
./target/release/izar-voter -c your_voter_config.toml
./target/release/izar-relayer -c your_relayer_config.toml
```
