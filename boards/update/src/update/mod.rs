pub mod update_flash;

use rustBoot::flashapi::FlashApi;
use rustBoot::Result;

pub trait UpdateInterface: FlashApi {
    fn rustboot_start(self) -> !;
    fn update_trigger(self) -> Result<()>;
    fn update_success(self) -> Result<()>;
}
