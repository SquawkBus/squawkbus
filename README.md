# A Prototype Message Bus

## Usage

### Start the server

Use the `RUST_LOG` environment variable to enable logging.

The only argument is the location of the config file.

```bash
RUST_LOG=debug cargo run --bin server etc/config-simple.yaml
```

### Start the clients

```bash
cargo run --bin client -- -h beast.jetblack.net -p 8080
```

## Things to do

* Frame IO?
* Configuration.
* Check usize for serialization.
* Command line args.
* TLS.
* File password authentication.
* LDAP authentication.
* log
