import sys

from PyQt5.QtWidgets import (
    QApplication,
)
from niftymic_gui.gui import QtApp


if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = QtApp()
    window.show()
    app.exec()
