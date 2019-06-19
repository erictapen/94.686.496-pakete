use std::fs::File;

// For reading and opening files
use std::io::BufWriter;
use std::path::PathBuf;
// To use encoder.set()
use png::HasParameters;


use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use cachable::CachablePNG;

use pcarp::Capture;
use std::time::UNIX_EPOCH;

#[derive(Hash)]
pub struct Datagram {
    pub packet_path: PathBuf,
    pub gray_value: u8,
}

impl CachablePNG for Datagram {
    fn png_cached(&self) -> Result<PathBuf, &str> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash: u64 = hasher.finish();
        let path = PathBuf::from(&format!("./cache/{}.png", &hash.to_string()));
        if path.exists() {
            println!("Cache hit for {}", &path.to_str().unwrap());
            Ok(path)
        } else {
            println!("Cache miss for {}", &path.to_str().unwrap());
            self.png_uncached(path)
        }
    }
    fn png_uncached(&self, path: PathBuf) -> Result<PathBuf, &str> {
        let file = match File::open(&self.packet_path) {
            Ok(f) => f,
            Err(_) => {
                panic!("Could not find {}", &self.packet_path.to_str().unwrap());
            },
        };
        let mut pcap = Capture::new(file).unwrap();
        let pkt = pcap.next().unwrap().unwrap();
        let _ts = pkt.timestamp.unwrap_or(UNIX_EPOCH);

        let packet_size: usize = pkt.data.len();

        let lines: usize = packet_size / (32) + 1;
        let pixel: usize = lines * 32 * 8;
        println!("{} lines, {} pixel", lines, pixel);

        // actual image data
        let mut image: Vec<u8> = vec![255; pixel * 4];

        // build image
        for i in 0..pkt.data.len() {
            for j in 0..8 {
                if pkt.data[i] & (1 << 7 - j) != 0 {
                    for rgb_offset in 0..3 {
                        image[((i * 8) + j) * 4 + rgb_offset] = self.gray_value;
                    }
                }
            }
        }

        println!("Datagram will have {} lines", lines);

        let file = File::create(path.clone()).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, 32 * 8, lines as u32);
        // Width is 2 pixels and height is 1.
        encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&image).unwrap(); // Save

        Ok(path)
    }
}

/* unused code
use svg::node::element::Group;
use svg::node::element::Rectangle;
use svg::node::Node;
fn svg_from_data(data: Vec<u8>, pixel_width: f64, line_width: u64) -> Group {
    let mut g = Group::new();
    for i in 0..data.len() {
        g.append(
            Rectangle::new()
                .set("x", (i as u64 % line_width) as f64 * pixel_width)
                .set("y", (i as u64 / line_width) as f64 * pixel_width)
                .set("width", pixel_width)
                .set("height", pixel_width)
                .set(
                    "style",
                    format!(
                        "fill:rgb({0},{0},{0});stroke:rgb({0},{0},{0});stroke-width:0.0",
                        data[i]
                    ),
                ),
        );
    }
    return g;
}
*/

