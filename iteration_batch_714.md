# Iteration Batch 714 — Distributed Consensus & CAP Research

**Date**: 2026-09-06
**Prior Batch (713)**: EventBus → transport + durable EventStore separation; SEAL event-sourced state; schema registry for cross-session events; no idempotency on projections; no crash recovery

---

## Search Results Summary

### 1. Consensus (Raft/Paxos/BFT)

| Source | Key Finding |
|--------|------------|
| [NexProTools 2026-09-01](https://www.nexprotools.com/blog/distributed-consensus-raft-vs-paxos-vs-pbft-architecture-guide) | Raft vs Paxos vs PBFT architectural comparison; log compaction, membership changes, BFT variants |
| [CodeLucky 2025-08-28](https://codelucky.com/distributed-consensus-raft-paxos/) | CAP theorem + consensus linkage; Fast/Cheap/Byzantine Paxos variants; anti-patterns: "roll your own Paxos" |
| [NoSQLSummer 2026-06-26](https://www.nosqlsummer.org/blog/interview-distributed-systems-engineer-consensus-2026) | AI agents borrowing Paxos/Raft for split-brain coordination in multi-agent runtimes |
| [CodeLit 2026-03-28](https://codelit.io/blog/distributed-systems-consensus) | CAP trade-offs in real systems (CockroachDB, TiKV, etcd); leader election pitfall guide |
| [youngju.dev 2026-04-15](https://www.youngju.dev/blog/culture/2026-04-15-distributed-consensus-paxos-raft-zab-flp-etcd-zookeeper-kraft-bft-crdt-deep-dive-guide-2025.en) | etcd 8GB limit; KRaft replacing ZooKeeper; anti-patterns: 4-node cluster, Raft without BFT |
| [NanoTech Insight 2026-07-13](https://nanotechinsight.com/post/distributed-systems-consensus-raft-paxos-cap) | BFT needs 3f+1 nodes; Raft/Paxos are CFT only; HotStuff as enterprise BFT path |
| [CodeForge 2026-06-25](https://thecodeforge.io/system-design/consensus-paxos-raft/) | Production war stories; minority partition leader steps down cleanly via heartbeat |
| [NanoTech Insight 2026-08-23](https://www.nanotechinsight.com/post/raft-vs-paxos-practical-consensus-guide) | Raft won engineering consensus 2026; EPaxos/Flexible Paxos for geo-distributed |

### 2. CAP Theorem / PACELC

| Source | Key Finding |
|--------|------------|
| [DesignGurus 2026-04-16](https://www.designgurus.io/blog/system-design-interview-basics-cap-vs-pacelc) | PACELC = CAP + latency dimension; real DB mapping (DynamoDB=AP/EL, Spanner=CP/EC) |
| [PracHub 2026-04-26](https://prachub.com/resources/consistency-or-availability-the-cap-theorem-explained-for-senior-engineers) | PACELC as the "price of normal life"; configurable consistency levels at runtime |
| [martinuke0 2026-05-16](https://martinuke0.github.io/posts/2026-05-16-what-the-pacelc-theorem-reveals-about-distributed-consistency/) | Two binary decisions: partition= A vs C, normal= L vs C; CockroachDB mirrors Spanner CP/EC |
| [AppScale 2026-07-12](https://appscale.blog/en/blog/cap-theorem-pacelc-distributed-systems-real-databases-2026) | Quorum reads/writes: R+W>N formula; cross-region strong consistency adds 50-200ms |
| [techinterview.org 2026-04-20](https://www.techinterview.org/post/3233474185/system-design-cap-theorem-deep-dive-consistency-availability-partition-tolerance-pacelc-eventual-consistency-linearizability/) | Per-workload (not per-company) consistency; eventual consistency between services, strong within |
| [VibeEngines 2026-06-13](https://vibeengines.com/handbook/cap-theorem) | CAP is a single forced choice that only matters during partition; PACELC is everyday trade-off |
| [Educative](https://www.educative.io/courses/grokking-system-design-fundamentals/cap-vs-pacelc-theorem-in-distributed-systems) | CAP and PACELC theory vs practice; real systems expose tunable consistency |
| [SystemDesignSpace 2026-08-17](https://system-design.space/en/chapter/pacelc-theorem) | PACELC separates steady-state policy from degradation policy |

### 3. Distributed Systems Patterns 2026

| Source | Key Finding |
|--------|------------|
| [PrecisionAIAcademy 2026-04-13](https://precisionaiacademy.com/blog/distributed-systems-guide) | CAP overused as excuse, underused as design constraint; streaming = glue not afterthought |
| [pdpspectra 2026-05-15](https://pdpspectra.com/blog/distributed-systems-patterns-2026/) | Consensus, replication, sagas as operational discipline patterns |
| [singhajit.com 2026-04-22](https://singhajit.com/distributed-systems/) | Emergent Leader pattern; Consistent Core pattern (3-5 node quorum cluster); Replicated Log; Gossip protocol |
| [thelinuxcode](https://thelinuxcode.com/latest-trends-in-distributed-systems-2026-what-im-building-around-now/) | Distributed SQL + Streaming + Lakehouse convergence; streaming as operational backbone |
| [Gitscrum](https://docs.gitscrum.com/en/best-practices/distributed-systems-architecture-patterns) | CRDTs for eventual consistency; cache-aside + write-through patterns |
| [SystemDesignHandbook 2026-07-07](https://www.systemdesignhandbook.com/guides/raft-consensus-algorithm/) | Raft core: leader election, log replication, state machine, safety; joint consensus for membership |
| [martinuke0 2026-05-13](https://martinuke0.github.io/posts/2026-05-13-the-raft-consensus-algorithm-a-deep-dive-into-distributed-state-machines/) | Persistent state for crash recovery; deterministic testing; AppendEntries consistency check |
| [martinuke0 2026-05-12](https://martinuke0.github.io/posts/2026-05-12-architecting-distributed-consensus-implementing-raft-for-high-availability-in-state-machine-replication/) | SMR requires consensus; storage backend choice; rolling upgrades + disaster recovery |

---

## NEW Defects & Improvements (Batch 714)

### DEFECT D-714.1: No EventStore Replication / Consensus
**Severity**: CRITICAL
**Evidence**: [youngju.dev](https://www.youngju.dev/blog/culture/2026-04-15-distributed-consensus-paxos-raft-zab-flp-etcd-zookeeper-kraft-bft-crdt-deep-dive-guide-2025.en) — "roll your own Paxos" is anti-pattern #1. [CodeLucky](https://codelucky.com/distributed-consensus-raft-paxos/) — CAP forces consistency vs availability choice during partition.
**Gap**: Batch 713 proposed splitting EventBus into transport + durable EventStore. But no replication strategy or consensus protocol is defined for the EventStore. Single-node SQLite EventStore = zero fault tolerance. On crash, all uncommitted events lost.
**Fix**: Adopt Raft (dominant 2026 consensus per [NanoTech 2026-08-23](https://www.nanotechinsight.com/post/raft-vs-paxos-practical-consensus-guide)) for EventStore replication. Minimum 3 nodes for 1-fault tolerance. Use joint consensus for membership changes.

### DEFECT D-714.2: No PACELC Latency-Consistency Trade-off Model
**Severity**: HIGH
**Evidence**: [DesignGurus 2026-04-16](https://www.designgurus.io/blog/system-design-interview-basics-cap-vs-pacelc) — PACELC adds L vs C decision during normal operation. [VibeEngines 2026-06-13](https://vibeengines.com/handbook/cap-theorem) — "PACELC is the everyday trade-off."
**Gap**: NeoTrix has no framework for choosing consistency level per operation. All reads/writes currently assume strongest consistency. This kills latency for cross-domain queries (NT-WORLD → NT-MEMORY → NT-CORE).
**Fix**: Define per-domain consistency profiles: CP/EC for critical state (SEAL pipeline), AP/EL for cross-session event replay. Expose tunable `ConsistencyLevel` enum: `Strong | BoundedStaleness | Eventual`.

### DEFECT D-714.3: No Event Sourcing Idempotency / Deterministic Replay
**Severity**: HIGH
**Evidence**: [martinuke0 2026-05-13](https://martinuke0.github.io/posts/2026-05-13-the-raft-consensus-algorithm-a-deep-dive-into-distributed-state-machines/) — "SMR requires deterministic state machine application." [martinuke0 2026-05-12](https://martinuke0.github.io/posts/2026-05-12-architecting-distributed-consensus-implementing-raft-for-high-availability-in-state-machine-replication/) — persistent state for crash recovery is mandatory.
**Gap**: Batch 713 identified "no idempotency on projections" but didn't address deterministic replay. If SEAL state is event-sourced, replaying events with non-deterministic side effects (LLM calls, network) corrupts state.
**Fix**: Split events into `DeterministicCommand` (pure state transitions) and `SideEffectCommand` (LLM calls, network). Deterministic commands replay safely. Side-effect commands use idempotency keys + dedup cache.

### DEFECT D-714.4: No Consistent Core for Quorum Operations
**Severity**: MEDIUM
**Evidence**: [singhajit.com 2026-07-03](https://singhajit.com/distributed-systems/) — "Consistent Core pattern: small 3-5 node cluster for quorum throughput; quorum throughput drops as clusters grow."
**Gap**: NeoTrix's KB is SQLite (single node). No quorum-based reads/writes. Cross-domain consistency (e.g., NT-CORE reading NT-MEMORY state) has no ordering guarantee.
**Fix**: Implement Consistent Core: 3-5 node Raft cluster for KB EventStore. Reads go through leader (linearizable) or follower with bounded staleness. Writes always through leader.

### DEFECT D-714.5: No Joint Consensus for Membership Changes
**Severity**: MEDIUM
**Evidence**: [SystemDesignHandbook 2026-07-07](https://www.systemdesignhandbook.com/guides/raft-consensus-algorithm/) — "Joint consensus avoids loss of quorum during reconfiguration." [youngju.dev](https://www.youngju.dev/blog/culture/2026-04-15-distributed-consensus-paxos-raft-zab-flp-etcd-zookeeper-kraft-bft-crdt-deep-dive-guide-2025.en) — anti-pattern: 4-node cluster (even number = quorum ambiguity).
**Gap**: No mechanism to add/remove EventStore nodes without downtime. No protection against split-brain during reconfiguration.
**Fix**: Implement joint consensus protocol for EventStore membership changes. Auto-reject even-numbered cluster configurations.

### DEFECT D-714.6: No Crash Recovery for Event Projections
**Severity**: HIGH
**Evidence**: [martinuke0 2026-05-12](https://martinuke0.github.io/posts/2026-05-12-architecting-distributed-consensus-implementing-raft-for-high-availability-in-state-machine-replication/) — "disaster recovery: node placement, rolling upgrades." Batch 713: "no crash recovery."
**Gap**: Projections (materialized views of event streams) are computed in-memory. On crash, all projections lost. Must replay entire event store to rebuild — O(n) cost.
**Fix**: Persist projections with snapshot + incremental log. Snapshot every N events. On recovery: load snapshot + replay tail. Add `ProjectionCheckpoint` marker in EventStore.

### DEFECT D-714.7: No CRDT Support for Cross-Session Event Metadata
**Severity**: MEDIUM
**Evidence**: [Gitscrum](https://docs.gitscrum.com/en/best-practices/distributed-systems-architecture-patterns) — "CRDTs: mathematically guaranteed convergence." [youngju.dev](https://www.youngju.dev/blog/culture/2026-04-15-distributed-consensus-paxos-raft-zab-flp-etcd-zookeeper-kraft-bft-crdt-deep-dive-guide-2025.en) — CRDTs converge without consensus.
**Gap**: Cross-session event metadata (schema versions, skill versions, KB namespace state) needs eventual convergence across sessions without requiring full consensus round-trip.
**Fix**: Use CRDTs (LWW-register for scalar metadata, OR-set for skill registrations) for session-local caches that merge on reconnect. Consensus only for critical state.

### DEFECT D-714.8: No Gossip Protocol for Event Dissemination
**Severity**: MEDIUM
**Evidence**: [singhajit.com 2026-02-10](https://singhajit.com/distributed-systems/) — "Gossip protocol: dissemination across Cassandra, Consul, DynamoDB." [CodeLit 2026-03-28](https://codelit.io/blog/distributed-systems-consensus) — quorum-based replication requires gossip for failure detection.
**Gap**: Events in the new EventStore have no dissemination mechanism to non-leader nodes. Followers must poll leader (wasteful) or events are lost on leader failover.
**Fix**: Implement gossip protocol for event dissemination. Leader pushes events to followers via gossip. Followers detect leader failure via gossip heartbeat (no separate failure detector needed).

### IMPROVEMENT I-714.1: Runtime Consistency Level Toggle
**Source**: [PracHub 2026-04-26](https://prachub.com/resources/consistency-or-availability-the-cap-theorem-explained-for-senior-engineers) — "many databases expose configurable consistency levels, allowing runtime shift."
**Action**: Add `ConsistencyLevel` parameter to all KB query/mutation APIs. Default: `BoundedStaleness(max_lag=1)`. Critical paths (SEAL pipeline commits): `Strong`. Cross-session reads: `Eventual`.

### IMPROVEMENT I-714.2: Event Store Log Compaction / Snapshotting
**Source**: [CodeLucky](https://codelucky.com/distributed-consensus-raft-paxos/) — "log compaction: snapshotting, log truncation, incremental snapshots." [youngju.dev](https://www.youngju.dev/blog/culture/2026-04-15-distributed-consensus-paxos-raft-zab-flp-etcd-zookeeper-kraft-bft-crdt-deep-dive-guide-2025.en) — "etcd 8GB limit — one day, sudden outage."
**Action**: Implement periodic snapshotting of EventStore. Compact old events into snapshots. Prevent unbounded log growth (etcd anti-pattern).

### IMPROVEMENT I-714.3: AppendEntries Consistency Check for Event Ordering
**Source**: [martinuke0 2026-05-13](https://martinuke0.github.io/posts/2026-05-13-the-raft-consensus-algorithm-a-deep-dive-into-distributed-state-machines/) — "AppendEntries consistency check ensures Log Matching invariant."
**Action**: Each event must carry `prev_event_id` for causal ordering. Reject events with mismatched `prev_event_id` (log matching property). This prevents out-of-order event application.

---

## Sources Cited (24 unique)

1. https://www.nexprotools.com/blog/distributed-consensus-raft-vs-paxos-vs-pbft-architecture-guide
2. https://codelucky.com/distributed-consensus-raft-paxos/
3. https://www.nosqlsummer.org/blog/interview-distributed-systems-engineer-consensus-2026
4. https://codelit.io/blog/distributed-systems-consensus
5. https://www.youngju.dev/blog/culture/2026-04-15-distributed-consensus-paxos-raft-zab-flp-etcd-zookeeper-kraft-bft-crdt-deep-dive-guide-2025.en
6. https://nanotechinsight.com/post/distributed-systems-consensus-raft-paxos-cap
7. https://thecodeforge.io/system-design/consensus-paxos-raft/
8. https://www.nanotechinsight.com/post/raft-vs-paxos-practical-consensus-guide
9. https://www.designgurus.io/blog/system-design-interview-basics-cap-vs-pacelc
10. https://prachub.com/resources/consistency-or-availability-the-cap-theorem-explained-for-senior-engineers
11. https://martinuke0.github.io/posts/2026-05-16-what-the-pacelc-theorem-reveals-about-distributed-consistency/
12. https://appscale.blog/en/blog/cap-theorem-pacelc-distributed-systems-real-databases-2026
13. https://www.techinterview.org/post/3233474185/system-design-cap-theorem-deep-dive-consistency-availability-partition-tolerance-pacelc-eventual-consistency-linearizability/
14. https://vibeengines.com/handbook/cap-theorem
15. https://www.educative.io/courses/grokking-system-design-fundamentals/cap-vs-pacelc-theorem-in-distributed-systems
16. https://system-design.space/en/chapter/pacelc-theorem
17. https://precisionaiacademy.com/blog/distributed-systems-guide
18. https://pdpspectra.com/blog/distributed-systems-patterns-2026/
19. https://singhajit.com/distributed-systems/
20. https://thelinuxcode.com/latest-trends-in-distributed-systems-2026-what-im-building-around-now/
21. https://docs.gitscrum.com/en/best-practices/distributed-systems-architecture-patterns
22. https://www.systemdesignhandbook.com/guides/raft-consensus-algorithm/
23. https://martinuke0.github.io/posts/2026-05-13-the-raft-consensus-algorithm-a-deep-dive-into-distributed-state-machines/
24. https://martinuke0.github.io/posts/2026-05-12-architecting-distributed-consensus-implementing-raft-for-high-availability-in-state-machine-replication/

---

## What's NEW vs Batch 713

| Batch 713 | Batch 714 Addition |
|-----------|-------------------|
| EventBus → transport + EventStore | **Which consensus protocol?** Raft (D-714.1), quorum sizing (D-714.4), joint consensus (D-714.5) |
| SEAL event-sourced state | **Deterministic vs side-effect split** (D-714.3), projection crash recovery (D-714.6) |
| Schema registry for cross-session events | **CRDTs for metadata convergence** (D-714.7), gossip dissemination (D-714.8) |
| No idempotency on projections | **PACELC trade-off model** (D-714.2), runtime consistency toggle (I-714.1) |
| No crash recovery | **Snapshot + checkpoint protocol** (I-714.2), AppendEntries consistency check (I-714.3) |
