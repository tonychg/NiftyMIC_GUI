import os
from pydantic import BaseSettings


class Settings(BaseSettings):
    DEBUG: bool = True
    BASE_DIRECTORY: str = f"{os.environ['HOME']}/NiftyMIC_GUI"
    OUTPUT_DIRECTORY: str = f"{BASE_DIRECTORY}/output"
    LOG_FILE: str = f"{BASE_DIRECTORY}/niftymic_gui.log"
    BINARY_DIR: str = f"{BASE_DIRECTORY}/bin"
    DCM2NIIX_PATH: str = f"{BINARY_DIR}/dcm2niix"
    MEDCON_PATH: str = f"{BINARY_DIR}/medcon"
    NIFTYMIC_DOCKER_IMAGE: str = "renbem/niftymic"


settings = Settings()
