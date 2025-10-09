# Multidustry

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![WASM](https://img.shields.io/badge/wasm-component%20model-purple.svg)](https://component-model.bytecodealliance.org/)

> **Alternative Mindustry server implementation focused on horizontal scalability and plugin ecosystem**

Multidustry is a next-generation server implementation for [Mindustry](https://github.com/Anuken/Mindustry) written in Rust. Unlike the original Java server that handles one world per instance and scales only vertically, Multidustry runs multiple worlds simultaneously with true horizontal scaling capabilities.

## 🌟 Key Features

- **Horizontal Scaling**: Scale apiserver, proxy, and gameserver components independently
- **Multi-World Support**: Run dozens of worlds on a single cluster with seamless world migration
- **Zero-Downtime Migration**: Transfer worlds between nodes without disconnecting players
- **Type-Safe Plugin System**: WASM-based plugins with Component Model and WIT interfaces
- **Hybrid Deployment**: Single-process monolith for small servers, distributed mode for clusters
- **JIT Compilation**: Accelerated MLOG execution via cranelift-jit with fuel metering
- **Cloud-Native**: Kubernetes operator for declarative cluster management
- **Production-Ready**: Built-in observability with OpenTelemetry, metrics, and tracing

## 🏗️ Architecture

Multidustry consists of three horizontally-scalable components communicating via QUIC+CBOR:

### Components

**Apiserver** (Control Plane)
- Raft-based consensus for cluster coordination
- World orchestration and scheduling
- Migration coordination with access tokens
- Plugin management and service discovery

**Proxy** (Gateway)
- Translates Mindustry TCP/UDP protocol to internal QUIC+CBOR
- Player routing and session management
- Buffers packets during world migration for zero-downtime
- DDoS protection and rate limiting

**Gameserver** (Simulation)
- Dual-threaded architecture: sync simulation + async networking
- Legion ECS for deterministic game logic
- Non-blocking queues for inter-thread communication
- World state snapshotting every 5 minutes

```mermaid

flowchart TD
	style A color:#000000,fill:#D9D9D9
	subgraph s1["Proxy"]
		n3@{ label: "Rectangle" }
		n2["Proxy plugin"]
		n1
	end
	n1["TCP/UDP original protocol handler"]
	n1 ---> |"kanal async"| n2
	n2 ---> |"kanal async"| n1
	n1
	n2 ---|"kanal async"| n3["QUIC/CBOR internal transport"]
	n3 ---|"kanal async"| n2["Proxy plugins"]
	subgraph s2["Apiserver"]
		n4["Raft based orcestration"]
	end
	subgraph s3["Consul"]
		n16["kv-store"]
		n5["service discovery"]
	end
	s1 ---|"http"| n5
	s2 ---|"http"| n5
	subgraph s4["Gameserver 1"]
		n9["QUIC/CBOR internal transport"]
		n8["Tokio async part"]
		n7["world2"]
		n6["world1"]
	end
	n1 ---|"tcp/udp"| A
	A["Players"] ---|"tcp/udp"| n1
	n1
	n1
	n1
	n8 --- n6
	n6 --- n8
	n8 --- n7
	n7 --- n8
	n9 --- n8
	n8 --- n9
	n3 ---|"QUIC/CBOR"| n9
	s4 ---|"http"| n5
	subgraph s5["Gameserver 2"]
		n13["QUIC/CBOR internal transport"]
		n12["world4"]
		n11["world3"]
		n10["Tokio async part"]
	end
	n10
	n11
	n10 ---|"kanal"| n12
	n12 ---|"kanal"| n10
	n11
	n10
	n13 ---|"kanal async"| n10
	n10 ---|"kanal async"| n13
	n3 ---|"QUIC/CBOR"| n13
	n13 ---|"QUIC/CBOR"| n3
	n9 ---|"QUIC/CBOR"| n3
	subgraph n11["world3"]
			n15["WorldPlugin1"]
		n14["simulation core"]
	end
	n10 ---|"kanal"| n14
	n14 ---|"kanal"| n10
	n14 ---|"wasm call"| n15
	n15 ---|"wasm call"| n14
	n4
	n4["Raft based orcestration<br><br>Оркестрирует миры, не игроков"]
	s5 ---|"http"| n16
	n15
	n16
	s2 --- n16
	s1 ---|"http"| n16
```

## 🔌 Plugin System

Four types of plugins, all WASM-based with WIT interfaces:

**World Plugins** - Attached to specific worlds, provide gameplay mechanics
```
async fn on_player_join(&mut self, player_id: &str) {
    let player = self.storage.get_document("players", player_id).await?;
    // Custom game logic
}
```

**Proxy Plugins** - Run on each proxy for security, whitelists, analytics

**Apiserver Plugins** - Cluster-wide functionality (Discord integration, webhooks)

**Storage Plugins** - Persistent database backends (MongoDB, PostgreSQL, Qdrant)

### Storage Tiers

1. **World-scoped KV**: Serialized and migrated with world
2. **Node-scoped KV**: Local cache, not migrated
3. **Cluster-scoped KV**: Raft-replicated, in-memory, async writes
4. **Persistent Document Store**: External DB via storage plugins, fully async

### Multi-Language Support

- **Rust** - Full SDK with async reactor
- **Python** - via componentize-py (coming soon)
- **JavaScript** - via jco (planned)
- **Lua** - future component model support

### Plugin Package Format

```
# manifest.toml
[package]
name = "my-plugin"
version = "1.0.0"

[multidustry]
min_version = "0.1.0"

[component]
type = "world-plugin"
entrypoint = "plugin.wasm"

[dependencies]
postgresql = "1.0.1"
```

Package as `.zip` with WASM, manifest, assets, and checksums for verification.

## 🚀 Quick Start

### Monolith Mode (Single Server)

```
# Run all components in one process
multidustry --mode monolith \
  --config config.toml \
  --world worlds/serpulo.msav
```

### Distributed Mode (Cluster)

```
# Apiserver (3 replicas for Raft quorum)
multidustry apiserver --peers node1,node2,node3

# Proxy (scale as needed)
multidustry proxy --apiserver http://apiserver:8080

# Gameserver (scale as needed)
multidustry gameserver --apiserver http://apiserver:8080
```

### Kubernetes Deployment

```
apiVersion: neodustry.ru/multidustry/v1
kind: MultidustryCluster
metadata:
  name: production
spec:
  version: "0.1.0"
  apiserver:
    replicas: 3
  proxy:
    replicas: 2
    type: LoadBalancer
  gameserver:
    minReplicas: 2
    maxReplicas: 20
  storage:
    - name: mongodb
      plugin:
        url: "https://github.com/.../mongodb-storage.zip"
```

## 🛠️ Technology Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust 1.75+ |
| Async Runtime | tokio |
| Networking | Quinn (QUIC) |
| Consensus | OpenRaft |
| Serialization | CBOR (ciborium) |
| WASM Runtime | Wasmtime |
| JIT Compiler | cranelift-jit |
| ECS | Legion |
| Observability | OpenTelemetry + tracing |

## 📚 Documentation

- [Architecture Decision Records (ADR)](docs/adr/)
- [Plugin Development Guide](docs/plugins.md)
- [Deployment Guide](docs/deployment.md)
- [API Reference](docs/api/)
- [Русская документация](docs/ru/)

## 🗺️ Roadmap

### v0.1.0 - MVP (Current)
- [x] Core architecture design
- [x] mlogjit compiler
- [ ] Basic world simulation
- [ ] Monolith mode
- [ ] Single storage plugin (MongoDB)

### v0.2.0 - Distributed
- [ ] Raft integration
- [ ] QUIC networking
- [ ] World migration
- [ ] Horizontal scaling

### v0.3.0 - Plugins
- [ ] WASM Component Model
- [ ] Async reactor for plugins
- [ ] Python SDK
- [ ] Plugin registry

### v0.4.0 - Cloud Native
- [ ] Kubernetes operator
- [ ] Helm charts
- [ ] Multi-cloud support
- [ ] Advanced observability

### v1.0.0 - Production Ready
- [ ] Full protocol compatibility
- [ ] Performance optimization
- [ ] Security audit
- [ ] Comprehensive tests

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```
git clone https://github.com/MichaAI/multidustry
cd multidustry
cargo build
cargo test
```

### Project Structure

```
multidustry/
├── apiserver/      - Control plane
├── proxy/          - Protocol gateway
├── gameserver/     - World simulation
├── monolith/       - Single-process mode
├── multidustrycore/ - Shared utilities
├── multidustrycli/ - Command-line interface
├── labs/           - Experimental prototypes
│   ├── quic/       - QUIC experiments
│   └── raft/       - Raft experiments
├── wit/            - WIT interface definitions
└── docs/           - Documentation
    └── adr/        - Architecture decisions
```

## ⚖️ Trade-offs

**Availability vs Consistency**: Multidustry prioritizes consistency and simulation speed over availability. Worlds on failed nodes resume from latest snapshot (5min intervals) rather than exact state.

**Single Datacenter**: Designed for single-region deployment, not geo-distributed clusters.

## 📄 License

Licensed under Apache License 2.0 - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [Mindustry](https://github.com/Anuken/Mindustry) by Anuke - Original game
- [Bytecode Alliance](https://bytecodealliance.org/) - Component Model
- Rust and WASM communities

## 💬 Community

- GitHub Issues: Bug reports and feature requests
- Discussions: Architecture and design discussions
- Discord: [Coming Soon]

---

**Status**: 🚧 Active Development - Not production ready yet

**Author**: MichaAI

**Made with** 🦀 **and** 💙
```

