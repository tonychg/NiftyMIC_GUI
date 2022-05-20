import sys

from PyQt5.QtWidgets import (
    QApplication,
)
from niftymic_gui.gui import QtApp


def main():
    app = QApplication(sys.argv)
    window = QtApp()
    window.show()
    app.exec()
