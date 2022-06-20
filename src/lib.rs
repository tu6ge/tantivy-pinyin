use std::str::Chars;

use tantivy::tokenizer::{ Token, Tokenizer, BoxTokenStream, TokenStream};

use pinyin::ToPinyin;

#[cfg(feature = "stop_words")]
pub mod stop_words;

#[derive(Clone)]
pub struct PinyinTokenizer;

pub struct PinyinTokenStream<'a> {
    chars: Chars<'a>,
    offset: usize,
    token: Token,
}

impl Tokenizer for PinyinTokenizer {
    fn token_stream<'a>(&self, text: &'a str) -> BoxTokenStream<'a> {

        BoxTokenStream::from(PinyinTokenStream {
            chars: text.chars(),
            offset: 0,
            token: Token::default(),
        })
    }
}

impl<'a> TokenStream for PinyinTokenStream<'a> {
    fn advance(&mut self) -> bool {
        self.token.text.clear();
        self.token.position = self.token.position.wrapping_add(1);
        while let Some(c) = self.chars.next() {
            let offset_to = self.offset + c.len_utf8();
            self.token.offset_from = self.offset;
            self.token.offset_to = offset_to;
            self.offset = offset_to;
            if let Some(pinyin) = c.to_pinyin() {
                self.token.text.push_str(pinyin.plain());
            }
            return true;
        }
        false
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}

#[cfg(test)]
mod tests{
    use tantivy::tokenizer::{Tokenizer, Token, TokenStream};
    use crate::{PinyinTokenStream, PinyinTokenizer};

    #[test]
    fn test_pinyin_tokenizer() {
        let tokens = token_stream_helper("大多数知识，不需要我们记住");
        assert_eq!(tokens.len(), 13);

        assert_token(&tokens[0], 0, "da", 0, 3);
        assert_token(&tokens[1], 1, "duo", 3, 6);
        assert_token(&tokens[2], 2, "shu", 6, 9);
        assert_token(&tokens[3], 3, "zhi", 9, 12);
        assert_token(&tokens[4], 4, "shi", 12, 15);

        assert_token(&tokens[5], 5, "", 15, 18);

        assert_token(&tokens[6], 6, "bu", 18, 21);
    }

    #[test]
    fn test_advance(){
        let text = "知识";
        let mut token_stream = PinyinTokenStream {
            chars: text.chars(),
            offset: 0,
            token: Token::default(),
        };

        assert_eq!(token_stream.advance(), true);
        assert_eq!(token_stream.advance(), true);
        assert_eq!(token_stream.advance(), false);
    }

    fn token_stream_helper(text: &str) -> Vec<Token> {
        let mut token_stream = PinyinTokenizer.token_stream(text);
        let mut tokens = vec![];
        while token_stream.advance() {
            tokens.push(token_stream.token().clone());
        }
        tokens
    }

    pub fn assert_token(token: &Token, position: usize, text: &str, from: usize, to: usize) {
        assert_eq!(
            token.position, position,
            "expected position {} but {:?}",
            position, token
        );
        assert_eq!(token.text, text, "expected text {} but {:?}", text, token);
        assert_eq!(
            token.offset_from, from,
            "expected offset_from {} but {:?}",
            from, token
        );
        assert_eq!(
            token.offset_to, to,
            "expected offset_to {} but {:?}",
            to, token
        );
    }
}