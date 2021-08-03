pub mod update_flash;

use crate::image::image::*;
use crate::Result;
pub(crate) trait UpdateInterface {
    fn copy_sector<SrcPart: ValidPart, DstPart: ValidPart>(
        self,
        src_part: &PartDescriptor<SrcPart>,
        dst_part: &PartDescriptor<DstPart>,
        sector: usize,
    ) -> Result<usize>;
    fn rustboot_update<'a>(self, rollback: bool) -> Result<RustbootImage<'a, Boot, StateTesting>>;
    fn rustboot_start(self) -> !;
    fn update_trigger(self);
    fn update_success(self);
}

pub(crate) trait FlashApi: Copy {
    fn flash_trailer_write<Part: ValidPart>(
        self,
        part: &PartDescriptor<Part>,
        offset: usize,
        data: *const u8,
        len: usize,
    );
    fn flash_write<Part: ValidPart>(
        self,
        part: &PartDescriptor<Part>,
        offset: usize,
        data: *const u8,
        len: usize,
    );
    fn flash_erase<Part: ValidPart>(self, part: &PartDescriptor<Part>, offset: usize, len: usize);
    fn flash_init();
    fn flash_lock();
    fn flash_unlock();
}
