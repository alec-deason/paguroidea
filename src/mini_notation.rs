use pest::{Parser, iterators::Pair};

use crate::{Pattern};

#[derive(Parser)]
#[grammar = "mini_notation.pest"]
struct MiniNotationParser;

pub fn parse_pattern(input: &str) -> Pattern<String> {
    let pattern = MiniNotationParser::parse(Rule::pattern, input).unwrap_or_else(|e| panic!("{}", e));
    _parse_pattern(pattern)
}

fn _parse_pattern(pair: Pair<Rule>) -> Pattern<String> {
    "thing"
}
