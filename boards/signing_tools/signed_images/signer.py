import os
import shutil

key_file = "ecc256.der"
cortex_m_boards = ["nrf52840", "stm32f3"]
firmware = ["blinky_blue", "blinky_red"]
target_path = "../../target/thumbv7em-none-eabihf/release/"

def find_elf(path):
    for filename in os.listdir(path):
        if filename in firmware:
            if filename not in os.listdir(os.getcwd()):
                shutil.copy((path + filename), os.getcwd())
            if key_file not in os.listdir(os.getcwd()):
                shutil.copy("../keygen/ecc256.der", os.getcwd())
            # boot image - version 1234
            if filename == "blinky_blue":
                os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " " + "ecc256.der 1234")
            # update image - version 1235
            elif filename == "blinky_red":
                os.system("python3 sign.py --ecc256 --sha256" + " " +
                      filename + " " + "ecc256.der 1235")

def find_board_bl():
    for filename in os.listdir(target_path):
        if filename in cortex_m_boards:
            if filename not in os.listdir(os.getcwd()):
                shutil.copy((target_path + filename), os.getcwd())

def clean_up():
    for fw in firmware:
        os.remove(os.getcwd() + "/" + fw)
    os.remove(os.getcwd() + "/ecc256.der")


find_elf(target_path)
find_board_bl()
clean_up()
