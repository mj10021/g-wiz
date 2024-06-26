use super::*;

pub fn parse_file(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let out = String::from_utf8(std::fs::read(path)?)
        .unwrap()
        .split("\n")
        .map(|s| s.split(';').next().unwrap()) // ignore any ';' comments
        .map(str::to_string)
        .filter(|s| !s.is_empty())
        .collect();
    Ok(out)
}

pub fn parse_str(str: &str) -> Vec<String> {
    String::from(str)
        .split("\n")
        .map(|s| s.split(';').next().unwrap())
        .map(str::to_string)
        .collect()
}

pub fn split_line(line: &str) -> Vec<Word> {
    let mut out = Vec::new();
    let words = line.split_whitespace();
    for word in words {
        let mut slice = word.chars();
        if let Some(letter) = slice.next() {
            assert!(letter.is_ascii_alphabetic());
            if let Ok(num) = slice.collect::<String>().parse::<f32>() {
                out.push(Word(letter, num, None));
            } else {
                return Vec::from([Word('X', f32::NEG_INFINITY, Some(line.to_owned()))]);
            }
        }
    }
    if let Word('N', ..) = out[0] {
        out.reverse();
        out.pop();
        out.reverse();
    }
    out
}
