use std::env;
use std::fs;

#[derive(Debug)]
struct PdfObject {
    number: u32,
    generation: u16,
    line: usize,
}

fn main() {
    let path = match env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("Usage: pdfparser <file.pdf>");
            std::process::exit(1);
        }
    };

    let data = match fs::read(&path) {
        Ok(d) => d,
        Err(err) => {
            eprintln!("Failed to read {path}: {err}");
            std::process::exit(1);
        }
    };

    if !data.starts_with(b"%PDF-") {
        eprintln!("Not a PDF header: expected %PDF-...");
        std::process::exit(1);
    }

    let header = data
        .split(|b| *b == b'\n' || *b == b'\r')
        .next()
        .unwrap_or(&[]);
    let header_str = String::from_utf8_lossy(header);

    let objects = scan_objects(&data);

    println!("Header: {header_str}");
    println!("Objects found: {}", objects.len());
    let preview = objects.len().min(20);
    if preview > 0 {
        println!("First {preview} objects:");
        for obj in objects.iter().take(preview) {
            println!("- {} {} obj (line {})", obj.number, obj.generation, obj.line);
        }
    }
}

fn scan_objects(data: &[u8]) -> Vec<PdfObject> {
    let mut objects = Vec::new();
    let mut line_num = 1usize;
    let mut start = 0usize;

    while start < data.len() {
        let mut end = start;
        while end < data.len() && data[end] != b'\n' && data[end] != b'\r' {
            end += 1;
        }

        let line = trim_ascii(&data[start..end]);
        if let Some((num, gen)) = parse_obj_line(line) {
            objects.push(PdfObject {
                number: num,
                generation: gen,
                line: line_num,
            });
        }

        while end < data.len() && (data[end] == b'\n' || data[end] == b'\r') {
            end += 1;
        }
        line_num += 1;
        start = end;
    }

    objects
}

fn trim_ascii(input: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = input.len();

    while start < end && input[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && input[end - 1].is_ascii_whitespace() {
        end -= 1;
    }

    &input[start..end]
}

fn parse_obj_line(line: &[u8]) -> Option<(u32, u16)> {
    if line.len() < 5 || !line.ends_with(b"obj") {
        return None;
    }

    let mut parts = line.split(|b| b.is_ascii_whitespace());
    let num = parts.next().and_then(parse_u32)?;
    let gen = parts.next().and_then(parse_u16)?;

    let last = parts.last().unwrap_or(&[]);
    if last != b"obj" {
        return None;
    }

    Some((num, gen))
}

fn parse_u32(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        return None;
    }
    let mut value = 0u32;
    for b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        value = value * 10 + (b - b'0') as u32;
    }
    Some(value)
}

fn parse_u16(bytes: &[u8]) -> Option<u16> {
    parse_u32(bytes).and_then(|v| u16::try_from(v).ok())
}
