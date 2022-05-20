import logging
from pathlib import Path

from rich.logging import RichHandler

from niftymic_gui.settings import settings

if not Path(settings.BASE_DIRECTORY).exists():
    Path(settings.BASE_DIRECTORY).mkdir(parents=True)

logger = logging.getLogger("niftymic_gui")
logger.setLevel(logging.DEBUG)
shell_handler = RichHandler()
shell_handler.setLevel(logging.DEBUG if settings.DEBUG else logging.INFO)
shell_handler.setFormatter(logging.Formatter("%(message)s"))
file_handler = logging.FileHandler(settings.LOG_FILE)
file_handler.setLevel(logging.DEBUG)
file_handler.setFormatter(
    logging.Formatter(
        "%(levelname)s %(asctime)s [%(filename)s:%(funcName)s:%(lineno)d] %(message)s"
    )
)
logger.addHandler(shell_handler)
logger.addHandler(file_handler)
