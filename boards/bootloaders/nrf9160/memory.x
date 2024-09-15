MEMORY
{
    FLASH                    : ORIGIN = 0x00000000, LENGTH = 188K
    NSC_FLASH                : ORIGIN = 0x0002F000, LENGTH = 4K
    NS_FLASH                 : ORIGIN = 0x00030000, LENGTH = 832K
    
    RAM                      : ORIGIN = 0x20000000, LENGTH = 128K
    NS_RAM                   : ORIGIN = 0x20020000, LENGTH = 128K
}

INCLUDE trustzone_memory.x

