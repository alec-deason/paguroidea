#![feature(box_syntax)]
use std::{
    collections::HashMap,
    io::Read,
    fs::File, io::BufReader,
    env::args,
};
use num::rational::Rational;

use paguroidea::*;

fn main() {
    let mut samples = sound::SampleBank::new();
    for dir in args().skip(1) {
        samples.add_sample_sets_from_dir(dir);
    }
    let player = sound::Player::new(samples);

    let pattern = sound(mini_notation::parse_pattern("<tabla:1 tabla*2> <~ tabla:3 drum:2*2 tabla*2> <~ drum:3*2 tabla> <drum:1 drum:3*2>"));
    let pattern = jux_by(unit(1.0), |p| off(unit((1,4).into()), |p| sometimes_by(unit(0.75), |p| chunk(2, |p| rev(p.clone()), p), p.clone()), p), pattern);
    player.set_pattern("d1", pattern);
    player.start_playback();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
