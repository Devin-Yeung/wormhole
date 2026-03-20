---
title: Workload Model
---

# Workload Model

## Overview

This system is designed to provide URL shortening and HTTP redirection at predictable scale. The primary design goal is
to keep short URLs compact while sustaining stable throughput on both the write path and the redirect path.

For clarity, this document defines the workload as a **write-to-read ratio of 1:10**. In other words, for every URL
creation request, the system is expected to serve ten redirect requests.

## Short Code Generation Requirements

To minimize the length of generated short URLs, the system uses a custom short-code scheme based on a **40-bit
identifier space**.

A dedicated in-house ID generator is used to allocate short codes for new URLs. This generator is part of the core
system design and must support distributed issuance while preserving the compactness of the resulting short URL.

## Capacity Requirements

Under peak load, with all ID generator instances enabled, the system must sustain the following throughput targets:

| Metric                  |     Target |
|-------------------------|-----------:|
| URL creation throughput |  1,024 QPS |
| Redirect throughput     | 10,240 QPS |
| Write-to-read ratio     |       1:10 |

These targets define the baseline capacity that the system must support in steady-state operation.

## Write Path Requirements

The URL creation path must be able to process **1,024 requests per second** under full load.

This includes:

- short-code generation
- request validation
- persistent storage of URL mappings
- any coordination required by the ID generation mechanism

The write path must remain stable when all generators are active and issuing codes concurrently.

## Read Path Requirements

The redirect path must be able to process **10,240 requests per second** under full load.

This includes:

- short-code lookup
- retrieval of the original URL mapping
- generation of the redirect response

Because redirect traffic is expected to dominate the workload, the overall architecture must prioritize read efficiency
and horizontal scalability on the redirect path.

## Design Implications

Based on the workload model above, the system should be treated as a read-heavy service. The write path must provide
reliable and compact short-code generation, while the redirect path must be optimized for significantly higher
throughput.

As a result, system sizing, cache strategy, storage access patterns, and service deployment topology should all be
planned around the requirement to sustain **10,240 redirect requests per second** while continuing to support **1,024
write requests per second**.

## Scope

This section defines throughput and workload assumptions only. It does not yet define:

- latency objectives
- availability targets
- disaster recovery requirements
- multi-region deployment requirements

These concerns should be documented separately in the system design specification.

Add this section to the note:

## Consistency Model

This system is designed for **eventual consistency**, not strong consistency.

In particular, the system does **not** guarantee immediate read-after-write consistency. After a new short URL is
created and persisted, there may be a short propagation window during which a subsequent read or redirect request can
still observe stale data.

This behavior is an intentional design choice. The system prioritizes high throughput and operational simplicity over
strict consistency guarantees on the read path.

The practical implication is that:

- a newly inserted short code may not be immediately visible to every read path component
- redirect requests issued immediately after creation may temporarily return stale or not-yet-updated results
- the system is expected to converge automatically, after which reads will return the latest mapping

This consistency model is acceptable for the current workload and business requirements, where brief stale reads after
insertion are tolerated.
