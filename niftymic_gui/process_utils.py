import os
import subprocess
import time
from pathlib import Path

import psutil

from niftymic_gui.helpers import logger


def execute_cmdline(command_line, cwd: Path = None) -> int:
    logger.info(f"Execute {command_line}")
    process = subprocess.Popen(
        command_line,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env={
            **os.environ,
            "DOCKER_HOST": "unix:///run/user/1000/docker.sock",
        },
    )
    with process.stdout:
        for line in iter(process.stdout.readline, b""):
            logger.info(line.decode("utf8").strip())
    process.wait(timeout=3600)
    return process.returncode


def get_niftymic_process():
    for process in psutil.process_iter(["pid", "name", "cmdline"]):
        cmdline = process.cmdline()

        if "python" in cmdline:
            if (
                "fetal_brain_seg.py" or "/usr/local/bin/niftymic_reconstruct_volume"
            ) in cmdline:
                yield process


def kill_all_process():
    while len(processes := list(get_niftymic_process())) > 0:
        for process in processes:
            logger.info("Kill process %s", process.cmdline())
            process.kill()
            process.terminate()
        time.sleep(0.5)


if __name__ == "__main__":
    kill_all_process()
