#![feature(box_syntax)]
use std::{
    collections::HashMap,
    io::Read,
    fs::File, io::BufReader,
};
use num::rational::Rational;

use paguroidea::*;

fn main() {
    let pattern = 1;
    let events = pattern.query(Arc { start: 0.into(), stop: 3.into() });
    let events:Vec<_> = events.into_iter().map(|e:Event<u32>| e.value).collect();
    println!("{:?}", events);

    let pattern = "one".to_string();
    let events = pattern.query(Arc { start: 0.into(), stop: 3.into() });
    let events:Vec<_> = events.into_iter().map(|e:Event<String>| e.value).collect();
    println!("{:?}", events);

    let pattern = Fast {
        speed: 2.into(),
        pattern: box 1
    };
    let events = pattern.query(Arc { start: 0.into(), stop: 3.into() });
    let events:Vec<u32> = events.into_iter().map(|e:Event<u32>| e.value).collect();
    println!("{:?}", events);

    let pattern = Fast {
        speed: (1, 3).into(),
        pattern: box 1
    };
    let events = pattern.query(Arc { start: 0.into(), stop: 3.into() });
    let events:Vec<_> = events.into_iter().map(|e:Event<u32>| e.value).collect();
    println!("{:?}", events);


    let pattern = Cat { subpatterns: vec![box 2, box 1] };
    let events = pattern.query(Arc { start: 0.into(), stop: 6.into() });
    let events:Vec<_> = events.into_iter().map(|e:Event<u32>| e.value).collect();
    println!("{:?}", events);

    let pattern = Fast {
        speed: 2.into(),
        pattern: box Cat { subpatterns: vec![box 2, box 1] },
    };
    let events = pattern.query(Arc { start: 0.into(), stop: 6.into() });
    let events:Vec<_> = events.into_iter().map(|e:Event<u32>| e.value).collect();
    println!("{:?}", events);

    let mut pattern = ControlMap(HashMap::new());
    pattern.0.insert("s".to_string(), Value::String("db".to_string()));
    pattern.0.insert("n".to_string(), Value::Integer(1));
    let events = pattern.query(Arc { start: 0.into(), stop: 3.into() });
    let events:Vec<_> = events.into_iter().map(|e:Event<ControlMap>| e.value).collect();
    println!("{:?}", events);

    let pattern = mini_notation::parse_pattern("bd cp <cp bd>");
    let events = pattern.query(Arc { start: 0.into(), stop: 12.into() });
    let events:Vec<_> = events.into_iter().map(|e:Event<String>| e.value).collect();
    println!("{:?}", events);

    let mut samples = sound::SampleBank::new();
    let mut file = File::open("/home/alec/.local/share/SuperCollider/downloaded-quarks/Dirt-Samples/bd/BT0A0A7.wav").unwrap();
    let mut data = vec![];
    file.read_to_end(&mut data);
    samples.add_sample_set("bd", vec![data]);

    let mut file = File::open("/home/alec/.local/share/SuperCollider/downloaded-quarks/Dirt-Samples/cp/HANDCLP0.wav").unwrap();
    let mut data = vec![];
    file.read_to_end(&mut data);
    samples.add_sample_set("cp", vec![data]);
    let player = sound::Player::new(samples);

    let pattern = Sound(box Stack(vec![
        box mini_notation::parse_pattern("bd cp <cp bd>"),
        box mini_notation::parse_pattern("bd"),
    ]));

/*
    let pattern = Off(
        box Sound(box mini_notation::parse_pattern("bd bd cp")),
        box Rev(box Sound(box mini_notation::parse_pattern("bd bd cp"))),
        box Rational::from((1,4)),
    );
    */
   let pattern = Sometimes(box Sound(box mini_notation::parse_pattern("bd bd bd bd bd")), box Sound(box mini_notation::parse_pattern("cp cp cp")), box 0.5);

    //let pattern = Sound(box mini_notation::parse_pattern("bd bd cp cp bd cp"));
    let events = pattern.query(Arc { start: 0.into(), stop: 12.into() });
    let events:Vec<_> = events.into_iter().map(|e:Event<ControlMap>| e.value).collect();
    println!("{:?}", events);
    println!("Pattern: {:#?}", pattern);
    player.set_pattern("d1", pattern);
    player.start_playback();
    std::thread::sleep(std::time::Duration::from_millis(12000));
}
