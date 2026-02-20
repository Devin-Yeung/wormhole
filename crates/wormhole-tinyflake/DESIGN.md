# Tinyflake Capacity Design Note

## 1. Goal

This note quantifies the two core capacity properties of the current Tinyflake layout:

1. How long the system can generate IDs before timestamp exhaustion
2. How many IDs per second it can generate (single node and full cluster)

## 2. Bit Layout and Assumptions

Current type layout:

- `timestamp`: 30 bits (`B30`), unit is seconds since a custom epoch
- `sequence`: 8 bits (`B8`), per-node counter that resets each new second
- `node_id`: 2 bits (`B2`), unique node index

Total width is `30 + 8 + 2 = 40` bits.

For calculations in this document:

- `T = 30` (timestamp bits)
- `S = 8` (sequence bits)
- `N = 2` (node bits)
- Number of representable values for a `k`-bit field is `2^k`

## 3. Core Capacity Calculations

### 3.1 Lifetime (How Long IDs Can Be Generated)

Timestamp uses seconds, so the number of available one-second slots is:

`2^T = 2^30 = 1,073,741,824 seconds`

Equivalent durations:

- `1,073,741,824 / 86,400 = 12,427.5674 days`
- `1,073,741,824 / (86,400 * 365) = 34.0481 years` (365-day years)
- `1,073,741,824 / (86,400 * 365.2425) = 34.0255 years` (solar-year average)

Practical interpretation:

- Lifetime window is about **34.03 years** from the configured custom epoch
- Since timestamp starts at `0`, the maximum timestamp value is `2^30 - 1`
- Valid generation range is `[epoch, epoch + (2^30 - 1) seconds]`

### 3.2 Throughput (IDs Per Second)

Per node, the sequence space per second is:

`2^S = 2^8 = 256 IDs/second/node`

Cluster node count is:

`2^N = 2^2 = 4 nodes`

Cluster-wide max throughput per second is:

`2^S * 2^N = 2^(S+N) = 2^10 = 1,024 IDs/second/cluster`

Equivalent sustained rates:

- Per node per minute: `256 * 60 = 15,360 IDs`
- Per node per day: `256 * 86,400 = 22,118,400 IDs`
- Cluster per minute: `1,024 * 60 = 61,440 IDs`
- Cluster per day: `1,024 * 86,400 = 88,473,600 IDs`

## 4. Derived Total ID Capacity

While runtime and per-second throughput are the two key operational metrics, the complete ID space is also useful for planning:

- Per node over full lifetime: `2^(T+S) = 2^38 = 274,877,906,944 IDs`
- Full 4-node cluster over full lifetime: `2^(T+S+N) = 2^40 = 1,099,511,627,776 IDs`

## 5. Operational Constraints and Edge Conditions

These bounds assume the implementation enforces the following:

1. **Unique `node_id` per active generator**
   - If two active generators share a node ID, uniqueness is no longer guaranteed.

2. **Per-second sequence overflow handling**
   - If a node receives more than 256 ID requests in the same second, it must either:
     - wait for the next second, or
     - return a retry/overflow error.

3. **Clock monotonicity and rollback strategy**
   - If wall clock time moves backward, generation must pause or use a corrective strategy.
   - Without rollback handling, strict monotonic ID ordering cannot be guaranteed.

4. **Custom epoch selection**
   - Epoch does not change throughput, but it directly determines the absolute calendar end date.

## 6. Final Capacity Summary

- **Lifetime:** `2^30 seconds` = `1,073,741,824 seconds` approx **34.03 years**
- **Per-node throughput:** `2^8` = **256 IDs/second**
- **4-node cluster throughput:** `2^(8+2)` = **1,024 IDs/second**

These are the hard capacity limits implied by the current `30/8/2` Tinyflake layout.

