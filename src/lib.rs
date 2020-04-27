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
pub type Pattern<A> = std::sync::Arc<dyn Fn(Arc) -> Vec<Event<A>> + Send + Sync>;

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

impl<A> Event<A> {
    pub fn whole_or_part(&self) -> Arc {
        if let Some(a) = self.whole {
            a
        } else {
            self.part
        }
    }
}

#[macro_export]
macro_rules! pattern {
    ($inner:expr) => {
        std::sync::Arc::new($inner)
    }
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

pub fn sound(p: Pattern<String>) -> Pattern<ControlMap> {
    pattern!(move |arc| {
        p(arc).into_iter().map(|e: Event<String>| {
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
    })
}
pub fn pan(p: Pattern<f32>) -> Pattern<ControlMap> {
    pattern!(move |arc| {
        p(arc).into_iter().map(|e| {
            let mut m = ControlMap(HashMap::new());
            m.0.insert("pan".to_string(), Value::Float(e.value));
            Event {
                whole: e.whole,
                part: e.part,
                value: m,
            }
        }).collect()
    })
}

pub fn apply_from_left<A: 'static + Clone, B: 'static + Clone>(f: fn(A, B) -> A, lhs: Pattern<A>, rhs: Pattern<B>) -> Pattern<A> {
    pattern!(move |arc| {
        let mut lhs:Vec<Event<A>> = lhs(arc);
        let mut rhs = rhs(arc);
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
                    value: f(current_l.value.clone(), current_r.value.clone()),
                });
            }
        }
        results
    })
}

pub fn jux_by(n: Pattern<f32>, f: fn(Pattern<ControlMap>) -> Pattern<ControlMap>, p: Pattern<ControlMap>) -> Pattern<ControlMap> {
    stack(vec![
        apply_from_left(
            |state, pan| {
                let mut state = state.clone();
                state.0.insert("pan".to_string(), Value::Float(pan));
                state
            },
            f(p.clone()),
            n.clone()
        ),
        apply_from_left(
            |state, pan| {
                let mut state = state.clone();
                state.0.insert("pan".to_string(), Value::Float(1.0-pan));
                state
            },
            p,
            n
        ),
    ])
}

pub fn sect(a: Arc, b: Arc) -> Arc {
    Arc {
        start: a.start.min(b.start),
        stop: a.stop.max(b.stop),
    }
}

pub fn sub_arc(a: Arc, b: Arc) -> Option<Arc> {
    let c = sect(a, b);
    if c.start == c.stop && c.start == a.stop && a.start < a.stop {
        None
    } else if c.start == c.stop && c.start == b.stop && b.start < b.stop {
        None
    } else if c.start <= c.stop {
        Some(c)
    } else {
        None
    }
}

pub fn inner_join<A: 'static>(p: Pattern<Pattern<A>>) -> Pattern<A> {
    pattern!(move |arc|
        p(arc).into_iter().filter_map(|e|
            sub_arc(arc, e.part).and_then(|p|
                sub_arc(p, arc)
            ).map(|arc|
                (e.value)(arc)
            )
        ).flatten().collect()
    )
}

pub fn superimpose<A: 'static>(f: impl Fn(&Pattern<A>) -> Pattern<A>, p: Pattern<A>) -> Pattern<A> {
    let np = f(&p);
    stack(vec![
        p,
        np
    ])
}

fn with_result_arc<A: 'static>(f: impl Fn(Arc)->Arc + Send + Sync + 'static, p: Pattern<A>) -> Pattern<A> {
    pattern!(move |arc| {
        p(arc).into_iter().map(|e| Event {
            whole: e.whole.map(|w| f(w)),
            part: f(e.part),
            value: e.value,
        }).collect()
    })
}

fn with_result_time<A: 'static>(f: impl Fn(Time) -> Time + Send + Sync + 'static, p: Pattern<A>) -> Pattern<A> {
    with_result_arc(move |arc| Arc { start: f(arc.start), stop: f(arc.stop) }, p)
}

fn with_query_arc<A: 'static>(f: impl Fn(Arc)->Arc + Send + Sync + 'static, p: Pattern<A>) -> Pattern<A> {
    pattern!(move |arc| {
        let arc = f(arc);
        p(arc)
    })
}

fn with_query_time<A: 'static>(f: impl Fn(Time) -> Time + Send + Sync + 'static, p: Pattern<A>) -> Pattern<A> {
    with_query_arc(move |arc:Arc| Arc { start: f(arc.start), stop: f(arc.stop) }, p)
}

pub fn rot_l<A: 'static>(t: Time, p: Pattern<A>) -> Pattern<A> {
    with_result_time(move |ot| ot - t, with_query_time(move |ot| t + ot, p))
}
pub fn rot_r<A: 'static>(t: Time, p: Pattern<A>) -> Pattern<A> {
    rot_l(-t, p)
}

fn _off<A: 'static>(t: Time, f: impl Fn(&Pattern<A>) -> Pattern<A> + Send + Sync + 'static, p: Pattern<A>) -> Pattern<A> {
    superimpose(move |pp| f(&rot_r(t, pp.clone())), p)
}

pub fn off<A: 'static>(tp: Pattern<Time>, f: impl Fn(&Pattern<A>) -> Pattern<A> + Send + Sync + 'static + Clone, p: Pattern<A>) -> Pattern<A> {
    inner_join(pattern!(move |arc| {
        let p = p.clone();
        let f = f.clone();
        tp(arc).into_iter().map(move |e| {
            Event {
                whole: e.whole,
                part: e.part,
                value: _off(e.value, f.clone(), p.clone())
            }
        }).collect()
    }))
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

pub fn overlay<A: 'static>(a: Pattern<A>, b: Pattern<A>) -> Pattern<A> {
    pattern!(move |arc| {
        let mut events:Vec<_> = a(arc).into_iter().chain(b(arc).into_iter()).collect();
        events.sort_by_key(|e| e.part.start);
        events
    })
}

fn t_param<A: 'static, T1: 'static, T2: 'static + Clone + Send + Sync>(f: impl Fn(T1, T2) -> Pattern<A> + Send + Sync +'static, tv: Pattern<T1>, p: T2) -> Pattern<A> {
    inner_join(pattern!(move |arc| {
        tv(arc).into_iter().map(|e| Event {
            whole: e.whole,
            part: e.part,
            value: f(e.value, p.clone())
        }).collect()
    }))
}

fn _degrade_by<A: 'static>(prob: f32, p: Pattern<A>) -> Pattern<A> {
    pattern!(move |arc| {
        p(arc).into_iter().filter(move |e| {
            let draw = time_rand((arc.start + arc.stop)/2isize);
            draw > prob
        }).collect()
    })
}

pub fn degrade_by<A: 'static>(prob: Pattern<f32>, p: Pattern<A>) -> Pattern<A> {
    t_param(|prob, p| {
        _degrade_by(prob, p)
    }, prob, p)
}

fn _undegrade_by<A: 'static>(prob: f32, p: Pattern<A>) -> Pattern<A> {
    pattern!(move |arc| {
        p(arc).into_iter().filter(|e| {
            let draw = time_rand((arc.start + arc.stop)/2isize);
            draw <= prob
        }).collect()
    })
}

pub fn undegrade_by<A: 'static>(prob: Pattern<f32>, p: Pattern<A>) -> Pattern<A> {
    t_param(|prob, p| {
        _undegrade_by(prob, p)
    }, prob, p)
}

pub fn filter_values<A: 'static>(f: impl Fn(&A) -> bool + Send + Sync + 'static, p: Pattern<A>) -> Pattern<A> {
    pattern!(move |arc| p(arc).into_iter().filter(|e| f(&e.value)).collect())
}

pub fn sometimes_by<A: 'static>(x: Pattern<f32>, f: impl Fn(Pattern<A>) -> Pattern<A> + Send + Sync, p: Pattern<A>) -> Pattern<A> {
    overlay(degrade_by(x.clone(), p.clone()), undegrade_by(x, f(p)))
}


pub fn unit<A: Clone + Sync + Send + 'static>(v: A) -> Pattern<A> {
    pattern!(move |arc| {
        arc_cycles_zw(arc).into_iter().filter_map(|a| {
            if arc.start.is_integer() {
                Some(Event {
                    whole: Some(Arc { start: a.start, stop: a.start + 1 }),
                    part: Arc { start: a.start, stop: (a.start + 1).min(a.stop) },
                    value: v.clone(),
                })
            } else { None }
        }).collect()
    })
}

pub fn fast<A: 'static>(r: Time, p: Pattern<A>) -> Pattern<A> {
    pattern!(move |arc| {
        if r == 0.into() {
            vec![]
        } else if r < 0.into() {
            todo!()
            //rev(arc, |arc| fast(arc, -r, p))
        } else {
            let arc = Arc {
                start: arc.start * r,
                stop: arc.stop * r,
            };
            p(arc).into_iter().map(|e| {
                Event {
                    whole: e.whole.map(|w| Arc {
                        start: w.start / r,
                        stop: w.stop / r,
                    }),
                    part: Arc {
                        start: e.part.start / r,
                        stop: e.part.stop / r,
                    },
                    value: e.value,
                }
            }).collect()
        }
    })
}

pub fn stack<A: 'static>(ps: Vec<Pattern<A>>) -> Pattern<A> {
    std::sync::Arc::new(move |arc| {
        let mut results = vec![];
        for p in &ps {
            results.extend(p(arc).into_iter());
        }
        results.sort_by_key(|e| e.part.start);
        results
    })
}

pub fn cat<A: 'static>(ps: Vec<Pattern<A>>) -> Pattern<A> {
    let n = ps.len();
    std::sync::Arc::new(move |arc: Arc| {
        let f = |arc: Arc| {
            let cyc = arc.start.floor();
            let mut i = cyc % n as isize;
            while i < 0isize.into() {
                i += n as isize;
            }
            let offset = cyc - ((cyc - i) / n as isize).floor();
            let arc = Arc {
                start: arc.start - offset,
                stop: arc.stop - offset
            };
            (ps[i.to_integer() as usize])(arc).into_iter().map(move |e| Event {
                whole: e.whole.map(|w| Arc {
                    start: w.start + offset,
                    stop: w.stop + offset
                }),
                part: Arc {
                    start: e.part.start + offset,
                    stop: e.part.stop + offset,
                },
                value: e.value
            })
        };
        arc_cycles_zw(arc).into_iter().flat_map(|arc| f(arc)).collect()
    })
}

pub fn filter_when<A: 'static>(test: impl Fn(Time) -> bool + Clone + Send + Sync + 'static, p: Pattern<A>) -> Pattern<A> {
    pattern!(move |arc| {
        let test = test.clone();
        p(arc).into_iter().filter(move |e| test(e.whole_or_part().start)).collect()
    })
}

fn sam(t: Time) -> Time {
    t.floor()
}

fn cycle_pos(t: Time) -> Time {
    t - sam(t)
}

pub fn within<A: 'static>(a: Arc, f: impl Fn(Pattern<A>)->Pattern<A> + Send + Sync + 'static, p: Pattern<A>) -> Pattern<A> {
    stack(vec![
       filter_when(move |t| {
           let cp = cycle_pos(t);
           cp >= a.start && cp < a.stop
       }, f(p.clone())),
       filter_when(move |t| {
           let cp = cycle_pos(t);
           !(cp >= a.start && cp < a.stop)
       }, p),
    ])
}

pub fn chunk<A: 'static>(n: usize, f: impl Fn(Pattern<A>) -> Pattern<A> + Clone + Send + Sync + 'static, p: Pattern<A>) -> Pattern<A> {
    cat((0..n-1).map(move |i| {
        within(Arc { start: (i as isize % n as isize).into(), stop: ((i+1) as isize % n as isize).into() }, f.clone(), p.clone())
    }).collect()
   )
}

fn split_queries<A: 'static>(p: Pattern<A>) -> Pattern<A> {
    pattern!(move |arc| {
        arc_cycles_zw(arc).into_iter().flat_map(|arc| {
            p(arc)
        }).collect()
    })
}

pub fn rev<A: 'static>(p: Pattern<A>) -> Pattern<A> {
    fn make_whole_relative<A>(e: Event<A>) -> Event<A> {
        if e.whole.is_none() {
            e
        } else {
            Event {
                whole: Some(Arc { start: e.part.start-e.whole.unwrap().start, stop: e.whole.unwrap().stop-e.part.stop }),
                part: e.part,
                value: e.value,
            }
        }
    }
    fn make_whole_absolute<A>(e: Event<A>) -> Event<A> {
        if e.whole.is_none() {
            e
        } else {
            Event {
                whole: Some(Arc { start: e.part.start-e.whole.unwrap().stop, stop: e.part.stop+e.whole.unwrap().start }),
                part: e.part,
                value: e.value
            }
        }
    }
    fn mid_cycle(a: Arc) -> Time {
        sam(a.start) + Rational::from((1,2))
    }
    fn map_parts<A>(f: impl Fn(Arc) -> Arc, es: Vec<Event<A>>) -> Vec<Event<A>> {
        es.into_iter().map(|e| Event {
            whole: e.whole,
            part: f(e.part),
            value: e.value
        }).collect()
    }
    fn mirror_arc(mid: Time, a: Arc) -> Arc {
        Arc {
            start: mid - (a.stop-mid),
            stop: mid + (mid-a.start)
        }
    }

    split_queries(pattern!(move |arc| {
        let na = mirror_arc(mid_cycle(arc), arc);
        let es:Vec<_> = p(na).into_iter().map(make_whole_relative).collect();
        let es = map_parts(|a| mirror_arc(mid_cycle(arc), a), es);
        es.into_iter().map(make_whole_absolute).collect()
    }))
}

pub fn id<A: 'static>(p: Pattern<A>) -> Pattern<A> {
    p
}
