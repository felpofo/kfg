use crate::{Token, TokenKind};

pub fn tokenize(buffer: &[u8]) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let [mut line, mut character] = [1, 0];

    let mut token = Token::default();

    let mut i = 0;
    while i < buffer.len() {
        let char = buffer[i] as char;

        if char == '\n' {
            line += 1;
            character = 0;
        }

        token.position.line = line;
        token.position.character = character;

        token.kind = char.into();

        if let TokenKind::Symbol(ref mut s) = token.kind {
            s.clear();

            loop {
                s.push(buffer[i].into());

                character += 1;

                if i + 1 < buffer.len() {
                    match TokenKind::from(buffer[i + 1] as char) {
                        TokenKind::Symbol(_) => {}
                        _ => break,
                    }
                }

                i += 1;

                // Reached EOF
                if i == buffer.len() {
                    break;
                }
            }
        } else {
            character += 1;
        }

        token.position.length = character - token.position.character;
        tokens.push(token.clone());

        i += 1;
    }

    tokens
}

pub fn filter(tokens: &[Token]) -> Vec<Token> {
    use TokenKind::*;

    let mut array: Vec<Token> = vec![];

    let mut iter = tokens.iter().peekable();

    while let Some(token) = iter.next() {
        let next = iter.peek().map(|&token| &token.kind);

        match (&token.kind, &next) {
            // Comment
            (Slash, Some(Slash)) => {
                while iter.next().is_some_and(|token| !matches!(token.kind, NewLine)) {}

                let from = token.position;
                trace!("\x1b[31m- \x1b[35mComment\x1b[m from \x1b[36m{}:{}\x1b[m to \x1b[36m{}:{}\x1b[m", from.line, from.character, from.line + 1, 0);

                continue;
            }
            // Comment block
            (Slash, Some(Asterisk)) => {
                while !matches!(
                    (iter.next(), iter.peek()),
                    (Some(left), Some(&right))
                        if matches!(left.kind, Asterisk)
                        && matches!(right.kind, Slash)
                ) {}

                let from = token.position;
                let to = iter.next().unwrap().position;
                trace!("\x1b[31m- \x1b[35mComment Block\x1b[m from \x1b[36m{}:{}\x1b[m to \x1b[36m{}:{}\x1b[m", from.line, from.character, to.line, to.character);

                // Remove \n after block end
                if iter.peek().is_some_and(|&token| matches!(token.kind, NewLine)) {
                    let token = iter.next().unwrap();
                    trace!("\x1b[31m- \x1b[33m{:?}\x1b[m at \x1b[36m{}\x1b[m", token.kind, token.position);
                }

                continue;
            }
            // String
            (Quote, _) => {
                let mut new = Token::new(Symbol("".into()), token.position);

                trace!("\x1b[32m+ \x1b[33m{:?}\x1b[m at \x1b[36m{}\x1b[m", token.kind, token.position);
                array.push(token.clone());

                while let Some(token) = iter.next() {
                    match token.kind {
                        Quote => {
                            new.position += token.position;

                            trace!("\x1b[32m+ \x1b[33m{:?}\x1b[m from \x1b[36m{}:{}\x1b[m to \x1b[36m{}:{}\x1b[m", new.kind, new.position.line, new.position.character, new.position.line, new.position.character + new.position.length);
                            trace!("\x1b[32m+ \x1b[33m{:?}\x1b[m at \x1b[36m{}\x1b[m", token.kind, token.position);
                            array.push(new);
                            array.push(token.clone());

                            break;
                        }
                        _ => new.join(token).unwrap(),
                    }
                }

                continue;
            }
            // Ignore spaces
            (Space, _) => {
                trace!("\x1b[31m- \x1b[33m{:?}\x1b[m at \x1b[36m{}\x1b[m", token.kind, token.position);
                continue;
            }
            _ => {}
        }

        trace!("\x1b[32m+ \x1b[33m{:?}\x1b[m at \x1b[36m{}\x1b[m", token.kind, token.position);
        array.push(token.clone());
    }

    array
}

pub fn lex(buffer: &[u8]) -> Vec<Token> {
    filter(&tokenize(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        use TokenKind::*;

        let buffer: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789,.\n:/' *=[]{}";

        let expect: [Token; 14] = [
            Token::new(Symbol("abcdefghijklmnopqrstuvwxyz0123456789".into()), (1, 0, 36)),
            Token::new(Comma, (1, 36, 1)),
            Token::new(Dot, (1, 37, 1)),
            Token::new(NewLine, (2, 0, 1)),
            Token::new(Colon, (2, 1, 1)),
            Token::new(Slash, (2, 2, 1)),
            Token::new(Quote, (2, 3, 1)),
            Token::new(Space, (2, 4, 1)),
            Token::new(Asterisk, (2, 5, 1)),
            Token::new(Equals, (2, 6, 1)),
            Token::new(OpenBracket, (2, 7, 1)),
            Token::new(CloseBracket, (2, 8, 1)),
            Token::new(OpenCurly, (2, 9, 1)),
            Token::new(CloseCurly, (2, 10, 1)),
        ];

        let tokens = tokenize(buffer);

        assert_eq!(tokens, expect);
    }

    #[test]
    fn test_filter() {
        use TokenKind::*;

        let tokens: [Token; 15] = [
            Token::new(Slash, (0, 0, 0)),
            Token::new(Slash, (0, 0, 0)),
            Token::new(Symbol("comment".into()), (0, 0, 0)),
            Token::new(NewLine, (0, 0, 0)),
            Token::new(Symbol("var".into()), (0, 0, 0)),
            Token::new(Equals, (0, 0, 0)),
            Token::new(Symbol("null".into()), (0, 0, 0)),
            Token::new(NewLine, (0, 0, 0)),
            Token::new(Slash, (0, 0, 0)),
            Token::new(Asterisk, (0, 0, 0)),
            Token::new(Symbol("comment".into()), (0, 0, 0)),
            Token::new(Space, (0, 0, 0)),
            Token::new(Symbol("block".into()), (0, 0, 0)),
            Token::new(Asterisk, (0, 0, 0)),
            Token::new(Slash, (0, 0, 0)),
        ];

        let expect: &[Token] = &tokens[4..8];

        let lexed = filter(&tokens);

        assert_eq!(lexed, expect);
    }
}