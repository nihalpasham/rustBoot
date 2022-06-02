
import os
import shutil

key_file = "ecc256.der"
firmware = ["nrf52840_bootfw.bin", "nrf52840_updtfw.bin","stm32f411_bootfw.bin","stm32f411_updtfw.bin"]
target_path = "."


def sign_image(path):
    for filename in os.listdir(path):
        if key_file not in os.listdir(os.getcwd()):
            shutil.copy("../keygen/ecc256.der", os.getcwd())
        # boot image - version 1234
        if filename == "nrf52840_bootfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1234")
        # update image - version 1235
        elif filename == "nrf52840_updtfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1235")
        elif filename == "stm32f411_bootfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1234")
        elif filename == "stm32f411_updtfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1235")
        elif filename == "stm32f446_bootfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1234")
        elif filename == "stm32f446_updtfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1235")

def clean_up():
    os.remove(os.getcwd() + "/ecc256.der")


sign_image(target_path)
clean_up()
