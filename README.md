# squawkbus

A broker based message bus supporting "selectfeed", authentication and authorization.

Common uses for this kind of message bus are:

* Event driven calculation servers
* Real time distribution of "permissioned" data

## Features

### Publish / Subscribe

The broker follows a standard pub-sub pattern. Publisher's send data to a "topic".
Subscribers subscribe to "topic patterns" which may include wildcard characters
(? for a single character, * for multiple characters).

The data is sent as "packets" of **bytes**, so any kind of message can be sent.

### Notification

Clients may request "notification" of subscriptions to a pattern. For example,
if the pattern was "NASDAQ.*", and a second client subscribed to "NASDAQ.AMZN",
the first client would be "notified" of this subscription.

### Send (Peer to Peer)

One client may send data directly to another.

### Selectfeed

Combining "notification" and "sending" enables a "selectfeed" pattern. This is
where a client only provides streaming data, when it has been requested. This
can be compared to the broadcast pattern when all data is sent to all clients.

Typically the client will be sent an initial data set (image)
via a "send" followed by changes (deltas) via a "publish".

### Authentication

The broker supports:

* Anonymous (no authentication)
* Password file
* LDAP

### Authorization

If authentication is enabled the feed can filter data sent to a client. For
example if an authenticated user is entitled to see level 1 NYSE data, but not
level 2, the broker will only send the level 1 data.

### Data Packets

Data is sent and received in "packets". Each packets has:

* Entitlements - a list of integers
* Headers - key-value pairs
* Data - an array of bytes

When data is sent to the broker it will only forward packets which
match the entitlements of the receiving client. This means that if the publisher
tags each packet with it's entitlements, the client will only receive data to
which it is entitled.

The headers can hold meta data. This is often the content type (e.g. JSON),
timestamps, etc.

### Disconnection

When a client disconnects, other "interested" clients are informed.

For example a client receiving notifications will be told when a subscriber has
disconnected (as well as when they unsubscribe). A client that has subscribe to
a topic will be informed when all publishers to the topic have disconnected.

## Usage

For the system to work a server must be running!

```bash
squawkbus
```

### Logging

Use the `RUST_LOG` environment variable to enable logging.

The only argument is a the path to a configuration file.

#### Start the server without TLS

The only argument is the location of the config file.

```bash
RUST_LOG=debug cargo run --bin squawkbus -- --authorizations-file /authorizations-simple.yaml
```

#### Start the server with TLS

The only argument is the location of the config file.

```bash
RUST_LOG=debug cargo run --bin squawkbus -- \
    --authorizations-file etc/authorizations-simple.yaml \
    --tls $HOME/.keys/server.crt $HOME/.keys/server.key
```

### Clients

#### Start the clients without TLS

```bash
cargo run --bin client -- -h localhost -p 8080
```

#### Start the clients with TLS

On Unix

```bash
cargo run --bin client -- -h beast.jetblack.net -p 8080 --tls --cafile /etc/ssl/certs/ca-certificates.crt
```

On Mac

```bash
cargo run --bin client -- -h brick.jetblack.net -p 8080 --tls --cafile /Users/rtb/.keys/ca-certificates.crt
```

#### Using the client

With one client subscribe to a topic starting with `PUB.`.

```bash
subscribe PUB.foo
```

With another client publish data on the topic. Note that the
second argument is the "entitlements" which is a command separated list of integers.

```bash
publish PUB.foo 0 hello
```

