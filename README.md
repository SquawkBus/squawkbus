# squawkbus

A broker based message bus supporting "selectfeed", authentication and authorization.

Common uses for this kind of message bus are:

* Event driven calculation servers
* Real time distribution of "permissioned" data

## Features

### Publish / Subscribe

The broker follows a standard pub-sub pattern. Clients subscribe to topic patterns.
Other clients publish to topics, which gets routed to the subscribers. The data
is sent as *packets* of bytes, so any kind of message can be sent.

### Data Packets

Data is sent and received as a number of "packets". Each packets has:

* Entitlements - a list of integers
* Headers - key-value pairs
* Data - an array of bytes

When the data is received by the broker it will only forward packets which
match the entitlements of the receiving client. This means that as long as the
publisher tags each packet with it's entitlements, clients will only receive
data to which they're entitled.

The headers can hold meta data. This is often the content type (e.g. JSON),
timestamps, etc.

### Notification

Clients may request *notification* of subscriptions to a topic pattern. For example,
if the pattern was "NASDAQ.*", and a second client subscribed to "NASDAQ.AMZN",
the first client would be notified of this subscription.

### Send (Peer to Peer)

One client may send data directly to another.

### Selectfeed

The *selectfeed* pattern is common in market data distribution systems. When a
client subscribes to a ticker, it receives an initial *image*. Subsequently this
client (and other subscribers) receive *deltas* (updates).

Combining notification and sending enables the selectfeed pattern.
The publisher requests notifications on the topic pattern for which it is publishing.
When a client subscribes, an initial image is sent. This is followed by deltas
which are published to all subscribers.

### Authentication

The broker supports:

* Anonymous (no authentication)
* Password file
* LDAP

### Authorization

If authentication is enabled the feed can filter data sent to a client. For
example if an authenticated user is entitled to see level 1 NYSE data, but not
level 2, the broker will only send the level 1 data.

### Disconnection

When a client disconnects, other "interested" clients are informed.

For example a client receiving notifications will be informed when a subscriber has
disconnected (as well as when they unsubscribe). A client that has subscribed to
a topic will be informed when all publishers to the topic have disconnected.

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

### TLS

The data can be encrypted with TLS. An authenticated feed is typically encrypted
to keep the password secret.

```bash
squawkbus \
    --tls server.crt server.key
```

### Password file authentication

Simple password file encryption is provided as a basic authentication mechanism.

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

Authorizations are typically better specified in a file. Here is an example:

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


```bash
squawkbus \
    --tls server.crt server.key \
    --authentication ldap ldap::/ns1.example.com \
    --authorization-file "authorization.yaml"
```
