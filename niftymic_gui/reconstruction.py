import datetime
from pathlib import Path
from typing import List, Optional

from niftymic_gui.settings import settings
from niftymic_gui.helpers import logger
from niftymic_gui.mri_process import (
    convert_to_nifti,
    convert_to_dicom,
    generate_mask,
    reconstruct,
)


class Reconstruction:
    def __init__(self, input_dicoms: List[str], output_directory: Optional[str] = None):
        self.input_dicoms = input_dicoms
        self.input_nii = []
        self.input_nii_masks = []
        self.input_nii_with_bias_field = []
        self.output_directory = (
            Path(output_directory)
            if output_directory is not None
            else Path(
                f"{settings.OUTPUT_DIRECTORY}/"
                f"{str(datetime.datetime.now().strftime('%m-%d-%Y %H-%M-%S'))}"
            )
        )
        self.output_nii = self.output_directory / "output.nii.gz"
        self.output_dicom = self.output_directory / "output.dicom"
        self.output_masks = self.output_directory / "masks"
        self.output_bias_field = self.output_directory / "bias_field"
        self.create_working_directory()

    def load_existing_files(self):
        for filename in self.output_directory.rglob("*.nii"):
            logger.info("Loading input NifTI %s", filename)
            self.input_nii.append(filename)
        for filename in self.output_directory.rglob("masks/**/*.nii.gz"):
            logger.info("Loading input mask %s", filename)
            self.input_nii_masks.append(filename)
        for filename in self.output_bias_field.rglob("*.nii"):
            logger.info("Loading input bias corrected NifTI %s", filename)
            self.input_nii_with_bias_field.append(filename)

    def create_working_directory(self):
        for directory in (
            self.output_directory,
            self.output_masks,
            self.output_bias_field,
            self.output_dicom,
        ):
            directory.mkdir(parents=True, exist_ok=True)

    def convert_dicoms(self):
        for dicom in self.input_dicoms:
            new_nifti_file = convert_to_nifti(dicom, self.output_directory)
            if new_nifti_file not in self.input_nii:
                self.input_nii.append(new_nifti_file)

    def generate_masks(self):
        self.input_nii_masks = generate_mask(
            self.input_nii, self.output_directory, self.output_masks
        )

    def reconstruct(self):
        return reconstruct(
            self.input_nii,
            self.input_nii_masks,
            self.output_directory,
            self.output_nii,
        )

    def convert_output_to_dicom(self):
        return convert_to_dicom(self.output_nii, self.output_dicom)
