#!/bin/bash

# PDFOCR CLI 도구 설치 스크립트
# 시스템 전역에서 사용 가능하도록 설치

set -e

echo "=========================================="
echo "PDFOCR CLI 도구 설치"
echo "=========================================="
echo ""

# 색상 코드
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# 스크립트 디렉토리
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# 설치 위치 옵션
echo "설치 방법을 선택하세요:"
echo "  1) 시스템 전역 설치 (/usr/local/bin) - 모든 사용자가 사용 가능"
echo "  2) 사용자 로컬 설치 (~/.local/bin) - 현재 사용자만 사용 가능"
echo "  3) 심볼릭 링크만 생성 (개발 모드)"
echo ""
read -p "선택 (1/2/3): " -n 1 -r
echo ""

case $REPLY in
    1)
        # 시스템 전역 설치
        INSTALL_DIR="/usr/local/bin"
        echo "시스템 전역 설치 선택"
        
        # pdfocr 실행 파일에 실행 권한 부여
        chmod +x "$SCRIPT_DIR/pdfocr"
        
        # 심볼릭 링크 생성
        echo "심볼릭 링크 생성: $INSTALL_DIR/pdfocr -> $SCRIPT_DIR/pdfocr"
        sudo ln -sf "$SCRIPT_DIR/pdfocr" "$INSTALL_DIR/pdfocr"
        
        echo -e "${GREEN}✓ 설치 완료!${NC}"
        echo "어디서든 'pdfocr' 명령어를 사용할 수 있습니다."
        ;;
    2)
        # 사용자 로컬 설치
        INSTALL_DIR="$HOME/.local/bin"
        echo "사용자 로컬 설치 선택"
        
        # 디렉토리가 없으면 생성
        if [ ! -d "$INSTALL_DIR" ]; then
            mkdir -p "$INSTALL_DIR"
            echo "디렉토리 생성: $INSTALL_DIR"
        fi
        
        # pdfocr 실행 파일에 실행 권한 부여
        chmod +x "$SCRIPT_DIR/pdfocr"
        
        # 심볼릭 링크 생성
        echo "심볼릭 링크 생성: $INSTALL_DIR/pdfocr -> $SCRIPT_DIR/pdfocr"
        ln -sf "$SCRIPT_DIR/pdfocr" "$INSTALL_DIR/pdfocr"
        
        echo -e "${GREEN}✓ 설치 완료!${NC}"
        
        # PATH 확인
        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            echo ""
            echo -e "${YELLOW}⚠ 경고: $INSTALL_DIR이 PATH에 없습니다${NC}"
            echo ""
            echo "다음 명령어를 ~/.bashrc 또는 ~/.zshrc에 추가하세요:"
            echo ""
            echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
            echo ""
            echo "그 후 터미널을 재시작하거나 다음을 실행하세요:"
            echo "    source ~/.bashrc  (또는 source ~/.zshrc)"
        else
            echo "어디서든 'pdfocr' 명령어를 사용할 수 있습니다."
        fi
        ;;
    3)
        # 심볼릭 링크만 (개발 모드)
        echo "개발 모드 - 심볼릭 링크만 생성"
        
        # pdfocr 실행 파일에 실행 권한 부여
        chmod +x "$SCRIPT_DIR/pdfocr"
        
        echo -e "${GREEN}✓ 실행 권한 부여 완료${NC}"
        echo ""
        echo "다음 방법으로 사용할 수 있습니다:"
        echo "  1) 절대 경로: $SCRIPT_DIR/pdfocr <PDF파일>"
        echo "  2) PATH 추가: export PATH=\"$SCRIPT_DIR:\$PATH\""
        echo "  3) alias 설정: alias pdfocr='$SCRIPT_DIR/pdfocr'"
        ;;
    *)
        echo -e "${RED}✗ 잘못된 선택${NC}"
        exit 1
        ;;
esac

echo ""
echo "=========================================="
echo "사용 예시:"
echo "=========================================="
echo ""
echo "# 현재 디렉토리의 PDF 처리"
echo "pdfocr document.pdf"
echo ""
echo "# 다른 디렉토리의 PDF 처리"
echo "pdfocr ~/Documents/lecture.pdf"
echo ""
echo "# 여러 PDF를 하나로 병합"
echo "pdfocr ~/pdfs/*.pdf --merge -o ~/output"
echo ""
echo "# 도움말 보기"
echo "pdfocr --help"
echo ""
