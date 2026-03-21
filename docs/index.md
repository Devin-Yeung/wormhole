---
title: Wormhole
---

Wormhole is a high-performance URL shortener written in Rust, built around
separate shortener, redirector, cache, and storage components.

This documentation covers the system design and the key implementation
decisions behind those services.

Current system design notes:

- [Workload Model](system-design/workload.md#overview)
- [Tinyflake Throughput Limit](system-design/tinyflake-throughput.md#overview)
- [Why a Traditional Bloom Filter Is Not Viable](system-design/bloom-filter-infeasibility.md#overview)
