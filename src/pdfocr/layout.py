"""
페이지 이미지를 블록(텍스트/표/수식 후보) 단위로 감지하는 기본 유틸.
"""
from dataclasses import dataclass
from pathlib import Path
from typing import List, Sequence

import cv2

from pdfocr.image_utils import read_image
from pdfocr.types import PathLike


@dataclass(frozen=True)
class Block:
    x: int
    y: int
    w: int
    h: int

    @property
    def area(self) -> int:
        return self.w * self.h

    def as_bbox(self) -> tuple[int, int, int, int]:
        return self.x, self.y, self.w, self.h


def detect_blocks(image_path: PathLike,
                  min_area: int = 800,
                  merge_kernel: tuple[int, int] = (15, 7)) -> List[Block]:
    """
    간단한 형태학적 연산으로 텍스트/표 블록 후보를 감지한다.
    - 흑백 변환 → 적응형 이진화 → 팽창으로 인접 문자/셀 병합 → 외곽선 감지
    """
    path = Path(image_path)
    image = read_image(path)

    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
    thresh = cv2.adaptiveThreshold(
        gray, 255, cv2.ADAPTIVE_THRESH_GAUSSIAN_C, cv2.THRESH_BINARY_INV, 35, 15
    )

    kernel = cv2.getStructuringElement(cv2.MORPH_RECT, merge_kernel)
    dilated = cv2.dilate(thresh, kernel, iterations=1)

    contours, _ = cv2.findContours(dilated, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)

    blocks: List[Block] = []
    for contour in contours:
        x, y, w, h = cv2.boundingRect(contour)
        if w * h < min_area:
            continue
        blocks.append(Block(x, y, w, h))

    # 좌상단→우하단 순서로 정렬
    blocks.sort(key=lambda b: (b.y, b.x))
    return blocks


def draw_blocks(image_path: PathLike, blocks: Sequence[Block], output_path: PathLike) -> Path:
    """
    감지된 블록을 직사각형으로 표시한 이미지를 저장한다.
    """
    path = Path(image_path)
    image = read_image(path)

    for idx, block in enumerate(blocks, start=1):
        x, y, w, h = block.as_bbox()
        cv2.rectangle(image, (x, y), (x + w, y + h), (0, 128, 255), 2)
        cv2.putText(
            image, str(idx), (x, y - 5),
            cv2.FONT_HERSHEY_SIMPLEX, 0.6, (0, 128, 255), 1, cv2.LINE_AA
        )

    out_path = Path(output_path)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    cv2.imwrite(str(out_path), image)
    return out_path
