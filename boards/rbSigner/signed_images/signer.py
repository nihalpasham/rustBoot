
import os
import shutil

key_file = "ecc256.der"
firmware = ["nrf52840_bootfw.bin", "nrf52840_updtfw.bin", "stm32f411_bootfw.bin", "stm32f411_updtfw.bin", "stm32f446_bootfw.bin", "stm32f446_updtfw.bin", "stm32h723_bootfw.bin", "stm32h723_updtfw.bin","stm32f746_bootfw","stm32f746_updtfw", "stm32f334_bootfw", "stm32f334_updtfw", "rp2040_bootfw.bin", "rp2040_updtfw.bin"]
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
        # boot image - version 1234
        elif filename == "stm32f411_bootfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1234")
        # update image - version 1235
        elif filename == "stm32f411_updtfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1235")
        # boot image - version 1234             
        elif filename == "stm32f446_bootfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1234")
        # update image - version 1235
        elif filename == "stm32f446_updtfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1235")
        # boot image - version 1234            
        elif filename == "stm32h723_bootfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1234")
        # update image - version 1235
        elif filename == "stm32h723_updtfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1235")
        # boot image - version 1234
        elif filename == "stm32f746_bootfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1234")
        # update image - version 1235
        elif filename == "stm32f746_updtfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1235")
        # boot image - version 1234 
        elif filename == "stm32f334_bootfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1234")
        # update image - version 1235
        elif filename == "stm32f334_updtfw.bin":
            # sign `bin` file
            os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1235")
        # boot image - version 1234 
        elif filename == "rp2040_bootfw.bin":
            # sign `bin` file
            os.system("py sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1234")
        # update image - version 1235
        elif filename == "rp2040_updtfw.bin":
            # sign `bin` file
            os.system("py sign.py --ecc256 --sha256" + " " +
                      filename + " ecc256.der 1235")
def clean_up():
    os.remove(os.getcwd() + "/ecc256.der")

sign_image(target_path)
clean_up()