# NiftyMIC GUI

These scripts create a PyQT5 interface to allow he use to uninitiated users of the NiftyMIC MRI
reconstruction algorithm.

https://github.com/gift-surg/NiftyMIC

## Prerequisites

You need Docker to use the docker image of NiftyMIC : https://hub.docker.com/r/renbem/niftymic

`dcm2niix` is use to convert DICOM to NifTI format, required by NiftyMIC.
`medcon` is use to convert NifTI output to DICOM after all actions.

## Installation

```
pip install niftymic-gui
```
