# squawkbus

A broker based pub-sub message bus supporting authentication,  authorization,
and a novel feature of notification; written in Rust.

Common uses for this message bus are:

* Real time distribution of permissioned data
* Distributed event driven calculation
* Selecteed message distribution

## Features

### Publish / Subscribe

The broker follows a standard pub-sub pattern. Clients subscribe to topic patterns.
Other clients publish to topics, which gets routed to the subscribers. The data
is sent as *packets* of bytes, so any kind of message can be sent.

The topic patterns use glob style where:

- `?` matches exactly one occurrence of any character.
- `*` matches arbitrary many (including zero) occurrences of any character.

An example pattern might be "*:NASDAQ" which would match "AAPL:NASDAQ".

Globbing was chosen over regex because it is up to two orders of magnitude
faster than regex, and is simpler for clients to understand.

### Notification

A novel feature of this message bus is *notification*.

A client can ask to be notified if any client subscribes to a topic.
For example, with the pattern "*:NASDAQ", a notification would be sent
if a client subscribed to "AAPL:NASDAQ".

The notification includes the *client-id* of the client that requested the subscription.

### Send

Instead of publishing to all subscribers a client can send data directly to
another client. A client id is discovered through a notification.

If a publisher has requested notifications on "*:NASDAQ", and a client
subscribes to "AAPL:NASDAQ", the publisher is notified of the subscription
and given the id of the subscribing client. The publisher can then send an
initial message directly to the client with all of the fields. After this it
can just publish updates on the fields that have changed.

### Authentication

The broker supports the following authentication methods:

* Anonymous (no authentication)
* Password file
* LDAP

### Authorization

If a client is authenticated it can be *authorized*.
Authorization describes the *roles* and *entitlements* it has.

Roles include: `Notifier`, `Publisher`, `Subscriber`

Entitlements are represented as a list of integers. An entitlement might allow viewing of level 2 data from a particular exchange, or access to see the P&L of a book.

Entitlements are used by the message broker to filter the messages sent to
subscribers. The message broker will only forward data that a client is
authorized to receive.

## How it works

Messages have the following properties:

* Headers - string key-value pairs
* Data - an array of bytes
* Entitlements - a list of integers

In addition to this data is sent as *packets*.

### Headers & Data

Data is published as bytes. This makes the protocol agnostic to the message
format. It could be protobuf, or JSON, or anything.

Headers are optional string key-value pairs which add meta-data to the
message. For example they might include:

* `content-type: application/json`
* `content-encoding: zstd`

Or if the messages were always in a known format it might be omitted entirely.

### Entitlements

The message broker will only forward data to a client for which it is entitled.
Those entitlements are included in the message it receives.
This allows messages to maintain a chain of entitlements.

If we take a P&L server in a trading bank as an example.
It maintains the P&L for a number of trading teams. The members of each team
can see their team's P&L, but not the others.
When a price is received the P&L for the relevant positions is recalculated and published.

The price message also includes the entitlements required to receive the price.
The P&L server attaches the price entitlements to it's own entitlements making
a chain. Because the message can have many packets (each with it's own entitlements)
the updates can be sent as a single message.

The server will filter out the data that is subscriber is not entitled to see.
This means that only user's with entitlements for the price *and* for the trading
team would receive the data.

### Packets

A message may contain data with different entitlements. For example a price
message may contain level 1 and level 2 data. To support this a message is
sent as packets.

Each packet contains the data, headers and entitlements. This allows a
publisher to send all the data and let the message broker handle the
filtering of data sent to the clients.

As well as splitting the data by entitlements, the packet structure also
allows sending data with different encoding. One packet could be JSON, with
another as an IPC arrow table.

### Disconnection

When a client disconnects, other "interested" clients are informed.

For example a client receiving notifications will be informed when a
subscriber has disconnected (as well as when they unsubscribe). A client
that has subscribed to a topic will be informed when all publishers to the
topic have disconnected.

The disconnection messages offer some useful information. They allow a
publisher to stop sending data when it is informed that there are no
subscriptions. A subscriber can use the information that there are no
publishers to mark data as stale.

### Selectfeed

The *selectfeed* pattern is common in market data distribution systems.

An exchange will typically publish every event. This is called a *broadcast"
feed.

With a *select* feed, data is only sent to subscribing clients.
When a client subscribes to a ticker, it receives an initial *image*. Subsequently this client (and other subscribers) receive *deltas* (updates).

Combining notification and sending images enables the selectfeed pattern.
The publisher requests notifications on the topic pattern for which it is publishing.
When a client subscribes, an initial image is sent. This is followed by deltas
which are published to all subscribers.

### Distributed Calculation Servers

While the majority of the examples have centered around market data, the
architecture lends itself to general event driven calculation of streaming
data.

### WebSockets

In addition to the standard socket interface the service supports connections
with web sockets to allow simple browser access.

## Usage

For the system to work a server must be running!

```bash
squawkbus
```

### Logging

Use the `RUST_LOG` environment variable to enable logging.

```bash
RUST_LOG=debug squawkbus
```

Logging levels include: trace, debug, info, warn and error.

### TLS

The data can be encrypted with TLS. An authenticated feed is typically encrypted
to keep the password secret.

```bash
squawkbus \
    --tls server.crt server.key
```

### Password file authentication

Simple password file encryption is provided as a basic authentication mechanism.
This uses the apache http server `htpasswd` utilities.

```bash
squawkbus \
    --tls server.crt server.key \
    --authentication basic ht.passwd
```

### LDAP authentication

Simple password file encryption is provided as a basic authentication mechanism.

```bash
squawkbus \
    --tls server.crt server.key \
    --authentication ldap ldap::/ns1.example.com
```

### Simple authorization

Authorizations can be made on the command line. Note that the server must 
be authenticating for authorizations to know the user to authorize.

```bash
squawkbus \
    --tls server.crt server.key \
    --authentication ldap ldap::/ns1.example.com \
    --authorization "alex:NYSE.*:Subscriber" \
    --authorization "kai:NYSE.*:Notifier,Publisher"
```

### File authorization

It is common that there are many authorizations. They may be saved in a file.

```yaml
# Harry is the publisher for LSE data.
harry:
  "LSE.*":
    entitlements:
    - &LSE_LEVEL1 1
    - &LSE_LEVEL2 2
    roles: Notifier | Publisher
# Freddy is the publisher for NYSE data.
freddy:
  "NYSE.*":
    entitlements:
    - &NYSE_LEVEL1 3
    - &NYSE_LEVEL2 4
    roles: Notifier | Publisher
# Tom gets both level 1 and 2 data for LSE and NYSE.
tom:
  "LSE.*":
    entitlements:
    - *LSE_LEVEL1
    - *LSE_LEVEL2
    roles: Subscriber
  "NYSE.*":
    entitlements:
    - *NYSE_LEVEL1
    - *NYSE_LEVEL2
    roles: Subscriber
# Dick gets level 1 for NYSE and LSE.
dick:
  "LSE.*":
    entitlements:
    - *LSE_LEVEL1
    roles: Subscriber
  "NYSE.*":
    entitlements:
    - *NYSE_LEVEL1
    roles: Subscriber
```

The file can be used as follows.

```bash
squawkbus \
    --tls server.crt server.key \
    --authentication ldap ldap::/ns1.example.com \
    --authorizations-file "authorizations.yaml"
```

Sending the `HUP` signal to the process forces a re-read of the authorizations
file.

### HTTP Endpoints

The server provides endpoints for health monitoring and metrics. This can be
set with `--http-endpoint <ip-addr>:<port>`.

The following endpoints exist.

* /metrics - for prometheus metrics.
* /health/startup - returns 200 if the server has started.
* /health/readiness - returns 200 if the server is ready to receive data.
* /health/liveness - returns 200 if the server is alive.

### Heartbeat

Periods of no network activity can lead to routers dropping connections.
To keep connections alive a heartbeat message is sent every 30 seconds.
The interval this is sent can be configured with `--heartbeat-seconds <seconds>`.

### Message Size

The default maximum message size is 4,294,967,295. This can be constrained
with `--max-message-size <bytes>`.

### Back Pressure

If a client is unable to handle messages promptly it can lead to them being
queued up on the server. Such a client would be termed a "slow consumer". The
consequence for the server is called "back pressure". Back pressure an cause
the server to run out of memory.

The server imposes a constraint of the maximum number of messages. This can
be configured with `max-queued-messages <count>`. The default is 32.
