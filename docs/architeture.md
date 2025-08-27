# 1. Architecture Overview

## Core Idea

This project is a **multi-threaded reactive key-value store** written in Rust.
The goal is to combine **performance** (shard-per-thread model, no locking), **modularity** (protocol adapters like TCP,
HTTP, gRPC), and **reactivity** (subscriptions & streaming updates) in one clean design.

Think of it as:

* **A Redis-like store** but with explicit sharding and job queues.
* **Multi-protocol** from the start (TCP → HTTP → gRPC → WebSockets).
* **Future extensibility** for scripting, reactivity, and custom modules.

---

## High-Level Design

### Shards

* Data is **split across shards**.
* Each shard is **owned by one Tokio worker thread**.
* **No locks needed** → one thread owns its shard’s memory.
* Communication with a shard is done by **pushing jobs to its job queue**.

### Job Queue

* Each shard has a **queue of jobs (commands)**.
* Jobs are **scheduled onto shards** based on the key being accessed (hashing).
* Shard executes jobs **sequentially** (single-threaded execution → no race conditions).

### Protocol Layer

* Clients can connect via **TCP (initial)**.
* Future support: **HTTP, gRPC, WebSockets, Protobufs**.
* Protocol layer **parses incoming requests into commands**, and hands them to the shard job queue.

### Command Execution

* Commands like `GET`, `SET`, `DEL` are **dispatched to shards**.
* Shard executes command and **returns result** via a response channel.

### Reactivity (Future Phase)

* Clients can **subscribe** to keys or patterns.
* Whenever a key is updated, subscribers get **pushed updates**.
* Useful for streaming or cache invalidation.

### Scripting (Future Phase)

* Support running **user-defined scripts** (Lua/JS).
* Scripts can read/write data from inside shard context.
* Ensures scripts are **sandboxed** and don’t break isolation.

---

## ASCII Architecture Diagram

```
         ┌─────────────────────────┐
         │       Client Apps       │
         │ (TCP, HTTP, gRPC, etc.) │
         └───────────┬─────────────┘
                     │
              Protocol Layer
    (parsers, serialization, request routing)
                     │
          ┌──────────┴───────────┐
          │   Job Dispatcher     │
          │ (hash key → shard)   │
          └──────────┬───────────┘
                     │
      ┌──────────────┼──────────────┐
      │              │              │
 ┌────▼─────┐   ┌────▼─────┐   ┌────▼─────┐
 │ Shard 1  │   │ Shard 2  │   │ Shard N  │
 │ (Thread) │   │ (Thread) │   │ (Thread) │
 │ Job Q    │   │ Job Q    │   │ Job Q    │
 │ Data     │   │ Data     │   │ Data     │
 └──────────┘   └──────────┘   └──────────┘
```

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
