use num::{
    FromPrimitive,
    rational::Rational,
};
use pest::{Parser, iterators::Pair};

use crate::{Pattern, Event, fast, cat, unit};

#[derive(Parser)]
#[grammar = "mini_notation.pest"]
struct MiniNotationParser;

pub fn parse_pattern(input: &'static str) -> Pattern<String> {
    let pattern = MiniNotationParser::parse(Rule::pattern, input).unwrap_or_else(|e| panic!("{}", e)).next().unwrap();
    _parse_pattern(pattern)
}

fn _parse_pattern(pair: Pair<'static, Rule>) -> Pattern<String> {
    match pair.as_rule() {
        Rule::fast_repeat => {
            let sequence: Vec<_> = pair.into_inner().next().unwrap().into_inner().map(_parse_pattern).collect();
            let n = sequence.len();
            fast((n as isize).into(), cat(sequence))
        },
        Rule::sequence => {
            let sequence: Vec<_> = pair.into_inner().map(_parse_pattern).collect();
            fast((sequence.len() as isize).into(), cat(sequence))
        },
        Rule::cycle => {
            let sequence: Vec<_> = pair.into_inner().next().unwrap().into_inner().map(_parse_pattern).collect();
            cat(sequence)
        },

        Rule::modified_event => {
            let mut inner = pair.into_inner();
            let event = _parse_pattern(inner.next().unwrap());
            let operator = inner.next().unwrap();
            let number:f32 = inner.next().unwrap().as_str().parse().unwrap();
            assert!(inner.next().is_none());

            match operator.as_str() {
                "*" => {
                    fast(Rational::from_f32(number).unwrap(), event)
                },
                "/" => {
                    fast(Rational::from_f32(1.0/number).unwrap(), event)
                },
                "!" => todo!(),
                _ => unreachable!(),
            }
        }

        Rule::raw_event |
        Rule::pattern |
        Rule::bracketed_pattern |
        Rule::event => _parse_pattern(pair.into_inner().next().unwrap()),

        Rule::string => unit(pair.as_str().to_string()),

        Rule::number => todo!(),

        Rule::operator => unreachable!(),
    }
}
