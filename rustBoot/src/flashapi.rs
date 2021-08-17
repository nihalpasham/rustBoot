
use crate::image::image::{ValidPart, PartDescriptor};
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