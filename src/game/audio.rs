//use rodio::Source;

pub fn test_audio() {
    /*
    let device = rodio::default_output_device().unwrap();
    let file = std::fs::File::open("test.ogg").unwrap();
    let s = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
    rodio::play_raw(&device, s.convert_samples());
    */
}

// TODO AudioEvent
// audio system needs to hold onto a rodio device?
// AudioDb
