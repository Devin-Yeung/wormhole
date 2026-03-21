---
title: Bloom Filter Infeasibility
---

# Bloom Filter Infeasibility

## Overview

This note explains why a **traditional Bloom filter**, especially a **full-history Bloom filter**, is not a viable
way to defend Wormhole against cache penetration under the current workload.

In this note:

- a **traditional Bloom filter** means a standard append-only Bloom filter used as a membership pre-check before
  hitting storage
- a **full-history Bloom filter** means a filter that attempts to represent the complete set of valid short URLs

This note focuses only on the feasibility of that design under the current workload. It does **not** compare
alternative mitigation strategies.

## Workload

The current workload model gives us the following write-side numbers:

- peak create throughput: **1,024 QPS**
- planning traffic level for steady operation: **60%** of peak
- sustained create throughput at that level:

$$
1{,}024 \times 0.6 = 614.4 \text{ QPS}
$$

That yields:

$$
614.4 \times 86{,}400 = 53{,}084{,}160 \text{ new URLs per day}
$$

This number matters because a membership filter must grow with the number of valid URLs it is expected to recognize.

For this discussion, the important product assumption is that created short URLs are **long-lived**. A URL created
weeks or months ago may still be requested today. That means a cache-penetration filter cannot safely protect only
"recent" writes unless the product also enforces a strict expiration policy for URLs themselves.

## Sizing

Using the provided design target:

- expected items: **53,084,160**
- target false positive rate: **0.1%**

the standard Bloom filter sizing formula is:

$$
m = -\frac{n \ln p}{(\ln 2)^2}
$$

where:

- $n$ is the number of items
- $p$ is the target false positive rate
- $m$ is the number of bits in the filter

For this workload, that gives:

- bits per item:

$$
\frac{m}{n} = -\frac{\ln p}{(\ln 2)^2} \approx 14.3776 \text{ bits/item}
$$

- bytes per item:

$$
\frac{14.3776}{8} \approx 1.7972 \text{ B/item}
$$

- ideal filter size: **763,222,159 bits**
- rounded filter size: **95,402,770 bytes**
- rounded filter size: **90.98 MiB**

At this false positive target, the optimal hash count is:

$$
k = \frac{m}{n} \ln 2 \approx 9.97 \approx 10
$$

So each protected lookup needs about **10 hash functions**.

The important point is not the exact constant. The important point is that the filter size is proportional to the
number of items. If the valid URL set keeps growing, the filter must keep growing with it to preserve the same false
positive rate.

## Unbounded Growth

At the current steady write load, a full-history Bloom filter grows by about **90.98 MiB per day**.

That sounds manageable if we only look at a single day. It becomes unacceptable once we look at the actual retention
problem:

| Historical URL set represented by the filter | Raw size of one full filter |
|----------------------------------------------|----------------------------:|
| 1 day                                        |                   90.98 MiB |
| 7 days                                       |                  636.88 MiB |
| 30 days                                      |                    2.67 GiB |
| 90 days                                      |                    8.00 GiB |
| 180 days                                     |                   15.99 GiB |
| 365 days                                     |                   32.43 GiB |
| 3 years                                      |                   97.29 GiB |

Those numbers are only the **raw bit-array size**. They do **not** include:

- process overhead
- allocator overhead
- metadata
- operational headroom
- replication for high availability

In practice, the real memory footprint is worse than the table suggests.

If the filter is replicated across three cache nodes for availability, the raw footprint becomes roughly:

- **97.29 GiB** for one year of history
- **291.87 GiB** for three years of history

That is no longer a small optimization. It becomes a permanent infrastructure commitment whose cost grows every day.

## Rotation Does Not Help

One tempting response is to avoid a single giant filter by rotating smaller filters, for example one filter per day or
per week.

That does not actually solve the underlying problem.

## Resetting Breaks Correctness

If we drop old filters to recover memory, we also drop membership information for still-valid URLs.

That means:

- an old but valid short URL will look absent to the Bloom filter
- the request will fall through the cache-penetration guard
- the system will hit storage even though the key is legitimate

For a URL shortener, that is not a minor edge case. Old links are part of the normal product behavior. Unless URLs are
guaranteed to expire after a short fixed window, a rotating filter with aggressive eviction creates exactly the kind of
false negative that this protection layer is supposed to avoid.

## Keeping Rotations Keeps the Cost

The other option is to keep every rotated filter that might still contain valid URLs.

That preserves correctness, but it does **not** solve the scaling problem:

- total memory still grows linearly with total historical URLs
- lookup cost on the protected cache-miss path now grows with the number of retained filters
- operational complexity increases because every protected cache miss may need to probe multiple filters

In other words, segmentation changes the packaging of the cost, not the slope of the cost curve.

## Sharding Does Not Help

Spreading the filter across multiple machines also fails to change the core economics.

There are only two practical outcomes:

- **replicate the full filter everywhere**, which multiplies memory usage by the number of replicas
- **shard the filter**, which keeps total memory linear while adding routing and network coordination to the lookup path

Neither option removes the need to store state proportional to the total number of valid URLs.

Replication is operationally simple but expensive. Sharding saves per-node memory but turns a fast local membership
test into a distributed dependency on the cache-miss path, which weakens one of the main reasons to use a Bloom filter
in the first place.

## Rebuild Cost

The raw memory curve is already enough to reject the design, but the operational story is even worse.

A full-history filter is derived state. If it is lost, corrupted, or intentionally rebuilt, it must be reconstructed
from the source of truth.

At the current write volume:

$$
53{,}084{,}160 \times 365 = 19{,}375{,}718{,}400
$$

So one year of history is about **19.38 billion** URLs.

$$
53{,}084{,}160 \times (3 \times 365) = 58{,}127{,}155{,}200
$$

So three years of history is about **58.13 billion** URLs.

Replaying tens of billions of items into a filter is not a lightweight recovery procedure.

## 0.1% Is Already a Compromise

This analysis already assumes a **0.1% false positive rate**, which is not especially strict.

If we want a lower false positive rate, the memory cost rises further. If we want deletions, a standard Bloom filter is
not enough and a more expensive variant is required. In both directions, the economics get worse, not better.

## Conclusion

Under the current workload, a traditional Bloom filter is not a bounded-cost defense against cache penetration.

At **53,084,160** new URLs per day, even a relatively relaxed **0.1%** false positive target requires about
**90.98 MiB** of new raw filter memory every day. Because valid short URLs accumulate over time, a correct
full-history filter must keep accumulating memory as well.

That leads to a simple conclusion:

- a single full filter becomes too large to justify
- rotating filters either break correctness or preserve the same linear growth
- sharding or replication only redistributes the same cost across more machines

For Wormhole's current workload and URL lifetime assumptions, using a traditional or full-history Bloom filter to
prevent cache penetration is therefore **not a viable design**.
