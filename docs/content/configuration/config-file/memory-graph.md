# Memory Graph Configuration

If you want to change some of the default behaviour of the memory graph widget, you can configure things under the `[memory_graph]` (or `[memory]`) section.

## Graph legend position

You can change where the legend for the graph is placed within the widget itself (or hidden with `"none"`). The default is `"top-right"`.

```toml
[memory_graph]
# One of ["none", "top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right"]
legend_position = "top-left"
```

## Collect/show cache memory

On Linux, you can change whether the memory used by [cache/slabs](https://serverfault.com/a/1025189) is collected and
shown. By default, it is `false`.

<!--TODO: Note this affects basic too... should I separate out a "general memory" setting and specific graph ones?-->

```toml
[memory_graph]
cache_memory = true
```

## Collect/show zswap pool memory

On Linux, you can show how much memory the [zswap](https://www.kernel.org/doc/html/latest/admin-guide/mm/zswap.html)
compressed pool is using, as a percentage of total RAM, in absolute terms, and its compression ratio. This is shown as
a `ZWP` entry in the memory widget. By default, it is `false`; if it is enabled but the system does not expose zswap
stats, nothing is shown.

```toml
[memory_graph]
enable_zswap = true
```

## Subtract the zswap pool from RAM

On Linux, when zswap reporting is enabled, you can additionally subtract the zswap pool from the total and used RAM
shown in the memory widget. The RAM entry then reflects the memory you have available after zswap reserves its
compressed pool, and the pool itself is shown separately as the `ZWP` entry. Enabling this implies
`memory_graph.enable_zswap`. Disabled by default.

```toml
[memory_graph]
free_zswap = true
```

## Short GPU names

If GPU support is enabled, you can use `short_gpu_names` to display simplified GPU names in the memory graph widget
instead of the full GPU names. If enabled, then if there's only one GPU, it'll just list the entry as `GPU`;
if there are multiple GPUs, it'll use entries like `GPU0`, `GPU1`, etc. By default, this is disabled.

```toml
[memory_graph]
short_gpu_names = true
```

## Subtract free-able ARC from memory

If ZFS is detected (note that the `zfs` feature must be enabled if built manually), you can enable `memory_graph.free_arc` to
not count [ARC](https://www.45drives.com/community/articles/zfs-caching/) memory in the RAM usage calculations. Disabled
by default.

```toml
[memory_graph]
free_arc = true
```
