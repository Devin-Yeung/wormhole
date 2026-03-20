---
title: Tinyflake Throughput
---

# Tinyflake Throughput Design Note

## 1. Goal

This note explains why the write path for system-generated short codes is intentionally capped at **1,024 QPS**.

The important point is that this ceiling comes from the **ID generator design**, not from MySQL, Redis, or any other
storage component. We chose this limit to preserve a product constraint:

- generated short codes must stay at **7 Base58 characters or fewer**

This note intentionally focuses on the generated-code path. User-provided custom aliases follow a different product
path and are outside this Tinyflake sizing discussion.

## 2. Start From the Product Constraint

The short code is produced by Base58-encoding the raw bytes of `TinyId`.

That product requirement gives us a hard budget on how large the generated ID can be:

- `58^7 = 2,207,984,167,552` possible 7-character Base58 strings
- this is a little more than `2^41`

If we only looked at pure integer capacity, it would be tempting to say that **41 bits** should fit under a 7-character
Base58 limit.

That is not the constraint we actually implement.

## 3. Why the Safe Budget Is 40 Bits, Not 41 Bits

Our implementation Base58-encodes the packed bytes of `TinyId` directly. In other words, the practical limit is shaped
by **byte width**, not only by abstract integer math.

This matters because:

- `40` bits pack cleanly into **5 bytes**
- `41` bits require **6 bytes**
- Base58 preserves leading zero bytes instead of silently discarding them

Once we cross from 5 bytes to 6 bytes, we can no longer guarantee that every generated value will remain within
7 Base58 characters. A 6-byte payload can spill into an 8-character string even when the underlying numeric value still
looks close to the 7-character range.

So the design constraint is:

- to guarantee `short_code.len() <= 7` for every generated code, we must keep the packed ID at **40 bits**

That is why `TinyId` is modeled as a 40-bit global identifier.

## 4. The 40-Bit Budget Must Be Split Between Lifetime and Throughput

Once the global ID width is fixed at 40 bits, every bit spent on one capacity dimension is unavailable to the others.

For Tinyflake, the budget is split into:

- timestamp bits: how long the system can keep issuing IDs before the epoch window is exhausted
- sequence bits: how many IDs one node can issue within the same second
- node bits: how many generators can issue IDs in parallel without collisions

The current layout is:

| Field     | Bits | Meaning                                                        |
|-----------|-----:|----------------------------------------------------------------|
| Timestamp |   30 | Whole seconds since a custom epoch                             |
| Sequence  |    8 | Per-node counter within the same second                        |
| Node ID   |    2 | Distinguishes up to 4 generators                               |
| Total     |   40 | Fits in 5 bytes, which keeps Base58 output within 7 characters |

That 40-bit layout gives a global ID space of `2^40 = 1,099,511,627,776` unique values across the full lifetime of
the system.

## 5. Why We Reserve 30 Bits for Time

The timestamp in `Tinyflake` uses whole-second precision.

With `30` timestamp bits, the system lifetime is:

- `2^30 = 1,073,741,824` seconds
- about `34.03` years

This is the longevity target we chose for the current design. It is long enough for a production system without forcing
generated short codes to exceed 7 characters.

If we had reserved fewer timestamp bits, we could have increased write throughput, but only by shortening the usable
life of the ID space.

## 6. What Remains for Concurrent Issuance

After reserving `30` bits for time, only `10` bits remain for concurrent issuance:

- `40 - 30 = 10`

Those `10` bits are split into:

- `8` sequence bits
- `2` node bits

That produces the following hard limits:

- per node: `2^8 = 256` IDs per second
- cluster-wide across 4 nodes: `2^(8 + 2) = 2^10 = 1,024` IDs per second

This is where the write ceiling comes from.

## 7. Why 1,024 QPS Is a Generator Limit, Not a Database Limit

The generated short code must exist before the record can be written to storage.

That means the generator sits **in front of** the database on the write path:

1. Tinyflake issues a new global ID
2. the ID is Base58-encoded into a short code
3. the short code is persisted

Because of that ordering, the allocator defines the architectural ceiling for generated writes.

Even if the database could absorb more than `1,024` inserts per second, the current Tinyflake layout still cannot issue
more than `1,024` unique generated IDs per second across the cluster while keeping all of these properties true at the
same time:

- short code length stays at `<= 7`
- system lifetime stays around `34` years
- up to `4` generator nodes can run concurrently

So when we say the create path is sized for **1,024 QPS**, we are documenting a **design tradeoff**, not a measured
database bottleneck.

## 8. Runtime Behavior at the Limit

This limit is also reflected in the implementation behavior.

When one generator exhausts its `256` sequence values for the current second, `Tinyflake::next_id` waits until the next
second boundary before issuing more IDs from that node. The generator does not mint extra IDs by overflowing the
sequence field, because doing so would break uniqueness.

Operationally, that means load above the bit-budgeted limit must show up as one of the following:

- queueing
- backpressure
- retries
- a redesign of the ID layout

It cannot be solved purely by scaling the database tier.

## 9. Tradeoff Table

The design is easier to understand when compared with nearby 40-bit alternatives:

| Layout (`timestamp/sequence/node`) | Approx lifetime | Cluster write ceiling | Consequence                                  |
|------------------------------------|----------------:|----------------------:|----------------------------------------------|
| `31 / 7 / 2`                       |       ~68 years |             `512 QPS` | Better longevity, worse write throughput     |
| `30 / 8 / 2`                       |       ~34 years |           `1,024 QPS` | Current choice                               |
| `29 / 9 / 2`                       |       ~17 years |           `2,048 QPS` | Better write throughput, shorter system life |

This is the core tradeoff:

- keeping short codes at `<= 7` characters fixes the total ID budget at `40` bits
- once `40` bits are fixed, lifetime and write concurrency must compete for the same space
- we chose **~34 years of lifetime** and therefore accepted **1,024 QPS** as the cluster-wide generated-write ceiling

## 10. Final Conclusion

The `1,024 QPS` limit is not an incidental implementation detail. It is the direct result of a deliberate chain of
design decisions:

1. generated short codes must be `<= 7` Base58 characters
2. that forces the packed global ID to stay within **40 bits**
3. we spend `30` of those bits on a `~34` year lifetime
4. only `10` bits remain for concurrent issuance
5. `10` issuance bits yield **1,024 generated IDs per second** across a 4-node cluster

If we want materially higher generated-write throughput in the future, at least one of the following must change:

- allow longer generated short codes
- reduce the supported lifetime of the ID space
- redesign the generator and encoding strategy
