use {
    std::{
        convert::TryInto,
        collections::HashMap,
        sync::Mutex,
    },

    num::Rational,
    rodio::{Source, Device},

    crate::{Event, Pattern, ControlMap, Arc},
};

pub struct Player {
    inner: std::sync::Arc<Mutex<InnerPlayer>>,
}

struct InnerPlayer {
    device: Device,
    samples: SampleBank,
    patterns: HashMap<String, Box<dyn Pattern<ControlMap> + Send>>,
}
impl InnerPlayer {
    fn play_sample(&self, sample: &str, variation: usize) {
        if let Some(sample) = self.samples.0.get(sample) {
            let sound = rodio::Decoder::new(std::io::Cursor::new(sample[variation].clone())).unwrap();
            rodio::play_raw(&self.device, sound.convert_samples());
        }
    }
}

pub struct SampleBank(HashMap<String, Vec<std::sync::Arc<[u8]>>>);
impl SampleBank {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_sample_set(&mut self, name: impl AsRef<str>, samples: Vec<Vec<u8>>) {
        self.0.insert(
            name.as_ref().to_string(),
            samples.into_iter().map(|s| std::sync::Arc::from(s)).collect()
        );
    }
}


impl Player {
    pub fn new(samples: SampleBank) -> Self {
        let device = rodio::default_output_device().unwrap();

        Self {
            inner: std::sync::Arc::new(Mutex::new(InnerPlayer {
                device,
                samples: samples,
                patterns: HashMap::new(),
            }))
        }
    }

    pub fn set_pattern(&self, name: impl AsRef<str>, pattern: impl Pattern<ControlMap> + Send + 'static) {
        let patterns = &mut self.inner.lock().unwrap().patterns;
        patterns.insert(name.as_ref().to_string(), box pattern);
    }

    pub fn start_playback(&self) {
        let player = self.inner.clone();
        std::thread::spawn(move || {
            let pattern_sample_granularity: Rational = (1, 30).into();
            let mut current: Rational = 0.into();
            let mut pending_events:Vec<Event<ControlMap>> = vec![];
            loop {
                let next = current + pattern_sample_granularity;
                {
                    let player = player.lock().unwrap();
                    for pattern in player.patterns.values() {
                        //FIXME: I don't really understand why I need the as_ref here
                        pending_events.extend(pattern.as_ref().query(Arc {
                            start: current,
                            stop: next,
                        }));
                    }
                    pending_events.sort_by_key(|e| e.part.start);
                    println!("{:?}, {:?}", player.patterns.keys(), pending_events);
                }
                for event in pending_events.drain(..) {
                    let gap = ((event.part.start - current) * 1000).to_integer().max(0) as u64;
                    //FIXME: check that we didn't wake early
                    std::thread::sleep(std::time::Duration::from_millis(gap));
                    current = event.part.start;
                    let sample:String = event.value.0["s"].clone().try_into().unwrap();
                    let variation:isize = event.value.0["n"].clone().try_into().unwrap();
                    player.lock().unwrap().play_sample(&sample, variation as usize);
                }
                let gap = ((next - current) * 1000).to_integer().max(0) as u64;
                std::thread::sleep(std::time::Duration::from_millis(gap));
                current = next;
            }
        });
    }
}
