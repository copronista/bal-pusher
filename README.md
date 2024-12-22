# bal-pusher

`bal-pusher` is a tool that retrieves Bitcoin transactions from a database and pushes them to the Bitcoin network when their **locktime** exceeds the **median time past** (MTP). It listens for Bitcoin block updates via ZMQ.

## Installation

To use `bal-pusher`, you need to compile and install Bitcoin with ZMQ (ZeroMQ) support enabled. Then, configure your Bitcoin node and `bal-pusher` to push the transactions.

### Prerequisites

1. **Bitcoin with ZMQ Support**:
   Ensure that Bitcoin is compiled with ZMQ support. Add the following line to your `bitcoin.conf` file:

   ```
   zmqpubhashblock=tcp://127.0.0.1:28332
   ```

2. **Install Rust and Cargo**:
   If you haven't already installed Rust and Cargo, you can follow the official instructions to do so: [Rust Installation](https://www.rust-lang.org/tools/install).

### Installation Steps

1. Clone the repository:

   ```bash
   git clone <repo-url>
   cd bal-pusher
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

3. Install the binary:

   ```bash
   sudo cp target/release/bal-pusher /usr/local/bin
   ```

## Configuration

`bal-pusher` can be configured using environment variables. If no configuration file is provided, a default configuration file will be created.

### Available Configuration Variables

| Variable                              | Description                              | Default                                      |
|---------------------------------------|------------------------------------------|----------------------------------------------|
| `BAL_PUSHER_CONFIG_FILE`              | Path to the configuration file. If the file does not exist, it will be created. | `$HOME/.config/bal-pusher/default-config.toml` |
| `BAL_PUSHER_DB_FILE`                  | Path to the SQLite3 database file. If the file does not exist, it will be created. | `bal.db`                                      |
| `BAL_PUSHER_ZMQ_LISTENER`             | ZMQ listener for Bitcoin updates.        | `tcp://127.0.0.1:28332`                      |
| `BAL_PUSHER_BITCOIN_HOST`             | Bitcoin server host for RPC connections. | `http://127.0.0.1`                           |
| `BAL_PUSHER_BITCOIN_PORT`             | Bitcoin RPC server port.                 | `8332`                                       |
| `BAL_PUSHER_BITCOIN_COOKIE_FILE`      | Path to Bitcoin RPC cookie file.         | `$HOME/.bitcoin/.cookie`                     |
| `BAL_PUSHER_BITCOIN_RPC_USER`         | Bitcoin RPC username.                    | -                                            |
| `BAL_PUSHER_BITCOIN_RPC_PASSWORD`     | Bitcoin RPC password.                    | -                                            |


## Running `bal-pusher`

Once the application is installed and configured, you can start `bal-pusher` by running the following command:

```bash
$ bal-pusher
```

This will start the service, which will listen for Bitcoin blocks via ZMQ and push transactions from the database when their locktime exceeds the median time past.
