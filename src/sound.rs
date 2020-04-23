use {
    std::{
        collections::HashMap,
    },

    rodio::{Source, Device},
};

pub struct Player {
    device: Device,
    samples: SampleBank,
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
            device,
            samples
        }
    }

    pub fn play_sample(&self, sample: &str) {
        if let Some(sample) = self.samples.0.get(sample) {
            let sound = rodio::Decoder::new(std::io::Cursor::new(sample[0].clone())).unwrap();
            rodio::play_raw(&self.device, sound.convert_samples());
        }
    }
}
