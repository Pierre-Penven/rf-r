//! # rf-r
//!
//! High-performance RF and satellite link budget calculations.
//!
//! All functions operate on plain `f64` values with the unit encoded in the
//! function and parameter names (`_db`, `_dbw`, `_hz`, `_m`, `_k`, ...).
//! They are `#[inline]`, allocation-free and follow IEEE-754 semantics:
//! converting `0.0` to dB yields `-inf`, negative linear values yield `NaN`.
//! Batch variants operating on slices are provided for hot loops.
//!
//! ```
//! use rf_r::prelude::*;
//!
//! // GEO Ku-band downlink: 12 GHz, 36 000 km slant range
//! let link = Link {
//!     eirp_dbw: 52.0,
//!     path_loss_db: fspl_db(36_000e3, 12e9),
//!     extra_losses_db: 0.5,
//!     g_over_t_db_k: 20.0,
//! };
//! let downlink_cn = link.cn_db(36e6);
//! let total_cn = combine_cn2_db(downlink_cn, 25.0); // with a 25 dB uplink
//! assert!((downlink_cn - 19.38).abs() < 0.01);
//! assert!(total_cn < downlink_cn);
//! ```

pub mod antenna;
pub mod constants;
pub mod db;
pub mod flux;
pub mod fspl;
pub mod geometry;
pub mod link;
pub mod noise;

/// Convenient glob import of the most used items.
pub mod prelude {
    pub use crate::antenna::*;
    pub use crate::constants::*;
    pub use crate::db::*;
    pub use crate::flux::*;
    pub use crate::fspl::*;
    pub use crate::geometry::*;
    pub use crate::link::*;
    pub use crate::noise::*;
}

#[cfg(test)]
pub(crate) mod test_util {
    /// Asserts that `actual` is within `tolerance` of `expected`.
    #[track_caller]
    pub fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "expected {expected}, got {actual} (tolerance {tolerance})"
        );
    }
}
