{
  description = "DocStruct - PDF document structure recovery system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable."1.84.0".default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        pythonEnv = pkgs.python312.withPackages (ps: with ps; [
          pdf2image
          pytesseract
          pillow
          opencv4
          numpy
          # pix2tex needs to be installed via pip as it's not in nixpkgs
        ]);

        buildInputs = with pkgs; [
          # Rust toolchain
          rustToolchain
          cargo
          rustc
          
          # Python environment
          pythonEnv
          
          # System dependencies
          poppler-utils  # pdfinfo, pdftotext, pdftoppm
          tesseract      # OCR engine
          
          # Additional utilities
          pkg-config
          openssl
        ];

        nativeBuildInputs = with pkgs; [
          # Build tools
          gcc
          git
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          
          shellHook = ''
            echo "DocStruct development environment"
            echo "=================================="
            echo "Rust version: $(rustc --version)"
            echo "Python version: $(python --version)"
            echo "Tesseract version: $(tesseract --version | head -n1)"
            echo ""
            echo "Note: pix2tex needs to be installed separately:"
            echo "  pip install --user 'pix2tex[gui]>=0.1.2'"
            echo ""
            
            # Set up Python path for local development
            export PYTHONPATH="${pythonEnv}/${pythonEnv.sitePackages}:$PYTHONPATH"
            
            # Ensure pip user installations are in PATH
            export PATH="$HOME/.local/bin:$PATH"
          '';

          # Environment variables
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        # Package definition for building the project
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "docstruct";
          version = "0.1.0";
          
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          nativeBuildInputs = nativeBuildInputs;
          buildInputs = buildInputs;
          
          meta = with pkgs.lib; {
            description = "PDF document structure recovery system";
            homepage = "https://github.com/zeetee1235/DocStruct";
            license = licenses.mit;
            maintainers = [ ];
          };
        };
      }
    );
}
