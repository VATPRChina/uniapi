use super::lexer::{LexerToken, LexerTokenValue};
use crate::modules::navdata::service::{NavdataResult, NavdataService, ResolvedIdent};

pub struct Resolver<'r, 's> {
    navdata: &'s NavdataService,
    tokens: Vec<LexerToken<'r>>,
}

impl<'r, 's> Resolver<'r, 's> {
    pub fn new(
        navdata: &'s NavdataService,
        tokens: impl IntoIterator<Item = LexerToken<'r>>,
    ) -> Self {
        Self {
            navdata,
            tokens: tokens.into_iter().collect(),
        }
    }

    pub async fn resolve_tokens(self) -> NavdataResult<Vec<ResolvedToken<'r>>> {
        let mut resolved = Vec::with_capacity(self.tokens.len());
        for token in self.tokens {
            let resolved_identifiers = match &token.value {
                LexerTokenValue::Identifier => self.navdata.resolve_identifier(token.str).await?,
                LexerTokenValue::IdentifierReference { ident, .. } => self
                    .navdata
                    .find_fixes(ident)
                    .await?
                    .into_iter()
                    .map(ResolvedIdent::Fix)
                    .collect(),
                LexerTokenValue::SpeedAndAltitude { .. }
                | LexerTokenValue::Direct
                | LexerTokenValue::Geo { .. } => Vec::new(),
            };
            resolved.push(ResolvedToken {
                token,
                resolved_identifiers,
            });
        }
        Ok(resolved)
    }
}

#[derive(Debug, PartialEq)]
pub struct ResolvedToken<'r> {
    pub token: LexerToken<'r>,
    pub resolved_identifiers: Vec<ResolvedIdent>,
}
