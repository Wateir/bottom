# Memory Widget

The memory widget provides a visual representation of RAM and swap usage over time.

<figure>
    <img src="../../../assets/screenshots/memory.webp" alt="A picture of an expanded memory widget."/>
</figure>

## Features

The legend displays the current usage in terms of percentage and actual usage in binary units (KiB, MiB, GiB, etc.).
If the total RAM or swap available is 0, then it is automatically hidden from the legend and graph.

One can also adjust the displayed time range through either the keyboard or mouse, with a range of 30s to 600s.

This widget can also be configured to display Nvidia and AMD GPU memory usage (`--disable_gpu` on Linux/Windows to disable), cache memory usage (`--enable_cache_memory`), or zswap pool memory usage (`--enable_zswap` on Linux). On Linux, `--free_zswap` additionally subtracts the zswap pool from the total and used RAM, so the RAM entry reflects the memory available after zswap reserves it.

## Key bindings

Note that key bindings are generally case-sensitive.

| Binding   | Action                                  |
| --------- | --------------------------------------- |
| ++plus++  | Zoom in on chart (decrease time range)  |
| ++minus++ | Zoom out on chart (increase time range) |
| ++equal++ | Reset zoom                              |

## Mouse bindings

| Binding      | Action                                                         |
| ------------ | -------------------------------------------------------------- |
| ++"Scroll"++ | Scrolling up or down zooms in or out of the graph respectively |

## How are memory values determined?

### Linux

Memory usage is calculated using the following formula based on values from `/proc/meminfo` (based on [htop's implementation](https://github.com/htop-dev/htop/blob/976c6123f41492aaf613b9d172eef1842fb7b0a3/linux/LinuxProcessList.c#L1584)):

```
MemTotal - MemFree - Buffers - (Cached + SReclaimable - Shmem)
```

You can find more info on `/proc/meminfo` and its fields [here](https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/6/html/deployment_guide/s2-proc-meminfo).

If zswap reporting is enabled (`--enable_zswap`), the `ZWP` entry reports the size of the compressed zswap pool in
RAM (the `Zswap` field in `/proc/meminfo`) as a percentage of total memory, alongside the pool's compression ratio
(e.g. `4.20:1`, the `Zswapped` field divided by the `Zswap` field). The ratio is only shown on kernels that report
`Zswapped` (6.8 and newer). The entry itself is only shown if the system exposes zswap stats.
If `--free_zswap` is set, the pool is instead subtracted from both the used and total RAM amounts, so the RAM
entry describes the memory available after zswap reserves its pool.

### Windows

In Windows, we calculate swap by querying `Get-Counter "\Paging File(*)\% Usage"`. This
is also what some libraries like [psutil](https://github.com/giampaolo/psutil/blob/master/psutil/arch/windows/mem.c) use. However, note there are also a few other valid methods of
representing "swap" in Windows (e.g. using `GetPerformanceInfo`), which all slightly don't
match.
