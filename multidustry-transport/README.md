# multidustry-transport

Type-safe, transport-agnostic communication layer for Multidustry distributed system.

## Features

- 🔒 **Type-safe** - compile-time guarantees for message types
- 🔌 **Transport-agnostic** - seamless inproc and QUIC support
- 🎯 **Service discovery** - automatic routing via UUID
- 📦 **Zero-copy inproc** - efficient local communication
- 🌐 **QUIC networking** - low-latency distributed mode
- 🔄 **Multiplexing** - multiple message types on single connection

## Architecture


## Usage

### Server (Listener)

```rust
use multidustry_transport::*;

let listener = TransportServer::<Request, Response>::builder()
    .uuid(service_uuid)
    .build()
    .create()
    .await;

let connection = listener.accept().await?;
let (tx, rx) = connection.split();
```

### Client

```rust
let connection = TransportClient::<Request, Response>::builder()
    .uuid(service_uuid)
    .timeout(Duration::from_secs(5))
    .retry(3)
    .build()
    .create()
    .await?;

let (tx, rx) = connection.split();
tx.send(request).await?;
let response = rx.recv().await?;
```

## Transport Modes

### Inproc (Monolith)
- Zero dependencies
- Same-process communication via channels
- Automatic when listener is local

### QUIC (Distributed)
- Network communication via QUIC protocol
- CBOR serialization
- Length-prefixed framing
- Automatic when listener is remote

## Status

- ✅ Inproc transport
- 🚧 QUIC transport (60% complete)
- 🚧 KV-based service discovery
- ⏳ Connection pooling
- ⏳ Load balancing

## License

Apache 2.0
