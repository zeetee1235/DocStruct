# Legacy nix-shell support (for users not using flakes)
# This file provides compatibility with traditional nix-shell command
# For flakes users, use: nix develop

{ pkgs ? import <nixpkgs> {} }:

let
  rustToolchain = pkgs.rustc;
  
  pythonEnv = pkgs.python312.withPackages (ps: with ps; [
    pdf2image
    pytesseract
    pillow
    opencv4
    numpy
  ]);

in pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustToolchain
    cargo
    rustfmt
    clippy
    
    # Python environment
    pythonEnv
    
    # System dependencies
    poppler-utils
    tesseract
    
    # Build dependencies
    pkg-config
    openssl
    gcc
    git
  ];

  shellHook = ''
    echo "DocStruct development environment (nix-shell)"
    echo "=============================================="
    echo "Rust version: $(rustc --version)"
    echo "Python version: $(python --version)"
    echo "Tesseract version: $(tesseract --version | head -n1)"
    echo ""
    echo "Note: pix2tex needs to be installed separately:"
    echo "  pip install --user 'pix2tex[gui]>=0.1.2'"
    echo ""
    
    export PATH="$HOME/.local/bin:$PATH"
  '';
}
