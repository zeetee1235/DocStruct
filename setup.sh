#!/bin/bash

# PDF → 이미지 → 텍스트 추출 파이프라인 환경 설정 스크립트
# 
# 이 스크립트는 다음을 수행합니다:
# 1. 시스템 의존성 설치 (poppler-utils, tesseract-ocr)
# 2. Python 가상환경 생성
# 3. Python 패키지 설치
# 4. 필요한 디렉토리 생성

set -e  # 오류 발생 시 스크립트 중단

echo "=========================================="
echo "PDF2TXT 파이프라인 환경 설정"
echo "=========================================="
echo ""

# 색상 코드 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# OS 감지
OS="$(uname -s)"
echo "감지된 OS: $OS"
echo ""

# 1. 시스템 패키지 설치
echo "【1단계】 시스템 의존성 설치"
echo "------------------------------------------"

if [[ "$OS" == "Linux" ]]; then
    # Linux 배포판 감지
    if [ -f /etc/debian_version ]; then
        # Debian/Ubuntu
        echo "Debian/Ubuntu 시스템 감지"
        echo "필요한 패키지: poppler-utils, tesseract-ocr, tesseract-ocr-kor"
        
        read -p "시스템 패키지를 설치하시겠습니까? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sudo apt-get update
            sudo apt-get install -y poppler-utils tesseract-ocr tesseract-ocr-kor
            echo -e "${GREEN}✓ 시스템 패키지 설치 완료${NC}"
        else
            echo -e "${YELLOW}⚠ 시스템 패키지 설치를 건너뜁니다${NC}"
            echo "  수동 설치 명령어: sudo apt-get install -y poppler-utils tesseract-ocr tesseract-ocr-kor"
        fi
    elif [ -f /etc/redhat-release ]; then
        # RHEL/CentOS/Fedora
        echo "RHEL/CentOS/Fedora 시스템 감지"
        echo "필요한 패키지: poppler-utils, tesseract, tesseract-langpack-kor"
        
        # dnf와 yum 중 사용 가능한 것 확인
        if command -v dnf &> /dev/null; then
            PKG_MANAGER="dnf"
        elif command -v yum &> /dev/null; then
            PKG_MANAGER="yum"
        else
            echo -e "${RED}✗ 패키지 관리자(dnf/yum)를 찾을 수 없습니다${NC}"
            exit 1
        fi
        
        echo "패키지 관리자: $PKG_MANAGER"
        
        read -p "시스템 패키지를 설치하시겠습니까? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sudo $PKG_MANAGER install -y poppler-utils tesseract tesseract-langpack-kor
            echo -e "${GREEN}✓ 시스템 패키지 설치 완료${NC}"
        else
            echo -e "${YELLOW}⚠ 시스템 패키지 설치를 건너뜁니다${NC}"
            echo "  수동 설치 명령어: sudo $PKG_MANAGER install -y poppler-utils tesseract tesseract-langpack-kor"
        fi
    elif [ -f /etc/arch-release ]; then
        # Arch Linux
        echo "Arch Linux 시스템 감지"
        echo "필요한 패키지: poppler, tesseract, tesseract-data-kor"
        
        read -p "시스템 패키지를 설치하시겠습니까? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sudo pacman -S --noconfirm poppler tesseract tesseract-data-kor
            echo -e "${GREEN}✓ 시스템 패키지 설치 완료${NC}"
        else
            echo -e "${YELLOW}⚠ 시스템 패키지 설치를 건너뜁니다${NC}"
            echo "  수동 설치 명령어: sudo pacman -S poppler tesseract tesseract-data-kor"
        fi
    else
        echo -e "${YELLOW}⚠ 알 수 없는 Linux 배포판${NC}"
        echo "다음 패키지를 수동으로 설치해주세요:"
        echo "  - poppler-utils (또는 poppler)"
        echo "  - tesseract-ocr"
        echo "  - tesseract 한국어 언어팩"
    fi
elif [[ "$OS" == "Darwin" ]]; then
    # macOS
    echo "macOS 시스템 감지"
    echo "필요한 패키지: poppler, tesseract, tesseract-lang"
    
    if command -v brew &> /dev/null; then
        read -p "Homebrew로 패키지를 설치하시겠습니까? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            brew install poppler tesseract tesseract-lang
            echo -e "${GREEN}✓ 시스템 패키지 설치 완료${NC}"
        else
            echo -e "${YELLOW}⚠ 시스템 패키지 설치를 건너뜁니다${NC}"
            echo "  수동 설치 명령어: brew install poppler tesseract tesseract-lang"
        fi
    else
        echo -e "${RED}✗ Homebrew가 설치되어 있지 않습니다${NC}"
        echo "Homebrew 설치: /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
    fi
else
    echo -e "${YELLOW}⚠ 지원하지 않는 OS: $OS${NC}"
    echo "시스템 패키지를 수동으로 설치해주세요."
fi

echo ""

# 2. Python 가상환경 생성
echo "【2단계】 Python 가상환경 설정 (uv 사용)"
echo "------------------------------------------"

if [ -d "venv" ]; then
    echo -e "${YELLOW}⚠ 가상환경이 이미 존재합니다${NC}"
    read -p "기존 가상환경을 삭제하고 재생성하시겠습니까? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf venv
        echo "기존 가상환경 삭제 완료"
    else
        echo "기존 가상환경 사용"
    fi
fi

if [ ! -d "venv" ]; then
    echo "Python 가상환경 생성 중 (uv venv)..."
    if command -v uv &> /dev/null; then
        uv venv venv
    else
        echo -e "${YELLOW}⚠ uv 명령을 찾을 수 없습니다. 기본 venv로 생성합니다.${NC}"
        python3 -m venv venv
    fi
    echo -e "${GREEN}✓ 가상환경 생성 완료${NC}"
fi

# 가상환경 활성화
echo "가상환경 활성화 중..."
source venv/bin/activate
echo -e "${GREEN}✓ 가상환경 활성화 완료${NC}"
echo "  Python 경로: $(which python)"
echo "  Python 버전: $(python --version)"
echo ""

# 3. Python 패키지 설치
echo "【3단계】 Python 패키지 설치 (uv 사용)"
echo "------------------------------------------"

if [ -f "requirements.txt" ]; then
    if command -v uv &> /dev/null; then
        echo "uv를 사용해 requirements.txt에서 패키지 설치 중..."
        uv pip install -r requirements.txt
    else
        echo -e "${YELLOW}⚠ uv 명령을 찾을 수 없습니다. pip로 설치합니다.${NC}"
        echo "pip, setuptools, wheel 업그레이드 중..."
        pip install --upgrade pip setuptools wheel
        echo ""
        echo "requirements.txt에서 패키지 설치 중..."
        pip install -r requirements.txt
    fi
    echo -e "${GREEN}✓ Python 패키지 설치 완료${NC}"
    echo ""
    echo "설치된 패키지:"
    python -m pip list | grep -E "pdf2image|pytesseract|Pillow" || true
else
    echo -e "${RED}✗ requirements.txt 파일을 찾을 수 없습니다${NC}"
fi

echo ""

# 4. 디렉토리 생성
echo "【4단계】 작업 디렉토리 생성"
echo "------------------------------------------"

mkdir -p images
mkdir -p output
echo -e "${GREEN}✓ 디렉토리 생성 완료${NC}"
echo "  - images/ (임시 이미지 저장)"
echo "  - output/ (추출된 텍스트 저장)"
echo ""

# 5. 설치 확인
echo "【5단계】 설치 확인"
echo "------------------------------------------"

echo "Tesseract 버전:"
tesseract --version 2>&1 | head -n 1 || echo -e "${RED}✗ Tesseract를 찾을 수 없습니다${NC}"

echo ""
echo "Tesseract 지원 언어:"
tesseract --list-langs 2>&1 | grep -E "kor|eng" || echo -e "${YELLOW}⚠ 한국어 언어팩이 설치되어 있지 않을 수 있습니다${NC}"

echo ""
echo "pdftoppm 설치 확인:"
command -v pdftoppm &> /dev/null && echo -e "${GREEN}✓ pdftoppm 사용 가능${NC}" || echo -e "${RED}✗ pdftoppm을 찾을 수 없습니다${NC}"

echo ""
echo "=========================================="
echo "환경 설정 완료!"
echo "=========================================="
echo ""
echo "사용 방법:"
echo "  1. 가상환경 활성화: source venv/bin/activate"
echo "  2. 프로그램 실행: python main.py <PDF파일>"
echo ""
echo "예시:"
echo "  python main.py lecture.pdf"
echo "  python main.py pdfs/*.pdf --merge"
echo ""
echo "도움말 보기:"
echo "  python main.py --help"
echo ""
