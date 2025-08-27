# Roadmap & Task List

We will build this in **phases**. Each phase produces a working system that contributors can test, benchmark, and
extend.

---

## **Phase 1: Minimal TCP Key-Value Store**

> 🎯 Goal: Have a working KV store over TCP with `GET` / `SET` / `DEL`.

**Tasks:**

* [ ] Setup project structure (workspace: `server/`, `protocols/`, `core/`).
* [ ] Implement TCP server (Tokio).
* [ ] Add basic RESP (Redis) command parser.
* [ ] Implement `Command` enum (`Get`, `Set`, `Del`, …).
* [ ] Add `Shard` struct with:

    * [ ] Job queue.
    * [ ] Simple `HashMap<String, Vec<u8>>` store.
    * [ ] Event loop (Tokio task).
* [ ] Dispatcher to route key → shard.
* [ ] Wire everything:

    * [ ] TCP request → parse command → send to shard → respond back.

✅ **Deliverable:**

* Run `cargo run`, connect with `nc`, and do:

  ```
  SET foo bar
  GET foo
  DEL foo
  ```

---

## **Phase 2: Multi-Shard & Multi-Thread**

> 🎯 Goal: Scale across multiple threads with shard isolation.

**Tasks:**

* [ ] Shard manager (spawn N shards, based on CPU count).
* [ ] Key hashing → select shard.
* [ ] Job queue per shard with channels.
* [ ] Thread affinity for shards (Tokio worker pinning if possible).
* [ ] Benchmarks: measure throughput with `redis-benchmark`.

✅ **Deliverable:**

* Store runs with **N shards** in parallel.
* Contributors can run benchmarks.

---

## **Phase 3: Protocol Abstractions**

> 🎯 Goal: Make protocol layer modular so we can add more.

**Tasks:**

* [ ] Define `Protocol` trait (parse bytes → Command, serialize Response → bytes).
* [ ] Refactor TCP to use protocol abstraction.
* [ ] Add HTTP REST adapter (`/get`, `/set`, `/del`).
* [ ] Add gRPC proto definitions + service.

✅ **Deliverable:**

* Multiple clients (TCP, HTTP, gRPC) can interact with the store.

---

## **Phase 4: Reactivity**

> 🎯 Goal: Subscriptions & push updates.

**Tasks:**

* [ ] Add `SUBSCRIBE key` command.
* [ ] Store subscriber list per shard.
* [ ] On key update, notify subscribers.
* [ ] Support streaming protocols (WebSocket, gRPC streaming).

✅ **Deliverable:**

* Clients can subscribe → server pushes updates when values change.

---

## **Phase 5: Scripting Support**

> 🎯 Goal: Extend store with embedded scripting.

**Tasks:**

* [ ] Sandbox Lua or JS runtime.
* [ ] Allow scripts to run inside shard context.
* [ ] Provide limited APIs: `kv.get`, `kv.set`, etc.
* [ ] Script execution is **queued as a job** (ensures isolation).

✅ **Deliverable:**

* Users can upload a script and run `EVAL`-style commands.

---

## **Phase 6: Production Hardening**

> 🎯 Goal: Make it robust for real-world use.

**Tasks:**

* [ ] Add persistence (WAL → recovery).
* [ ] Add replication support.
* [ ] Add clustering (multi-node sharding).
* [ ] Security: auth & TLS.
* [ ] Observability: metrics, logging, tracing.

✅ **Deliverable:**

* A production-ready reactive KV store.

---

# Final Notes for Contributors

* **Keep modules small** (protocols, shards, dispatcher).
* **Tests are critical** (unit + integration).
* **Benchmarks welcomed** (criterion, redis-benchmark).
* **Documentation is part of contribution** (update ROADMAP.md, explain code).

