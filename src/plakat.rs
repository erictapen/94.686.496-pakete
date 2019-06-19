use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use cachable::CachablePNG;

use std::boxed::Box;

#[allow(dead_code)]
pub struct Plakat {
    width_pixels: u32,
    height_pixels: u32,

    total_amount_of_packets: u64,
    pcap_files: Vec<File>,

    pub elements: HashMap<String, Box<CachablePNG>>,

    pub template_path: &'static Path,
}

impl Plakat {
    pub fn new(w: u32, h: u32) -> Plakat {
        Plakat {
            width_pixels: w,
            height_pixels: h,

            total_amount_of_packets: 0,
            pcap_files: Vec::new(),

            elements: HashMap::new(),
            template_path: &Path::new("./template.svg"),
        }
    }
}
