MEMORY
{
  /* FLASH and RAM are mandatory memory regions */

  /* STM32H742xI/743xI/753xI       */
  /* STM32H745xI/747xI/755xI/757xI */
  /* STM32H7A3xI/7B3xI             */
  FLASH  : ORIGIN = 0x08000000, LENGTH = 128K

  /* STM32H742xG/743xG       */
  /* STM32H745xG/STM32H747xG */
  /* STM32H7A3xG             */
  /* FLASH  : ORIGIN = 0x08000000, LENGTH = 512K */
  /* FLASH1 : ORIGIN = 0x08100000, LENGTH = 512K */

  /* STM32H750xB   */
  /* STM32H7B0     */
  /* FLASH  : ORIGIN = 0x08000000, LENGTH = 1M */

  /* DTCM  */
  RAM    : ORIGIN = 0x20000000, LENGTH = 128K
}
