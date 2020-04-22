use pest::{Parser};

#[derive(Parser)]
#[grammar = "mini_notation.pest"]
struct MiniNotationParser;

pub fn thing() {
    let pattern = MiniNotationParser::parse(Rule::pattern, "[1 <2 3> 3]").unwrap_or_else(|e| panic!("{}", e));

    for event in pattern {
        println!("{:?}", event);
    }

}
