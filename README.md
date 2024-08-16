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

### Wild-carding

This is currently regex, but it might be faster to use something more
like RabbitMQ.

### Entitlements

This could be made more efficient with caching,
however we might want to put in some code to prevent bad actors swamping
the cache and crashing the host.

A mechanism to reload entitlements (e.g. on SIGHUP).

Maybe entitlements should be exposed as a pluggable module with dynamic
access.

### Authentication

This is currently hardcoded to `nobody/trustno1`.

Alternatives include:

* Password file
* LDAP
* Pluggable authentication

### Serialization

This was the first thing to be written and could do with reworking.

We could wrap the packet in a length delimited frame to simplify the
serialization of primitives (i.e. to make that part non-async).

### Configuration and Command Line

It would be nice to have to control over the configuration with command line params.

Should entitlements be split into a separate file?
