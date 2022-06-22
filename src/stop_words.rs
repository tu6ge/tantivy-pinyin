
use tantivy::tokenizer::{StopWordFilter};

pub fn chinese_stop_words() -> StopWordFilter {
    let contents = include_str!("./stop_data");
    
    #[cfg(target_os = "windows")]
    const SPLIT_STR: &str = "\r\n";
    #[cfg(not(target_os = "windows"))]
    const SPLIT_STR: &str = "\n";

    let list: Vec<String> = contents.split(SPLIT_STR).map(|v|v.to_string()).collect();

    StopWordFilter::remove(list)
}


#[cfg(test)]
mod tests {
    use crate::tests::assert_token;
    use super::chinese_stop_words;
    use tantivy::tokenizer::{SimpleTokenizer, TextAnalyzer, Token};

    #[test]
    fn test_stop_word() {
        // 使用了简单的英文分词器（使用空格分词）
        let tokens = token_stream_helper("我 的 家里 有 只 猫");
        assert_eq!(tokens.len(), 2);
        assert_token(&tokens[0], 2, "家里", 8, 14);
        assert_token(&tokens[1], 5, "猫", 23, 26);
    }

    fn token_stream_helper(text: &str) -> Vec<Token> {
        let a = TextAnalyzer::from(SimpleTokenizer).filter(chinese_stop_words());
        let mut token_stream = a.token_stream(text);
        let mut tokens: Vec<Token> = vec![];
        let mut add_token = |token: &Token| {
            tokens.push(token.clone());
        };
        token_stream.process(&mut add_token);
        tokens
    }
}