#![feature(box_syntax)]
use std::{
    convert::TryInto,
    collections::HashMap,
};
use num::rational::Rational;
#[macro_use]
extern crate pest_derive;

pub mod mini_notation;
pub mod sound;

pub type Time = Rational;

#[derive(Copy, Clone, Debug)]
pub struct Arc {
    pub start: Rational,
    pub stop: Rational,
}

pub struct Event<A> {
    pub whole: Option<Arc>,
    pub part: Arc,
    pub value: A,
}


impl<A: std::fmt::Debug> std::fmt::Debug for Event<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("whole", &self.whole)
            .field("part", &self.part)
            .field("value", &self.value)
            .finish()
    }
}

pub trait Pattern<A>: std::fmt::Debug {
    fn query(&self, arc: Arc) -> Vec<Event<A>>;
}

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Integer(isize),
}

impl TryInto<String> for Value {
    type Error = ();
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Value::String(v) => Ok(v),
            _ => Err(())
        }
    }
}

impl TryInto<isize> for Value {
    type Error = ();
    fn try_into(self) -> Result<isize, Self::Error> {
        match self {
            Value::Integer(v) => Ok(v),
            _ => Err(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct ControlMap(pub HashMap<String, Value>);

impl<A: Clone + std::fmt::Debug> Pattern<A> for A {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        arc_cycles_zw(arc).into_iter().filter_map(|a| {
            if arc.start.is_integer() {
                Some(Event {
                    whole: Some(Arc { start: a.start, stop: a.start + 1 }),
                    part: Arc { start: a.start, stop: (a.start + 1).min(a.stop) },
                    value: self.clone(),
                })
            } else { None }
        }).collect()
    }
}
impl<A: Clone + std::fmt::Debug> Pattern<A> for Box<dyn Pattern<A> + Send> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        self.as_ref().query(arc)
    }
}

#[derive(Debug)]
pub struct Fast<A> where A:Send+std::fmt::Debug {
    pub speed: Rational,
    pub pattern: Box<dyn Pattern<A> + Send>,
}
impl<A: Send+std::fmt::Debug> Pattern<A> for Fast<A> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        let arc = Arc {
            start: arc.start * self.speed,
            stop: arc.stop * self.speed,
        };
        let mut result = self.pattern.query(arc);
        for e in &mut result {
            e.part.start = e.part.start / self.speed;
            e.part.stop = e.part.stop / self.speed;
            e.whole.map(|mut w| {
                w.start = w.start / self.speed;
                w.stop = w.stop / self.speed;
            });
        }
        result
    }
}

#[derive(Debug)]
pub struct Cat<A> where A:Send+std::fmt::Debug {
    pub subpatterns: Vec<Box<dyn Pattern<A> + Send>>,
}

fn arc_cycles(arc: Arc) -> Vec<Arc> {
    if arc.start >= arc.stop {
        vec![]
    } else if arc.start.floor() == arc.stop.floor() {
        vec![arc]
    } else {
        let mut result = vec![Arc { start: arc.start, stop: arc.start.floor()+1 }];
        result.extend(arc_cycles(Arc { start: arc.start.floor()+1, stop: arc.stop }));
        result
    }
}

fn arc_cycles_zw(arc: Arc) -> Vec<Arc> {
    if arc.start == arc.stop {
        vec![arc]
    } else {
        arc_cycles(arc)
    }
}

impl<A: Send+std::fmt::Debug> Pattern<A> for Cat<A> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        arc_cycles_zw(arc).into_iter().flat_map(|a| {
            let n:Rational = (self.subpatterns.len() as isize).into();
            let cyc = a.start.floor();
            let i: Rational = cyc % n;
            let offset: Rational = (cyc - ((cyc - i) / n)).floor();
            let i:usize = i.to_integer() as usize;
            let a = Arc {
                start: a.start - offset,
                stop: a.stop - offset,
            };
            let mut result:Vec<Event<A>> = self.subpatterns[i].query(a);
            for e in &mut result {
                e.part.start += offset;
                e.part.stop += offset;
                e.whole.map(|mut w| {
                    w.start += offset;
                    w.stop += offset;
                });
            }
            result
        }).collect()
    }
}

#[derive(Debug)]
pub struct Sound(pub Box<dyn Pattern<String> + Send>);
impl Pattern<ControlMap> for Sound {
    fn query(&self, arc: Arc) -> Vec<Event<ControlMap>> {
        self.0.query(arc).into_iter().map(|e| {
            let mut m = ControlMap(HashMap::new());
            m.0.insert("s".to_string(), Value::String(e.value));
            m.0.insert("n".to_string(), Value::Integer(0));
            Event {
                whole: e.whole,
                part: e.part,
                value: m,
            }
        }).collect()
    }
}
