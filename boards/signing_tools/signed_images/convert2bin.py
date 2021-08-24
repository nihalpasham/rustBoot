import os

firmware = ["nrf52840_bootfw", "nrf52840_updtfw"]
target_path = "../../target/thumbv7em-none-eabihf/release/"


def convert_to_bin(path):
    for filename in os.listdir(path):
        if filename == "nrf52840_bootfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary nrf52840_bootfw.bin")
        elif filename == "nrf52840_updtfw" and (filename + ".bin") not in os.listdir(os.getcwd()):
            os.system("rust-objcopy -I elf32-littlearm" + " " + target_path +
                      filename + " " + "-O binary nrf52840_updtfw.bin")

convert_to_bin(target_path)