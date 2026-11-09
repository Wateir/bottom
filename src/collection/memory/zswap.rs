//! zswap pool memory collection.
//!
//! [zswap](https://www.kernel.org/doc/html/latest/admin-guide/mm/zswap.html) is
//! a compressed cache for swap pages. When it is compiled in and enabled, the
//! kernel reports the size of the compressed pool in RAM via the `Zswap` field
//! in `/proc/meminfo`, and (on newer kernels) the uncompressed size of the
//! pages it holds via the `Zswapped` field.

use std::num::NonZeroU64;

use super::MemData;

/// zswap pool usage data.
#[derive(Debug, Clone)]
pub struct ZswapData {
    /// The pool's physical footprint in RAM, relative to total memory.
    pub pool: MemData,

    /// The uncompressed size of the pages held in the pool (`Zswapped`), if
    /// the kernel reports it.
    pub zswapped_bytes: Option<u64>,
}

impl ZswapData {
    /// Returns the pool's compression ratio (uncompressed/compressed size),
    /// e.g. `4.2` for a `4.2:1` ratio, if it can be determined.
    pub(crate) fn compression_ratio(&self) -> Option<f64> {
        let zswapped = self.zswapped_bytes?;

        if self.pool.used_bytes == 0 {
            return None; // Avoid dividing by (or reporting) a degenerate ratio.
        }

        Some(zswapped as f64 / self.pool.used_bytes as f64)
    }
}

/// Returns the zswap pool's memory usage, if available.
///
/// Returns [`None`] if the system does not expose zswap stats (e.g. zswap is
/// not compiled in or is not enabled).
pub(crate) fn get_zswap_usage(sys: &sysinfo::System) -> Option<ZswapData> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    let (pool_bytes, zswapped_bytes) = parse_zswap_fields(&meminfo)?;

    NonZeroU64::new(sys.total_memory()).map(|total_bytes| ZswapData {
        pool: MemData {
            total_bytes,
            used_bytes: pool_bytes,
        },
        zswapped_bytes,
    })
}

/// Subtracts the zswap pool from a RAM [`MemData`]'s used and total amounts,
/// treating the pool as reserved memory that is reported separately by the
/// `ZWP` entry.
///
/// This does nothing if the pool is not smaller than the total (which should
/// never happen in practice, but could with e.g. cgroup-limited totals).
pub(crate) fn subtract_pool_from_ram(mem: &mut MemData, pool_bytes: u64) {
    if let Some(total) = NonZeroU64::new(mem.total_bytes.get().saturating_sub(pool_bytes)) {
        mem.total_bytes = total;
        mem.used_bytes = mem.used_bytes.saturating_sub(pool_bytes);
    }
}

/// Parses the zswap pool size and the uncompressed size of the pages it holds,
/// in bytes, out of a `/proc/meminfo` dump.
///
/// Returns [`None`] if there is no `Zswap` entry. `Zswapped` is reported as
/// [`None`] on kernels that don't expose it.
fn parse_zswap_fields(meminfo: &str) -> Option<(u64, Option<u64>)> {
    let mut pool = None;
    let mut zswapped = None;

    for line in meminfo.lines() {
        // The labels must be checked as whole tokens, since `Zswap:` is a
        // prefix of `Zswapped:`.
        let mut fields = line.split_whitespace();

        let target = match fields.next() {
            Some("Zswap:") => &mut pool,
            Some("Zswapped:") => &mut zswapped,
            _ => continue,
        };

        if let Some(kilobytes) = fields.next().and_then(|value| value.parse::<u64>().ok()) {
            *target = Some(kilobytes.saturating_mul(1024));
        }
    }

    pool.map(|pool| (pool, zswapped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_zswap_entries() {
        let meminfo = "\
MemTotal:       16294384 kB
MemFree:         4194304 kB
Zswap:            40960 kB
Zswapped:        122880 kB
";

        assert_eq!(
            parse_zswap_fields(meminfo),
            Some((40960 * 1024, Some(122880 * 1024)))
        );
    }

    #[test]
    fn parses_pool_without_zswapped() {
        // Kernels older than 6.8 don't report `Zswapped`.
        assert_eq!(
            parse_zswap_fields("Zswap: 40960 kB\n"),
            Some((40960 * 1024, None))
        );
    }

    #[test]
    fn parses_empty_zswap_pool() {
        // zswap is enabled but nothing is in the pool yet.
        assert_eq!(
            parse_zswap_fields("Zswap: 0 kB\nZswapped: 0 kB\n"),
            Some((0, Some(0)))
        );
    }

    #[test]
    fn does_not_confuse_zswapped_for_zswap() {
        assert_eq!(parse_zswap_fields("Zswapped: 999999 kB\n"), None);
    }

    #[test]
    fn missing_zswap_entry_returns_none() {
        let meminfo = "\
MemTotal:       16294384 kB
MemFree:         4194304 kB
SwapTotal:       4194304 kB
";

        assert_eq!(parse_zswap_fields(meminfo), None);
    }

    fn zswap_data(used: u64, zswapped: Option<u64>) -> ZswapData {
        ZswapData {
            pool: MemData {
                used_bytes: used,
                total_bytes: NonZeroU64::new(16 * 1024).expect("total must be non-zero"),
            },
            zswapped_bytes: zswapped,
        }
    }

    #[test]
    fn compression_ratio_is_uncompressed_over_pool() {
        let data = zswap_data(1024, Some(4096));
        assert_eq!(data.compression_ratio(), Some(4.0));
    }

    #[test]
    fn compression_ratio_missing_without_zswapped() {
        assert_eq!(zswap_data(1024, None).compression_ratio(), None);
    }

    #[test]
    fn compression_ratio_undefined_for_empty_pool() {
        assert_eq!(zswap_data(0, Some(0)).compression_ratio(), None);
    }

    fn ram(used: u64, total: u64) -> MemData {
        MemData {
            used_bytes: used,
            total_bytes: NonZeroU64::new(total).expect("total must be non-zero"),
        }
    }

    #[test]
    fn subtract_pool_lowers_used_and_total() {
        let mut mem = ram(8 * 1024, 16 * 1024);
        subtract_pool_from_ram(&mut mem, 2 * 1024);

        assert_eq!(mem.used_bytes, 6 * 1024);
        assert_eq!(mem.total_bytes.get(), 14 * 1024);
    }

    #[test]
    fn subtract_pool_saturates_used() {
        // Shouldn't happen in practice, but used must not underflow.
        let mut mem = ram(1024, 16 * 1024);
        subtract_pool_from_ram(&mut mem, 2 * 1024);

        assert_eq!(mem.used_bytes, 0);
        assert_eq!(mem.total_bytes.get(), 14 * 1024);
    }

    #[test]
    fn subtract_pool_noop_when_pool_covers_total() {
        // E.g. a cgroup limit smaller than the system-wide pool; leave as-is.
        let mut mem = ram(1024, 2048);
        subtract_pool_from_ram(&mut mem, 2048);

        assert_eq!(mem.used_bytes, 1024);
        assert_eq!(mem.total_bytes.get(), 2048);
    }

    #[test]
    fn subtract_zero_pool_is_a_noop() {
        let mut mem = ram(8 * 1024, 16 * 1024);
        subtract_pool_from_ram(&mut mem, 0);

        assert_eq!(mem.used_bytes, 8 * 1024);
        assert_eq!(mem.total_bytes.get(), 16 * 1024);
    }
}
