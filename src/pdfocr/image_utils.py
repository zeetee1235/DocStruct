"""
Shared image loading utilities.
"""
from pathlib import Path

import cv2
import numpy as np

from pdfocr.types import PathLike


def read_image(path: PathLike) -> np.ndarray:
    """
    Read an image from disk, raising a helpful error when loading fails.
    """
    image = cv2.imread(str(Path(path)), cv2.IMREAD_COLOR)
    if image is None:
        raise FileNotFoundError(f"이미지를 읽을 수 없습니다: {path}")
    return image
