#![allow(dead_code)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing,
         clippy::integer_division, clippy::let_unit_value, clippy::manual_rotate,
         clippy::needless_range_loop, clippy::needless_return,
         clippy::drop_non_drop, clippy::manual_unwrap_or_default,
         clippy::legacy_numeric_constants, clippy::len_without_is_empty,
         clippy::result_unit_err, clippy::module_inception,
         clippy::doc_lazy_continuation,
         mismatched_lifetime_syntaxes)]

pub mod blockdevice;
pub mod controller;
mod fat;
pub mod filesystem;
mod structure;
