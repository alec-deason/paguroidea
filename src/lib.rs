#![feature(box_syntax)]
use std::{
    convert::TryInto,
    collections::HashMap,
};
use rand::{rngs::StdRng, SeedableRng, Rng};
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

pub trait Pattern<A>: std::fmt::Debug + PatternClone<A> + Send {
    fn query(&self, arc: Arc) -> Vec<Event<A>>;
}

pub trait PatternClone<A> {
        fn clone_box(&self) -> Box<Pattern<A>>;
}

impl<T, A> PatternClone<A> for T
where
    T: 'static + Pattern<A> + Clone,
{
    fn clone_box(&self) -> Box<dyn Pattern<A>> {
        Box::new(self.clone())
    }
}

impl<A> Clone for Box<Pattern<A>> {
    fn clone(&self) -> Box<dyn Pattern<A>> {
        self.as_ref().clone_box()
    }
}

impl<A: 'static> Pattern<A> for Box<dyn Pattern<A>> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        self.as_ref().query(arc)
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Integer(isize),
    Float(f32),
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

impl TryInto<f32> for Value {
    type Error = ();
    fn try_into(self) -> Result<f32, Self::Error> {
        match self {
            Value::Float(v) => Ok(v),
            _ => Err(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct ControlMap(pub HashMap<String, Value>);

impl<A: Send + Clone + std::fmt::Debug+'static> Pattern<A> for A {
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

#[derive(Clone, Debug)]
pub struct Fast<A> where A:Send+std::fmt::Debug {
    pub speed: Rational,
    pub pattern: Box<dyn Pattern<A>>,
}
impl<A: Send+std::fmt::Debug+Clone+'static> Pattern<A> for Fast<A> {
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

#[derive(Clone, Debug)]
pub struct Cat<A> where A:Send+std::fmt::Debug {
    pub subpatterns: Vec<Box<dyn Pattern<A>>>,
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

impl<A: Send+std::fmt::Debug+Clone+'static> Pattern<A> for Cat<A> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        arc_cycles_zw(arc).into_iter().flat_map(|a| {
            let n:Rational = (self.subpatterns.len() as isize).into();
            let cyc = a.start.floor();
            let mut i: Rational = cyc % n;
            while i < 0.into() {
                i += n;
            }
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

#[derive(Clone, Debug)]
pub struct Sound(pub Box<dyn Pattern<String>>);
impl Pattern<ControlMap> for Sound {
    fn query(&self, arc: Arc) -> Vec<Event<ControlMap>> {
        self.0.query(arc).into_iter().map(|e: Event<String>| {
            let mut m = ControlMap(HashMap::new());
            let parts:Vec<_> = e.value.split(":").collect();
            m.0.insert("s".to_string(), Value::String(parts[0].to_string()));
            if parts.len() > 1 {
                m.0.insert("n".to_string(), Value::Integer(parts[1].parse().unwrap()));
            } else {
                m.0.insert("n".to_string(), Value::Integer(0));
            }
            Event {
                whole: e.whole,
                part: e.part,
                value: m,
            }
        }).collect()
    }
}

#[derive(Clone, Debug)]
pub struct Pan(pub Box<dyn Pattern<f32>>);
impl Pattern<ControlMap> for Pan {
    fn query(&self, arc: Arc) -> Vec<Event<ControlMap>> {
        self.0.query(arc).into_iter().map(|e| {
            let mut m = ControlMap(HashMap::new());
            m.0.insert("pan".to_string(), Value::Float(e.value));
            Event {
                whole: e.whole,
                part: e.part,
                value: m,
            }
        }).collect()
    }
}

#[derive(Clone)]
pub struct ApplyFromLeft<A, B> where A: 'static, B: 'static {
    f: fn(A, B) -> A,
    lhs: Box<dyn Pattern<A>>,
    rhs: Box<dyn Pattern<B>>,
}
impl<A: std::fmt::Debug, B> std::fmt::Debug for ApplyFromLeft<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplyFromLeft")
            .field("f", &"...")
            .field("lhs", &self.lhs)
            .field("rhs", &self.rhs)
            .finish()
    }
}
impl<A: Send+std::fmt::Debug+Clone+'static, B: Clone+'static+std::fmt::Debug> Pattern<A> for ApplyFromLeft<A, B> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        let mut lhs:Vec<Event<A>> = self.lhs.query(arc);
        let mut rhs = self.rhs.query(arc);
        let mut results = vec![];
        if lhs.len() == 0 || rhs.len() == 0 {
            return results;
        }

        let mut current_r:Option<Event<B>> = None;
        for current_l in &lhs {
            if current_r.is_some() {
                if current_r.as_ref().unwrap().part.start > current_l.part.stop {
                    current_r = None;
                }
            }
            while !rhs.is_empty() {
                if rhs[0].part.start > current_l.part.start {
                    break;
                }
                current_r = Some(rhs.remove(0));
            }
            if let Some(current_r) = current_r.as_ref() {
                results.push(Event {
                    whole: current_l.whole,
                    part: current_l.part,
                    value: (self.f)(current_l.value.clone(), current_r.value.clone()),
                });
            }
        }
        results
    }
}

pub fn jux_by(n: impl Pattern<f32>, f: impl Fn(&dyn Pattern<ControlMap>) -> Box<dyn Pattern<ControlMap>>, p: impl Pattern<ControlMap> + 'static) -> Box<dyn Pattern<ControlMap>> {
    box Stack(vec![
        box ApplyFromLeft {
            f: |state:ControlMap, pan| {
                let mut state = state.clone();
                state.0.insert("pan".to_string(), Value::Float(pan));
                state
            },
            lhs: p.clone_box(),
            rhs: n.clone_box(),
        },
        box ApplyFromLeft {
            f: |state:ControlMap, pan| {
                let mut state = state.clone();
                state.0.insert("pan".to_string(), Value::Float(1.0-pan));
                state
            },
            lhs: f(&p),
            rhs: n.clone_box(),
        },
    ])
}

#[derive(Clone, Debug)]
pub struct Off<A>(pub Box<dyn Pattern<A>>, pub Box<dyn Pattern<A>>, pub Box<dyn Pattern<Time>>) where A: std::fmt::Debug;
impl<A: Send+std::fmt::Debug + Clone + 'static> Pattern<A> for Off<A> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        let mut results = self.0.query(arc);
        let offsets: Vec<Event<Time>> = self.2.query(arc);
        for offset in offsets {
            let arc = Arc {
                start: offset.part.start + offset.value,
                stop: offset.part.stop + offset.value,
            };
            results.extend(self.1.query(arc).into_iter().map(|e|
                Event {
                    whole: e.whole.map(|w| Arc {
                        start: w.start - offset.value,
                        stop: w.stop - offset.value,
                    }),
                    part: Arc {
                        start: e.part.start - offset.value,
                        stop: e.part.stop - offset.value,
                    },
                    value: e.value,
                }
            ));
        }
        results.sort_by_key(|e| e.part.start);
        results
    }
}

#[derive(Clone, Debug)]
pub struct Stack<A>(pub Vec<Box<dyn Pattern<A>>>) where A: std::fmt::Debug;
impl<A: Send+std::fmt::Debug + Clone + 'static> Pattern<A> for Stack<A> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        let mut results:Vec<_> = self.0.iter().flat_map(
            |pattern|
            pattern.query(arc)
        ).collect();
        results.sort_by_key(|e| e.part.start);
        results
    }
}

#[derive(Clone, Debug)]
pub struct Rev<A>(pub Box<dyn Pattern<A>>) where A: std::fmt::Debug;
impl<A: Send+std::fmt::Debug + Clone + 'static> Pattern<A> for Rev<A> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        let mut current = arc.start;
        let mut results:Vec<Event<A>> = vec![];
        while current < arc.stop {
            let start = current;
            let stop = (current+1).min(arc.stop);
            current = stop;
            let mut mid:Rational = (1, 2).into();
            mid += start.floor();
            let (new_start, new_stop) = (mid - (stop-mid), (mid+(mid-start)));
            let a = Arc { start: new_start, stop: new_stop };
            results.extend(
                self.0.query(a).into_iter().map(|e| {
                    let (new_start, nev_stop) = (mid - (e.part.stop-mid), (mid+(mid-e.part.start)));
                    Event {
                        //FXME: I don't really understand what 'whole' is for and this is certainly not handling it corectly
                        whole: e.whole,
                        part: Arc {
                            start: new_start,
                            stop: new_stop,
                        },
                        value: e.value,
                    }
                })
            );
        }
        results
    }
}

fn time_rand(t: Time) -> f32 {
    let x = (t*t) / 1000000;
    //TODO: Uh. Understand what this does in a bunch of cases. It's probably pretty "random" sometimes
    // Also it obviously throws away information when the numbers are large but they normally won't be so maybe that doesn't matter. Also this is not crypto and not a place where pattern artifacts will be very visible.
    let mut seed = [0; 32];
    seed[0] = *x.numer() as u8;
    seed[1] = *x.denom() as u8;
    let mut rng:StdRng = StdRng::from_seed(seed);
    rng.gen()
}

//FIXME: this should take a f32 pattern not a static value
#[derive(Clone, Debug)]
pub struct Sometimes<A>(pub Box<dyn Pattern<A>>, pub Box<dyn Pattern<A>>, pub f32) where A: std::fmt::Debug;
impl<A: Send+std::fmt::Debug + Clone + 'static> Pattern<A> for Sometimes<A> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        let mut results: Vec<_> = self.0.query(arc).into_iter()
            .filter(|e| time_rand(e.part.start) > self.2).chain(
                self.1.query(arc).into_iter()
                .filter(|e| time_rand(e.part.start) <= self.2)).collect();
        results.sort_by_key(|e| e.part.start);
        results
    }
}
