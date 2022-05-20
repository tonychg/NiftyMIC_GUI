from typing import List
from pathlib import Path


from niftymic_gui.settings import settings
from niftymic_gui.exceptions import NiftyMICError
from niftymic_gui.process_utils import execute_cmdline


def convert_to_nifti(dicom: str, directory: Path):
    nifty_path = Path(directory) / f"{Path(dicom).name}.nii"

    if not nifty_path.exists():
        return_code = execute_cmdline(
            [
                settings.DCM2NIIX_PATH,
                "-o",
                directory,
                "-f",
                Path(dicom).name,
                dicom,
            ]
        )
        if return_code != 0:
            raise NiftyMICError(
                f"Failed to convert DICOM {dicom} with return code {return_code}",
                dicom,
                return_code,
            )
    return nifty_path


def generate_mask(filenames: List[Path], source: Path, directory: Path):
    return_code = execute_cmdline(
        [
            "docker",
            "run",
            "--rm",
            "-t",
            "-v",
            f"{source}:/{source}",
            settings.NIFTYMIC_DOCKER_IMAGE,
            "niftymic_segment_fetal_brains",
            "--filenames",
            *[str(filename) for filename in filenames],
            "--dir-output",
            str(directory),
        ]
    )
    if return_code != 0:
        raise NiftyMICError("Failed to generate mask")
    return [str(filename) for filename in directory.rglob("*.nii.gz")]


def bias_field_correction(
    filename: List[Path], filename_mask: List[Path], directory: Path
):
    for filename, mask in zip(sorted(filename), sorted(filename_mask)):
        output = directory / filename.name
        return_code = execute_cmdline(
            [
                "docker",
                "run",
                "--rm",
                "-t",
                "-v",
                f"{directory}:{directory}",
                settings.NIFTYMIC_DOCKER_IMAGE,
                "niftymic_correct_bias_field",
                "--filename",
                filename,
                "--filename-mask",
                mask,
                "--output",
                output,
            ]
        )
        if return_code != 0:
            raise NiftyMICError(
                f"Failed to apply bias field correction {filename} {mask}"
            )
    return list(directory.rglob("*.nii"))


def reconstruct(
    filenames: List[Path],
    filenames_masks: List[Path],
    output_directory: Path,
    output: Path,
    alpha: float = 0.01,
    outlier_rejection: int = 1,
    threshold_first: float = 0.5,
    threshold: float = 0.85,
    intensity_correction: int = 1,
    isotropic_resolution: float = 0.8,
    two_step_cycles: int = 3,
):
    return_code = execute_cmdline(
        [
            "docker",
            "run",
            "--rm",
            "-t",
            "-v",
            f"{output_directory}:{output_directory}",
            settings.NIFTYMIC_DOCKER_IMAGE,
            "niftymic_reconstruct_volume",
            "--filenames",
            *sorted(filenames),
            "--filenames-masks",
            *sorted(filenames_masks),
            "--alpha",
            str(alpha),
            "--outlier-rejection",
            str(outlier_rejection),
            "--threshold-first",
            str(threshold_first),
            "--threshold",
            str(threshold),
            "--intensity-correction",
            str(intensity_correction),
            "--isotropic-resolution",
            str(isotropic_resolution),
            "--two-step-cycles",
            str(two_step_cycles),
            "--verbose",
            "1",
            "--output",
            output,
        ]
    )
    if return_code != 0:
        raise NiftyMICError("Failed to reconstruct")
    return return_code


def convert_to_dicom(filename: Path, directory: Path):
    return_code = execute_cmdline(
        [
            settings.MEDCON_PATH,
            "-f",
            filename,
            "-split3d",
            "-c",
            "dicom",
        ],
        cwd=directory,
    )
    if return_code != 0:
        raise NiftyMICError(f"Failed to convert NifTI {filename} to DICOM")
    return return_code
