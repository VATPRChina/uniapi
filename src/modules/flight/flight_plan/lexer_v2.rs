pub struct Lexer<'a> {
    route: &'a str,
    state: LexerState,
}

pub enum LexerState {
    Fix,
    Leg,
}

pub struct LexerToken<'a> {
    str: &'a str,
    value: LexerTokenValue,
}

pub enum LexerTokenValue {
    Direct,
}

impl<'a> Lexer<'a> {
    pub fn new(route: &'a str) -> Self {
        Self {
            route,
            state: LexerState::Fix,
        }
    }

    pub fn parse_all(&self) -> impl Iterator<Item = LexerToken<'a>> {
        self.route
            .split([' ', '\t', '\n', '\r'])
            .filter(|seg| !seg.is_empty())
            .map(|seg| LexerToken {
                str: seg,
                value: LexerTokenValue::Direct,
            })
    }
}

trait TokenHandler {
    fn handle_segment<'a>(segment: &'a str, lexer: &'a Lexer<'a>) -> (LexerToken<'a>, LexerState);
}
