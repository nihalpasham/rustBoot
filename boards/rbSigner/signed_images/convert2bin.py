import os

firmware = ["nrf52840_bootfw", "nrf52840_updtfw", "stm32f411_bootfw", "stm32f411_updtfw", "stm32f446_bootfw", "stm32f446_updtfw", "stm32h723_bootfw", "stm32h723_updtfw","stm32f746_updtfw","stm32f746_bootfw", "stm32f334_bootfw", "stm32f334_updtfw","rp2040_bootfw", "rp2040_updtfw"]
target_path = "../../target/thumbv7em-none-eabihf/release/"
# target_path = "../../target/thumbv6m-none-eabi/release/"      # Use this path for building RP2040 Board

def convert_to_bin(path):

    for filename in os.listdir(path):
        if filename == "nrf52840_bootfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary nrf52840_bootfw.bin")
        elif filename == "nrf52840_updtfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary nrf52840_updtfw.bin")
        elif filename == "stm32f411_bootfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f411_bootfw.bin")
        elif filename == "stm32f411_updtfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f411_updtfw.bin")
        elif filename == "stm32f446_bootfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f446_bootfw.bin")
        elif filename == "stm32f446_updtfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f446_updtfw.bin")
        elif filename == "stm32h723_bootfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32h723_bootfw.bin")
        elif filename == "stm32h723_updtfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32h723_updtfw.bin")
        elif filename == "stm32f746_bootfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f746_bootfw.bin")
        elif filename == "stm32f746_updtfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f746_updtfw.bin")
        elif filename == "stm32f334_bootfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f334_bootfw.bin")
        elif filename == "stm32f334_updtfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary stm32f334_updtfw.bin")
        elif filename == "rp2040_bootfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy --strip-all -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary rp2040_bootfw.bin")
        elif filename == "rp2040_updtfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy --strip-all -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary rp2040_updtfw.bin")

convert_to_bin(target_path)