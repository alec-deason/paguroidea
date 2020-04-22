#![feature(box_syntax)]
use std::collections::HashMap;
use num::rational::Rational;

use paguroidea::*;

fn main() {
    let pattern = 1;
    let events = pattern.query(Arc { start: 0.into(), stop: 3.into() });
    let events:Vec<_> = events.into_iter().map(|e| e.value).collect();
    println!("{:?}", events);

    let pattern = "one".to_string();
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

    let pattern = Fast {
        speed: 2.into(),
        pattern: box Cat { subpatterns: vec![box 2, box 1] },
    };
    let events = pattern.query(Arc { start: 0.into(), stop: 6.into() });
    let events:Vec<_> = events.into_iter().map(|e| e.value).collect();
    println!("{:?}", events);

    let mut pattern = ControlMap(HashMap::new());
    pattern.0.insert("s".to_string(), Value::String("db".to_string()));
    pattern.0.insert("n".to_string(), Value::Integer(1));
    let events = pattern.query(Arc { start: 0.into(), stop: 3.into() });
    let events:Vec<_> = events.into_iter().map(|e| e.value).collect();
    println!("{:?}", events);

    let pattern = mini_notation::parse_pattern("bd <cp*2 bd*2> [cp bd]");
    let events = pattern.query(Arc { start: 0.into(), stop: 6.into() });
    let events:Vec<_> = events.into_iter().map(|e| e.value).collect();
    println!("{:?}", events);

    let player = sound::Player::new();

    let pattern = mini_notation::parse_pattern("bd cp*2 bd cp");
    let mut current: Rational = 0.into();
    for event in &pattern.query(Arc { start: 0.into(), stop: 12.into() }) {
        let gap = ((event.part.start - current) * 1000).to_integer() as u64;
        std::thread::sleep(std::time::Duration::from_millis(gap));
        current = event.part.start;
        player.play_sample(&event.value);
    }
}
