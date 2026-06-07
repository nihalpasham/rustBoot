#![allow(clippy::unwrap_used, clippy::expect_used,
         clippy::needless_borrow, clippy::redundant_closure,
         clippy::bool_assert_comparison, clippy::nonminimal_bool,
         clippy::indexing_slicing, clippy::redundant_guards,
         clippy::single_match, clippy::extra_unused_lifetimes,
         clippy::redundant_locals, clippy::manual_strip,
         clippy::unnecessary_unwrap, clippy::missing_safety_doc,
         clippy::match_like_matches_macro, clippy::manual_unwrap_or_default,
         clippy::unnecessary_cast, clippy::integer_division,
         clippy::too_many_arguments, clippy::manual_saturating_arithmetic,
         clippy::doc_lazy_continuation)]

mod common;
#[macro_use]
mod fit;
#[cfg_attr(test, macro_use)]
mod internal;
pub mod patch;
mod reader;
mod struct_item;
mod writer;

pub use common::*;
pub use fit::*;
pub use patch::*;
pub use reader::*;
pub use struct_item::*;
pub use writer::*;
