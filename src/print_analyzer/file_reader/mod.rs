use super::*;

pub fn parse_file(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let out = String::from_utf8(std::fs::read(path)?)
        .unwrap()
        .split("\n")
        .map(|s| s.split(';').nth(0).unwrap()) // ignore any ';' comments
        .map(str::to_string)
        .filter(|s| s.len() > 0)
        .collect();
    Ok(out)
}

pub fn parse_str(str: &str) -> Vec<String> {
    String::from(str)
        .split("\n")
        .map(|s| s.split(';').nth(0).unwrap())
        .map(str::to_string)
        .collect()
}

pub fn split_line(line: &str) -> Vec<Word> {
    let mut out = Vec::new();
    let mut words = line.split_whitespace();
    while let Some(word) = words.next() {
        let mut slice = word.chars();
        if let Some(letter) = slice.nth(0) {
            assert!(letter.is_ascii_alphabetic());
            if let Ok(num) = slice.collect::<String>().parse::<f32>() {
                out.push(Word(letter, num, None));
            } else {
                return Vec::from([Word('X', NEG_INFINITY, Some(line.to_owned()))])
            }
        }
    }
    match out[0] {
        Word('N', ..) => {
            out.reverse();
            out.pop();
            out.reverse();
        }
        _ => {}
    }
    out
}