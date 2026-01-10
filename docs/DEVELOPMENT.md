# Development Guide

## Setup Development Environment

### 1. Clone Repository
```bash
git clone https://github.com/zeetee1235/pdfocr.git
cd pdfocr
```

### 2. Run Setup Script
```bash
./setup.sh
```

This will:
- Install system dependencies (poppler, tesseract)
- Create Python virtual environment in `.venv/`
- Install Python packages from `requirements.txt`

### 3. Activate Virtual Environment
```bash
# Bash/Zsh
source .venv/bin/activate

# Fish
source .venv/bin/activate.fish
```

## Project Layout

```
pdfocr/
├── src/pdfocr/        # Main source code
├── test/              # Test files and fixtures
├── docs/              # Documentation
├── .venv/             # Virtual environment (gitignored)
└── images/            # Temporary image directory (gitignored)
```

## Running the Application

### Development Mode
```bash
# Direct execution (requires PYTHONPATH)
PYTHONPATH=src python -m pdfocr.main test.pdf

# Using wrapper script
python main.py test.pdf

# Using CLI script
./pdfocr test.pdf
```

### Installation for Testing
```bash
# Create symlink (development mode)
./install.sh
# Choose option 3: Development mode
```

## Code Style

### Python Style Guide
- Follow PEP 8
- Use type hints where appropriate
- Docstrings for all public functions
- Maximum line length: 100 characters

### Example Function
```python
def process_file(input_path: PathLike, output_dir: PathLike | None = None) -> Path:
    """
    Process a single file.
    
    Args:
        input_path: Path to input file
        output_dir: Optional output directory
    
    Returns:
        Path to generated output file
    """
    # Implementation
    pass
```

## Adding New Features

### 1. Create Feature Branch
```bash
git checkout -b feature/your-feature-name
```

### 2. Implement Changes
- Add code in appropriate module under `src/pdfocr/`
- Add type hints
- Add docstrings
- Handle errors gracefully

### 3. Test Changes
```bash
# Run integration test
./test.fish

# Manual testing
./pdfocr test/test_document.pdf -o /tmp/test
```

### 4. Update Documentation
- Update README.md if user-facing
- Update ARCHITECTURE.md if structural changes
- Add comments for complex logic

### 5. Commit Changes
```bash
git add .
git commit -m "feat: add your feature description"
```

## Common Development Tasks

### Adding a New OCR Language
1. Install language pack:
   ```bash
   sudo dnf install tesseract-langpack-<language>
   ```

2. Test with language code:
   ```bash
   ./pdfocr file.pdf -l <language_code>
   ```

### Adding a New Module
1. Create file in `src/pdfocr/`
2. Add imports to `src/pdfocr/__init__.py` if needed
3. Write functions with type hints
4. Add to main pipeline if needed

### Debugging

#### Enable Debug Logging
Modify logging level in module:
```python
logging.basicConfig(
    level=logging.DEBUG,  # Change from INFO to DEBUG
    format='%(message)s'
)
```

#### Keep Intermediate Files
```bash
./pdfocr file.pdf --keep-images -i ./debug_images
```

#### Examine OCR Output
```bash
# View detected text
cat output/file_extracted.txt

# Check block detection
cat test/output/block_debug_page1.json
```

## Testing

### Integration Test
```bash
./test.fish
```

This script:
1. Builds test PDF from LaTeX
2. Runs full OCR pipeline
3. Tests block detection
4. Verifies output files

### Manual Testing Checklist
- [ ] Single PDF processing
- [ ] Multiple PDF processing
- [ ] Merge functionality
- [ ] Different languages (kor, eng, kor+eng)
- [ ] Different DPI settings
- [ ] Keep images option
- [ ] Error handling (invalid paths, corrupt PDFs)

### Test Cases to Cover
1. **Valid inputs**
   - Single PDF
   - Multiple PDFs
   - Wildcards

2. **Invalid inputs**
   - Non-existent file
   - Invalid PDF
   - No write permissions

3. **Edge cases**
   - Empty PDF
   - Very large PDF (100+ pages)
   - Scanned documents
   - Text-based PDFs

## Troubleshooting

### Import Errors
```bash
# Ensure PYTHONPATH includes src/
export PYTHONPATH="${PWD}/src:${PYTHONPATH}"
```

### Tesseract Not Found
```bash
# Check installation
tesseract --version

# Check language packs
tesseract --list-langs
```

### Poppler Not Found
```bash
# Verify pdftoppm is available
which pdftoppm

# Test manually
pdftoppm -png test.pdf test_page
```

### Permission Errors
```bash
# Make scripts executable
chmod +x pdfocr main.py test.fish setup.sh install.sh
```

## Release Process

### Version Numbering
Follow semantic versioning: `MAJOR.MINOR.PATCH`
- MAJOR: Breaking changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes

### Creating a Release

1. **Update version references**
   - No version file currently, consider adding `__version__.py`

2. **Update CHANGELOG** (if exists)
   ```markdown
   ## [0.0.2] - 2026-01-07
   ### Added
   - Feature description
   
   ### Fixed
   - Bug fix description
   ```

3. **Commit changes**
   ```bash
   git add .
   git commit -m "chore: prepare release v0.0.2"
   ```

4. **Create tag**
   ```bash
   git tag -a v0.0.2 -m "Release v0.0.2

   Changes:
   - Feature 1
   - Feature 2
   - Bug fix
   "
   ```

5. **Push to remote**
   ```bash
   git push origin main
   git push origin v0.0.2
   ```

## Contributing Guidelines

### Before Submitting
- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Commit messages are clear

### Commit Message Format
```
<type>: <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

Example:
```
feat: add support for batch processing

Added ability to process multiple PDFs in parallel.
Includes --merge option to combine outputs.

Closes #123
```

## Performance Optimization

### Profiling
```python
import cProfile
import pstats

# Profile main function
cProfile.run('main()', 'profile_stats')

# View results
p = pstats.Stats('profile_stats')
p.sort_stats('cumulative').print_stats(20)
```

### Memory Profiling
```bash
# Install memory_profiler
pip install memory_profiler

# Profile script
python -m memory_profiler main.py test.pdf
```

### Optimization Tips
1. Process images one at a time (already done)
2. Use generators for large file lists
3. Consider multiprocessing for batch operations
4. Cache frequently accessed data

## Useful Commands

```bash
# Format code
black src/

# Type checking
mypy src/

# Lint code
pylint src/pdfocr/

# Count lines of code
find src/ -name "*.py" -exec wc -l {} + | sort -n

# Find TODO comments
grep -r "TODO" src/

# List all imports
grep -r "^import\|^from" src/ | sort -u
```

## Resources

- [Tesseract Documentation](https://tesseract-ocr.github.io/)
- [pdf2image GitHub](https://github.com/Belval/pdf2image)
- [Pillow Documentation](https://pillow.readthedocs.io/)
- [OpenCV Python Tutorials](https://docs.opencv.org/master/d6/d00/tutorial_py_root.html)
