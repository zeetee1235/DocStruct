#!/usr/bin/env python3
import sys
from pathlib import Path
import difflib
import json

FIXTURES_DIR = Path('tests/fixtures')
OUTPUT_DIRS = [
    'test_output_korean_test_cli',
    'test_output_stress_test_cli',
    'test_output_test_document_cli',
    'test_output_twocolumn_cli',
]


def read_text(p: Path):
    if not p.exists():
        return ''
    return p.read_text(encoding='utf-8', errors='ignore')


def count_blocks_json(p: Path):
    if not p.exists():
        return None
    try:
        j = json.loads(p.read_text(encoding='utf-8'))
        if isinstance(j, dict) and 'pages' in j:
            total = 0
            for page in j['pages']:
                total += len(page.get('blocks', []))
            return total
        # fallback if j is list of blocks
        if isinstance(j, list):
            return len(j)
    except Exception:
        pass
    return None


def similarity(a: str, b: str):
    return difflib.SequenceMatcher(None, a, b).ratio()


def evaluate(output_dir: Path):
    base = output_dir.name.replace('test_output_', '').replace('_cli','')
    fixture_out = FIXTURES_DIR / (base + '.out')
    result = {'name': base, 'output_dir': str(output_dir)}
    doc_txt = output_dir / 'document.txt'
    text = read_text(doc_txt)
    result['chars'] = len(text)
    result['words'] = len(text.split())
    result['lines'] = len(text.splitlines())
    result['blocks'] = count_blocks_json(output_dir / 'document.json')
    if fixture_out.exists():
        expected = read_text(fixture_out)
        result['similarity'] = round(similarity(expected, text), 4)
    else:
        result['similarity'] = None
    return result


def main():
    out = []
    for d in OUTPUT_DIRS:
        p = Path(d)
        if not p.exists():
            print(f"Warning: output dir not found: {d}")
            continue
        out.append(evaluate(p))
    # print summary table
    print("name,chars,words,lines,blocks,similarity")
    for r in out:
        sim = r['similarity'] if r['similarity'] is not None else ''
        blocks = r['blocks'] if r['blocks'] is not None else ''
        print(f"{r['name']},{r['chars']},{r['words']},{r['lines']},{blocks},{sim}")

if __name__ == '__main__':
    main()
