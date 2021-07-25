/*   */

use super::image::*;
/// Using the sealed-trait pattern to seal or limit possible `states` and `valid partitions`
/// to the types included in this module.
pub trait Sealed {}

impl<'a, Part: ValidPart + Swappable, State: TypeState> Sealed for RustbootImage<'a, Part, State> {}
impl Sealed for NoState {}
impl Sealed for StateNew {}
impl Sealed for StateSuccess {}
impl Sealed for StateTesting {}
impl Sealed for StateUpdating {}
impl Sealed for Boot {}
impl Sealed for Swap {}
impl Sealed for Update {}
