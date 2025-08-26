# 🦀 Lock-Free Modular Key-Value Store

Welcome to the **Lock-Free Modular Key-Value Store** — a highly concurrent, sharded, and extensible key-value engine
written in Rust.
Think **Redis-like performance** with **Rust safety guarantees** and a clean modular design for future extensibility.

---

## 🔑 Core Principles

* **Lock-free concurrency** → Each shard owns its data; no global locks, no contention.
* **Shard-per-thread model** → Data is partitioned via consistent hashing and processed by independent tasks.
* **Actor-style message passing** → Commands are sent via channels; shards never share memory.
* **Pluggable layers** → Designed for logging, persistence (AOF), and reactive features like Pub/Sub.

---

## ⚡ High-Level Architecture

```
                ┌─────────────────────────────────┐
                │           TCP Server            │
                │   (RESP request/response)       │
                └──────────────┬──────────────────┘
                               │
                               ▼
                    ┌────────────────────┐
                    │       Router       │
                    │  (consistent hash) │
                    └──────────┬─────────┘
                               │
       ┌───────────────────────┼─────────────────────────┐
       │                       │                         │
       ▼                       ▼                         ▼
┌─────────────┐         ┌─────────────┐          ┌─────────────┐
│   Shard 0   │         │   Shard 1   │   ...    │   Shard N   │
│ tokio task  │         │ tokio task  │          │ tokio task  │
│ owns state  │         │ owns state  │          │ owns state  │
│             │         │             │          │             │
│   Data +    │         │   Data +    │          │   Data +    │
│  Command    │         │  Command    │          │  Command    │
│  Queue      │         │  Queue      │          │  Queue      │
└─────┬───────┘         └─────┬───────┘          └─────┬───────┘
      │                       │                         │
      ▼                       ▼                         ▼
  Responses               Responses                 Responses
      │                       │                         │
      └───────────────┬───────┴───────────┬────────────-┘
                      ▼                   ▼
                  ┌───────────────────────────┐
                  │        Router             │
                  │  (Serialize RESP result)  │
                  └──────────────┬────────────┘
                                 ▼
                        Back to TCP Client
```

---

## 🧩 Modular Layers

### 1. **Networking Layer (RESP over TCP)**

* Handles client connections.
* Parses Redis Serialization Protocol (RESP) frames.
* Forwards parsed commands to the router.
* Writes back serialized responses in order.

### 2. **Router**

* Responsible for **consistent hashing** of keys → decides which shard owns the request.
* Forwards commands to shard queues via `tokio::mpsc`.
* Collects responses and ensures correct ordering per connection.

### 3. **Shard (Core Engine Unit)**

* Each shard is a **tokio task** running its own event loop.
* Shard owns:

    * **In-memory data store** (hash map, skiplist, or future pluggable engine).
    * **Command queue** (receiver side of `mpsc`).
* Processes commands **sequentially within the shard** → no locks required.

### 4. **Extensibility Hooks**

* **Logging layer** → Hook every command/response for observability.
* **Persistence layer** → Append-only file (AOF) or snapshotting.
* **Reactivity layer** → Pub/Sub, key watchers, notifications.
* These layers can be modularized via traits and middleware-style composition.

---

## 🎭 Design Patterns

* **Actor Model** → Each shard acts like an actor: owns its state, communicates via messages.
* **Message Passing** → `tokio::mpsc` channels connect the router ↔ shards.
* **Traits for Extensibility** → Define clear interfaces for persistence, logging, replication.
* **Separation of Concerns** → Networking ≠ Routing ≠ Storage ≠ Extensions.

---

## ⚠️ Pitfalls to Avoid

1. **Response ordering**

    * Make sure router preserves per-connection command order, even with async shard replies.

2. **Backpressure**

    * Use bounded `mpsc` channels to avoid unbounded memory growth under load.

3. **Shard imbalance**

    * Consistent hashing works well, but hot keys may overload one shard → consider rebalancing strategies in the
      future.

4. **Extensibility creep**

    * Keep logging/persistence hooks modular — don’t bake them into shard logic directly.

---

## 🚀 Roadmap

* [ ] Core key-value commands (`GET`, `SET`, `DEL`, etc.).
* [ ] Logging layer (structured events).
* [ ] AOF persistence.
* [ ] Snapshotting & replication.

