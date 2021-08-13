pub mod update_flash;

use crate::image::image::*;
use crate::Result;
pub trait UpdateInterface: FlashApi {
    fn rustboot_start(self) -> !;
    fn update_trigger(self) -> Result<()>;
    fn update_success(self) -> Result<()>;
}

pub trait FlashApi: Copy {
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
