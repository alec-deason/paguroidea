#![feature(box_syntax)]
use std::{
    collections::HashMap,
    io::Read,
    fs::File, io::BufReader,
};
use num::rational::Rational;

use paguroidea::*;

fn main() {
    println!("{:?}", (mini_notation::parse_pattern("bd"))(Arc { start: 0.into(), stop: 3.into()}));

    let mut samples = sound::SampleBank::new();
    samples.add_sample_sets_from_dir("/home/alec/.local/share/SuperCollider/downloaded-quarks/Dirt-Samples/");
    samples.add_sample_sets_from_dir("/home/alec/tidal/samples-extra/");
    let player = sound::Player::new(samples);

    let pattern = sound(mini_notation::parse_pattern("<hi:1 hi*2> <~ hi:3 lo:2*2 hi*2> <~ lo:3*2 hi> <lo:1 lo:3*2>"));
    let pattern = jux_by(unit(1.0), |p| off(unit((1,4).into()), |p| sometimes_by(unit(0.75), |p| chunk(2, |p| rev(p.clone()), p), p.clone()), p), pattern);
    player.set_pattern("d1", pattern);
    player.start_playback();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
