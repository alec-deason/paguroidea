use {
    std::{
        io::Read,
        collections::HashMap,
        fs::File, io::BufReader,
    },

    rodio::{Source, Device},
};

pub struct Player {
    device: Device,
    samples: HashMap<String, std::sync::Arc<[u8]>>,
}

impl Player {
    pub fn new() -> Self {
        let device = rodio::default_output_device().unwrap();
        let mut samples = HashMap::new();

        let mut file = File::open("/home/alec/.local/share/SuperCollider/downloaded-quarks/Dirt-Samples/bd/BT0A0A7.wav").unwrap();
        let mut data = vec![];
        file.read_to_end(&mut data);
        samples.insert("bd".to_string(), std::sync::Arc::from(data));

        let mut file = File::open("/home/alec/.local/share/SuperCollider/downloaded-quarks/Dirt-Samples/cp/HANDCLP0.wav").unwrap();
        let mut data = vec![];
        file.read_to_end(&mut data);
        samples.insert("cp".to_string(), std::sync::Arc::from(data));
        Self {
            device,
            samples
        }
    }

    pub fn play_sample(&self, sample: &str) {
        let sample = self.samples[sample].clone();
        let sound = rodio::Decoder::new(std::io::Cursor::new(sample)).unwrap();
        rodio::play_raw(&self.device, sound.convert_samples());
    }
}
