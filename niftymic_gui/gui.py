import os
import logging
import shutil
from pathlib import Path

from PyQt5.QtWidgets import (
    QLabel,
    QPushButton,
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QFileDialog,
    QListWidget,
    QFileSystemModel,
    QTreeView,
)
from PyQt5 import (
    QtCore,
    QtWidgets,
)

from PyQt5.QtCore import QObject, QThread, pyqtSignal
from niftymic_gui.helpers import logger
from niftymic_gui.settings import settings
from niftymic_gui.exceptions import NiftyMICError
from niftymic_gui.reconstruction import Reconstruction
from niftymic_gui.process_utils import kill_all_process


class QTextEditLogger(logging.Handler, QtCore.QObject):
    appendPlainText = QtCore.pyqtSignal(str)

    def __init__(self, parent):
        super().__init__()
        QtCore.QObject.__init__(self)
        self.widget = QtWidgets.QPlainTextEdit(parent)
        self.widget.setReadOnly(True)
        self.appendPlainText.connect(self.widget.appendPlainText)

    def emit(self, record):
        msg = self.format(record)
        self.appendPlainText.emit(msg)


class Worker(QObject):
    finished = pyqtSignal()
    progress = pyqtSignal(str)

    def __init__(self, current_reconstruction):
        super().__init__()
        self.current_reconstruction = current_reconstruction
        self.is_stopped = False

    def run(self):
        try:
            if not self.is_stopped:
                self.progress.emit("1/5 Loading existing files")
                self.current_reconstruction.load_existing_files()
            if not self.is_stopped:
                self.progress.emit("2/5 Converting DICOMs to NifTI")
                self.current_reconstruction.convert_dicoms()
            if not self.is_stopped:
                self.progress.emit("3/5 Generating masks from NifTI images")
                self.current_reconstruction.generate_masks()
            if not self.is_stopped:
                self.progress.emit(
                    "4/5 Reconstruct high resolution 3D volume from input NifTI"
                )
                self.current_reconstruction.reconstruct()
            if not self.is_stopped:
                self.progress.emit("5/5 Convert NifTI output to DICOM")
                self.current_reconstruction.convert_output_to_dicom()
            self.finished.emit()
        except NiftyMICError as error:
            logger.error(error)

    def stop(self):
        self.is_stopped = True
        self.finished.emit()


class QtApp(QWidget):
    def __init__(self):
        super().__init__()
        self.title = "NiftyMIC GUI - v0.0.1"
        self.setWindowTitle(self.title)
        self.resize(1220, 720)

        self.layout = QHBoxLayout()

        self.create_output_directory()

        # Left Panel
        self.left_panel = QVBoxLayout()
        self.label_title = QLabel("Select a DICOM input directory")
        self.button_add = QPushButton("Import DICOM")
        self.button_add.clicked.connect(self.dialog)
        self.button_delete = QPushButton("Delete DICOM")
        self.button_delete.clicked.connect(self.delete)
        self.button_start = QPushButton("Start Reconstruction")
        self.button_start.clicked.connect(self.start)
        self.button_stop = QPushButton("Stop Reconstruction")
        self.button_stop.clicked.connect(self.stop)
        self.dicom_list = QListWidget()
        self.label_status = QLabel("Welcome to NiftyMIC GUI")
        self.left_panel.addWidget(self.label_title)
        self.left_panel.addWidget(self.dicom_list)
        self.left_panel.addWidget(self.button_add)
        self.left_panel.addWidget(self.button_delete)
        self.left_panel.addWidget(self.button_start)
        self.left_panel.addWidget(self.button_stop)
        self.left_panel.addWidget(self.label_status)
        self.layout.addLayout(self.left_panel)

        # Right panel
        self.right_layout = QVBoxLayout()
        self.logger_window = QTextEditLogger(self)
        logger.addHandler(self.logger_window)
        logger.setLevel(logging.DEBUG)
        self.logger_window.setFormatter(logging.Formatter("%(asctime)s %(message)s"))

        # Right panel
        self.label_output_directory = QLabel("Output directory : DEFAULT")
        self.model = QFileSystemModel()
        self.model.setRootPath("")
        self.tree = QTreeView()
        self.tree.setModel(self.model)
        self.tree.setRootIndex(self.model.index(settings.OUTPUT_DIRECTORY))
        self.tree.hideColumn(1)
        self.tree.hideColumn(2)
        self.tree.hideColumn(3)

        self.tree.setAnimated(False)
        self.tree.setIndentation(20)
        self.tree.setSortingEnabled(True)

        self.tree.setWindowTitle("Dir View")
        self.tree.resize(800, 720)
        self.tree.clicked.connect(self.select_tree)
        self.button_remove_output = QPushButton("Remove Output Directory")
        self.button_remove_output.clicked.connect(self.remove_output)

        self.right_layout.addWidget(self.tree)
        self.right_layout.addWidget(self.label_output_directory)
        self.right_layout.addWidget(self.button_remove_output)
        self.right_layout.addWidget(self.logger_window.widget)
        self.layout.addLayout(self.right_layout)

        self.setLayout(self.layout)

        self.current_item = None
        self.output_directory = None
        self.current_reconstruction = None
        self.worker = None
        self.thread = None

        self.button_start.setEnabled(False)

        self.showMaximized()

    @classmethod
    def create_output_directory(cls):
        output_directory = Path(settings.OUTPUT_DIRECTORY)
        if not output_directory.exists():
            output_directory.mkdir(parents=True, exist_ok=True)

    def dialog(self):
        dicom_dir = QFileDialog.getExistingDirectory(
            None,
            "Select a folder: ",
            os.environ["HOME"],
        )
        for dicom in Path(dicom_dir).iterdir():
            logger.info(f"Add {dicom} to the list of input DICOMs")
            self.dicom_list.insertItem(
                0,
                str(dicom),
            )
        if self.dicom_list.count() > 0:
            self.button_start.setEnabled(True)

    def delete(self):
        items = self.dicom_list.selectedItems()
        if items is not None:
            for item in items:
                logger.info(f"Remove {item.text()} from the list of input DICOMs")
                self.dicom_list.takeItem(self.dicom_list.row(item))

    def report_progress(self, n):
        self.label_status.setText(n)

    def select_tree(self):
        if len(self.tree.selectedIndexes()) > 1:
            logger.info("You can't select more than one output directory")
        else:
            self.output_directory = str(
                Path(settings.OUTPUT_DIRECTORY)
                / self.model.itemData(self.tree.selectedIndexes()[0])[0]
            )
            self.label_output_directory.setText(self.output_directory)

    def start(self):
        self.current_reconstruction = Reconstruction(
            [
                str(self.dicom_list.item(i).text())
                for i in range(self.dicom_list.count())
            ],
            output_directory=self.output_directory,
        )
        self.thread = QThread()
        # Step 3: Create a worker object
        self.worker = Worker(self.current_reconstruction)
        # Step 4: Move worker to the thread
        self.worker.moveToThread(self.thread)
        # Step 5: Connect signals and slots
        self.thread.started.connect(self.worker.run)
        self.worker.finished.connect(self.thread.quit)
        self.worker.finished.connect(self.worker.deleteLater)
        self.thread.finished.connect(self.thread.deleteLater)
        self.worker.progress.connect(self.report_progress)
        # Step 6: Start the thread
        self.thread.start()

        # Final resets
        self.thread.finished.connect(lambda: self.button_start.setEnabled(True))
        self.thread.finished.connect(
            lambda: self.label_status.setText("All jobs are finished")
        )

    def remove_output(self):
        try:
            if (
                self.output_directory is not None
                and Path(self.output_directory).exists()
            ):
                shutil.rmtree(self.output_directory)
                self.output_directory = None
                self.label_output_directory.setText("Output directory : DEFAULT")
        except FileNotFoundError as error:
            logger.error(error)

    def stop(self):
        if self.worker is not None:
            self.worker.stop()
        if self.thread is not None:
            self.thread.terminate()
        kill_all_process()
