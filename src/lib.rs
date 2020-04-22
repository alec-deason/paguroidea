use num::rational::Rational;

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

pub trait Pattern<A> {
    fn query(&self, arc: Arc) -> Vec<Event<A>>;
}

impl<A: Clone> Pattern<A> for A {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        let start:isize = arc.start.ceil().to_integer();
        let stop:isize = arc.stop.floor().to_integer();
        (start..stop).map(|t| {
            Event {
                whole: Some(Arc { start: t.into(), stop: (t+ 1).into() }),
                part: Arc { start: t.into(), stop: (t + 1).min(stop).into() },
                value: self.clone(),
            }
        }).collect()
    }
}
impl<A: Clone> Pattern<A> for Box<dyn Pattern<A>> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        self.query(arc)
    }
}

pub struct Silence;
impl<A> Pattern<A> for Silence {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        vec![]
    }
}

pub struct Fast<A> {
    pub speed: Rational,
    pub pattern: Box<dyn Pattern<A>>,
}
impl<A> Pattern<A> for Fast<A> {
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

pub struct Cat<A> {
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

impl<A> Pattern<A> for Cat<A> {
    fn query(&self, arc: Arc) -> Vec<Event<A>> {
        arc_cycles_zw(arc).into_iter().flat_map(|a| {
            let n:Rational = (self.subpatterns.len() as isize).into();
            let cyc = a.start.floor();
            let i: Rational = cyc % n;
            let offset: Rational = cyc - ((cyc - i) / n);
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

pub struct FastCat<A> {
    pub subpatterns: Vec<Box<dyn Pattern<A>>>,
}
