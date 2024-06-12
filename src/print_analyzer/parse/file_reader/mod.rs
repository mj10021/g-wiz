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

fn split_line(line: &str) -> Vec<Word> {
    let mut out = Vec::new();
    let mut words = line.split_whitespace();
    while let Some(words) = words.next() {
        let mut slice = words.chars();
        if let Some(letter) = slice.nth(0) {
            assert!(letter.is_ascii_alphabetic());
            out.push(Word(
                letter,
                slice.collect::<String>().parse::<f32>().unwrap(),
                None,
            ));
        }
    }
    out
}

pub fn read_line(line: &str) -> Vec<Word> {
    // here i rly want to check if there is a character that doesn't make sense
    // and just pass the raw string through if that's the case
    let mut out = Vec::new();
    let words = line.split_whitespace();
    let mut valid = true;
    let mut first = true;
    for word in words {
        let mut chars = word.chars();
        if let Some(letter) = chars.next() {
            if let Ok(num) = chars.collect::<String>().parse::<f32>() {
                // FIXME: this needs to be only first word
                if first && (num % 1.0).abs() > EPSILON {
                    valid = false;
                } else {
                    out.push(Word(letter, num, None));
                }
            } else {
                valid = false;
            }
        } else {
            valid = false;
        }
        first = false;
    }
    if !valid {
        if line.len() > 1 {
            let word = Word('X', NEG_INFINITY, Some(line.to_owned()));
            return Vec::from([word]);
        } else {
            let word = Word('X', NEG_INFINITY, None);
            return Vec::from([word]);
        }
    }
    split_line(line)
}
