use crate::common::RecordBigram;
use super::Word;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct WordPair {
    pub word0: Word,
    pub word1: Word,
}

impl WordPair {
    pub fn convert(bigram_rows: Vec<RecordBigram>) -> Vec<WordPair> {
        let mut wordpair_vec: Vec<WordPair> = vec!();
        let mut number_of_parse_errors = 0;
        for bigram_row in bigram_rows {
            let word0 = match Word::parse(&bigram_row.word0) {
                Some(value) => value,
                None => {
                    number_of_parse_errors += 1;
                    continue;
                }
            };
            let word1 = match Word::parse(&bigram_row.word1) {
                Some(value) => value,
                None => {
                    number_of_parse_errors += 1;
                    continue;
                }
            };
            let pair = WordPair {
                word0: word0,
                word1: word1
            };
            wordpair_vec.push(pair);
        }
        if number_of_parse_errors > 0 {
            error!("number of parse errors: {}", number_of_parse_errors);
        }
        wordpair_vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::parse_csv_data;

    #[test]
    fn test_10000_parse_ok() {
        let data = "\
count;word0;word1
29843;START;mov
28868;add;mov
24764;mov;STOP
";
        let mut input: &[u8] = data.as_bytes();
        let records: Vec<RecordBigram> = parse_csv_data(&mut input).unwrap();
        let wordpair_vec = WordPair::convert(records);
        let strings: Vec<String> = wordpair_vec.iter().map(|record| {
            format!("{} {}", record.word0, record.word1)
        }).collect();
        let strings_joined: String = strings.join(",");
        assert_eq!(strings_joined, "START mov,add mov,mov STOP");
    }
}
