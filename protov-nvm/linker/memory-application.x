/* Application: keep in sync with memory-bootloader.x and src/layout.rs */

MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  BOOT2                             : ORIGIN = 0x10000000, LENGTH = 0x100
  BOOTLOADER_STATE                  : ORIGIN = 0x10006000, LENGTH = 4K

  FLASH                             : ORIGIN = 0x10007000, LENGTH = 800K
  DFU                               : ORIGIN = 0x100CF000, LENGTH = 800K + 4K

  RESERVED                          : ORIGIN = 0x10198000, LENGTH = 156K
  
  CONFIG                            : ORIGIN = 0x101BF000, LENGTH = 256K
  FACTORY                           : ORIGIN = 0x101FF000, LENGTH = 4K

  RAM   : ORIGIN = 0x20000000, LENGTH = 264K
}

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(BOOT2);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(BOOT2);

__bootloader_active_start = ORIGIN(FLASH) - ORIGIN(BOOT2);
__bootloader_active_end = ORIGIN(FLASH) + LENGTH(FLASH) - ORIGIN(BOOT2);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(BOOT2);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(BOOT2);

__reserved_start = ORIGIN(RESERVED) - ORIGIN(BOOT2);
__reserved_end = ORIGIN(RESERVED) + LENGTH(RESERVED) - ORIGIN(BOOT2);

__config_start = ORIGIN(CONFIG) - ORIGIN(BOOT2);
__config_end = ORIGIN(CONFIG) + LENGTH(CONFIG) - ORIGIN(BOOT2);

__factory_start = ORIGIN(FACTORY) - ORIGIN(BOOT2);
__factory_end = ORIGIN(FACTORY) + LENGTH(FACTORY) - ORIGIN(BOOT2);
