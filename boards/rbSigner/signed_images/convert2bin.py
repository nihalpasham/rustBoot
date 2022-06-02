import os

firmware = ["stm32f411_bootfw", "stm32f411_updtfw","boot_fw_blinky_green","update_fw_blinky_red"]
target_path = "../../target/thumbv7em-none-eabihf/release/"


def convert_to_bin(path):
    for filename in os.listdir(path):
        if filename == "nrf52840_bootfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary nrf52840_bootfw.bin")
        elif filename == "nrf52840_updtfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary nrf52840_updtfw.bin")
        elif filename == "update_fw_blinky_red" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f411_updtfw.bin")
        elif filename == "boot_fw_blinky_green" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f411_bootfw.bin")
        elif filename == "stm32f446_boot_fw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f446_bootfw.bin")
        elif filename == "stm32f446_updt_fw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f446_updtfw.bin")
        
convert_to_bin(target_path)