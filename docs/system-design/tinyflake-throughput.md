---
title: Tinyflake Throughput Limit
---

# Tinyflake Throughput Limit

## Overview

This note explains why the write path for system-generated short codes is intentionally capped at **1,024 QPS**.

This limit comes from the **Tinyflake ID layout**, not from MySQL, Redis, or any other storage component. It is a
deliberate design tradeoff made to preserve a product constraint:

- generated short codes must stay at **7 Base58 characters or fewer**

This note only covers the generated-code path. User-provided custom aliases follow a different product path and are
outside this Tinyflake sizing discussion.

## Product Constraint

Generated short codes are produced by Base58-encoding the raw bytes of `TinyId`.

That requirement gives the generator a hard size budget: $58^7 = 2{,}207{,}984{,}167{,}552 > 2^{41}$.

If we only looked at abstract integer capacity, it would be tempting to say that a 41-bit ID should fit under a
7-character Base58 limit.

The implementation constraint is stricter than that.

## ID Width Constraint

In this system, Base58 encodes the packed bytes of `TinyId` directly. That means the safe limit is shaped by **byte
width**, not just by abstract integer math.

In practice:

- `40` bits pack cleanly into **5 bytes**
- `41` bits require **6 bytes**
- Base58 preserves leading zero bytes instead of silently discarding them

Once the payload grows from 5 bytes to 6 bytes, we can no longer guarantee that every generated value stays within
7 Base58 characters. A 6-byte payload can spill into an 8-character string even when the underlying numeric value still
looks close to the 7-character range.

Because of that, the design constraint is:

- to guarantee `short_code.len() <= 7` for every generated code, the packed global ID must stay within **40 bits**

## Tinyflake Bit Layout

Once the global ID width is fixed at 40 bits, that budget must be shared across system lifetime and concurrent
issuance.

The current Tinyflake layout is:

| Field     | Bits | Meaning                                                        |
|-----------|-----:|----------------------------------------------------------------|
| Timestamp |   30 | Whole seconds since a custom epoch                             |
| Sequence  |    8 | Per-node counter within the same second                        |
| Node ID   |    2 | Distinguishes up to 4 generators                               |
| Total     |   40 | Fits in 5 bytes, which keeps Base58 output within 7 characters |

This layout yields a global ID space of $2^{40} = 1{,}099{,}511{,}627{,}776$ unique values across the full life of
the system.

## Lifetime Budget

The timestamp in `Tinyflake` uses whole-second precision.

With `30` timestamp bits, the system can issue IDs for $2^{30} = 1{,}073{,}741{,}824$ seconds, or
$\frac{2^{30}}{365.25 \times 24 \times 60 \times 60} \approx 34.03$ years.

This is the lifetime target chosen by the current design. It is long enough for a production system without forcing
generated short codes to exceed 7 characters.

If we reserved fewer bits for time, we could increase write throughput, but only by shortening the usable life of the
ID space.

## Throughput Budget

After reserving `30` bits for time, only $40 - 30 = 10$ bits remain for concurrent issuance.

Those `10` bits are split into:

- `8` sequence bits
- `2` node bits

That produces the following hard limits:

- per node: $2^8 = 256$ IDs per second
- cluster-wide across 4 nodes: $2^{8 + 2} = 2^{10} = 1{,}024$ IDs per second

This is where the **1,024 QPS** ceiling comes from.

## Why This Is Not a Database Limit

The generated short code must exist before the record can be written to storage.

That means the generator sits in front of the database on the write path:

1. Tinyflake issues a new global ID
2. the ID is Base58-encoded into a short code
3. the short code is persisted

Because of that ordering, the generator defines the architectural ceiling for generated writes.

Even if the database could absorb more than `1,024` inserts per second, the current Tinyflake layout still cannot issue
more than `1,024` unique generated IDs per second across the cluster while keeping all of these properties true at the
same time:

- short code length stays at `<= 7`
- system lifetime stays around `34` years
- up to `4` generator nodes can run concurrently

So the create-path target of **1,024 QPS** should be read as a **design tradeoff**, not as a measured database
bottleneck.

## Operational Behavior

This limit is also reflected in the implementation behavior.

When a generator exhausts its `256` sequence values for the current second, `Tinyflake::next_id` waits until the next
second boundary before issuing more IDs from that node. It does not overflow the sequence field, because that would
break uniqueness.

Operationally, load above the bit-budgeted limit must show up as one of the following:

- queueing
- backpressure
- retries
- a redesign of the ID layout

It cannot be solved purely by scaling the database tier.

## Design Tradeoff

The tradeoff becomes clearer when compared with nearby 40-bit alternatives:

| Layout (`timestamp/sequence/node`) | Approx lifetime | Cluster write ceiling | Consequence                                  |
|------------------------------------|----------------:|----------------------:|----------------------------------------------|
| `31 / 7 / 2`                       |       ~68 years |             `512 QPS` | Better longevity, worse write throughput     |
| `30 / 8 / 2`                       |       ~34 years |           `1,024 QPS` | Current choice                               |
| `29 / 9 / 2`                       |       ~17 years |           `2,048 QPS` | Better write throughput, shorter system life |

The core tradeoff is:

- keeping generated short codes at `<= 7` characters fixes the total ID budget at `40` bits
- once `40` bits are fixed, lifetime and write concurrency must compete for the same space
- the current design chooses **~34 years of lifetime** and therefore accepts **1,024 QPS** as the cluster-wide
  generated-write ceiling

## Conclusion

The `1,024 QPS` limit is the direct result of a deliberate chain of design decisions:

1. generated short codes must be `<= 7` Base58 characters
2. that forces the packed global ID to stay within **40 bits**
3. `30` of those bits are reserved for a `~34` year lifetime
4. only `10` bits remain for concurrent issuance
5. `10` issuance bits yield **1,024 generated IDs per second** across a 4-node cluster

If we want materially higher generated-write throughput in the future, at least one of the following must change:

- allow longer generated short codes
- reduce the supported lifetime of the ID space
- redesign the generator and encoding strategy
