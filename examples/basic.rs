#![feature(box_syntax)]
use num::rational::Rational;

use paguroidea::*;

fn main() {
    let pattern = 1;
    let events = pattern.query(Arc { start: 0.into(), stop: 3.into() });
    let events:Vec<_> = events.into_iter().map(|e| e.value).collect();
    println!("{:?}", events);

    let pattern = Fast {
        speed: 2.into(),
        pattern: box 1
    };
    let events = pattern.query(Arc { start: 0.into(), stop: 3.into() });
    let events:Vec<_> = events.into_iter().map(|e| e.value).collect();
    println!("{:?}", events);

    let pattern = Fast {
        speed: (1, 3).into(),
        pattern: box 1
    };
    let events = pattern.query(Arc { start: 0.into(), stop: 3.into() });
    let events:Vec<_> = events.into_iter().map(|e| e.value).collect();
    println!("{:?}", events);


    let pattern = Cat { subpatterns: vec![box 2, box 1] };
    let events = pattern.query(Arc { start: 0.into(), stop: 6.into() });
    let events:Vec<_> = events.into_iter().map(|e| e.value).collect();
    println!("{:?}", events);
}
