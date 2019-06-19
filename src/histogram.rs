extern crate bincode;
extern crate csv;
extern crate png;
extern crate xz2;

use std::fs::File;

use std::time::{Duration, SystemTime};

// For reading and opening files
use std::io::BufWriter;
use std::path::{Path, PathBuf};
// To use encoder.set()
use png::HasParameters;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bincode::{deserialize, serialize};

use cachable::{CachableData, CachablePNG};

use std::process::Command;

#[derive(Debug, Deserialize)]
struct CSVLine {
    time_string: String,
    len: usize,
}

// that part of histogram, that, if changed, would require the expensive rereading of all datafiles
// again
#[derive(Hash)]
pub struct HistogramData {
    pub filter: Option<String>,
    pub filter_description: Option<String>,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub width: usize,
    pub do_pps: bool,
    pub data_file: &'static Path,
}

impl CachableData for HistogramData {
    fn data_cached(&self) -> Vec<u64> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash: u64 = hasher.finish();
        let cache_path = PathBuf::from(&format!(
            "./cache/{}-{}.vec",
            &hash.to_string(),
            self.filter_description
                .clone()
                .unwrap_or("unnamed".to_string()),
        ));
        if cache_path.exists() {
            println!(
                "Cache hit for {} for data_file {}",
                &cache_path.to_str().unwrap(),
                &self.data_file.to_str().unwrap()
            );
            deserialize(&std::fs::read(&cache_path).unwrap()).unwrap()
        } else {
            use std::io::Write;
            println!("Cache miss for {}", &cache_path.to_str().unwrap());
            let res: Vec<u64> = self.data_uncached();
            let bytes: Vec<u8> = serialize(&res).unwrap();
            let mut file: File = File::create(cache_path).unwrap();
            file.write_all(&bytes).unwrap();
            res
        }
    }
    /// Read one data file, filter it optionally, go through every packet and sorts
    /// them into a bucket depending on its timestamp.
    fn data_uncached(&self) -> Vec<u64> {
        let mut histo_data: Vec<u64> = vec![0; self.width];
        let mut count: u32 = 0;
        let mut byte_sum: u64 = 0;

        // path to CSV file, that is the result of the filter application
        let filtered_pcap: PathBuf = {
            let f = self.filter.clone().unwrap_or("".to_string());
            std::fs::create_dir("./tmp").ok();
            let mut hasher = DefaultHasher::new();
            self.data_file.hash(&mut hasher);
            let hash: u64 = hasher.finish();
            let res_path: PathBuf = PathBuf::from(format!(
                "./tmp/filter-{}-{}.csv",
                self.filter_description
                    .clone()
                    .unwrap_or("unnamed".to_string()),
                hash.to_string()
            ));
            let cmd = format!(
                    "tshark -T fields -E separator='|' -e frame.time_epoch -e frame.len -r '{orig}' '{filter}' > {new}",
                    orig = self.data_file.display(),
                    new = res_path.display(),
                    filter = f,
                );
            println!("Running command: {}", &cmd);
            let status = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .status()
                .expect("Faiil");
            assert!(status.success(), "Tshark filtering failed!");
            res_path
        };

        println!(
            "Reading CSV with filter results of {} ...",
            &self.data_file.display()
        );
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'|')
            .has_headers(false)
            .from_path(&filtered_pcap)
            .unwrap();

        // sort packets into buckets in histo_data
        for result in rdr.deserialize() {
            let packet: CSVLine = result.unwrap();
            count += 1;
            // println!("{}", count);
            byte_sum += packet.len as u64;
            let timestamp = {
                let split: Vec<&str> = packet.time_string.splitn(2, ".").collect();
                SystemTime::UNIX_EPOCH
                    + Duration::new(
                        // std::u64::from_str_radix(s, 10),
                        // std::u32::from_str_radix(ns, 10),
                        split[0].parse().unwrap(),
                        split[1].parse().unwrap(),
                    )
            };
            let index = match get_bucket(self.width, self.start_time, self.end_time, timestamp) {
                Some(i) => i,
                None => continue,
            };
            histo_data[index] += if self.do_pps {
                1
            } else {
                packet.len as u64 * 8 // bits transfered
            };
        }
        // delete the data file if it was temporary
        match self.filter {
            Some(_) => {
                assert!(
                    filtered_pcap.starts_with("./tmp"),
                    "filtered_cap is not in tmp???"
                );
                std::fs::remove_file(filtered_pcap).unwrap();
            }
            // DO NOT REMOVE THE ORIGINAL DATA FILE
            None => {}
        }
        println!("{} Bytes, {} Packets in whole dataset.", byte_sum, count);
        histo_data
    }
}

pub struct Histogram {
    pub do_log: bool,
    pub yscale: f64,
    pub data: Vec<Box<HistogramData>>,
    pub color: (u8, u8, u8),
}

impl Hash for Histogram {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.do_log.hash(state);
        // f64 is not hashable m(
        self.yscale.to_string().hash(state);
        self.data.hash(state);
        self.color.hash(state);
        // implementation
        "20".hash(state);
    }
}

impl CachablePNG for Histogram {
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
        let log_base = 10.0;
        let log_min_display = 3.0; // minimum exponent where to start displaying
        let width = if self.data.is_empty() { 100 } else { self.data[0].width };
        let mut histo_data: Vec<i64> = vec![0; width];
        for d in &self.data {
            let data_cached: Vec<u64> = d.data_cached();
            for i in 0..histo_data.len() {
                histo_data[i] += data_cached[i] as i64;
            }
        }

        // find out bucket with min/max value
        let mut max_value: i64 = 0;
        let mut min_value: i64 = i64::max_value();
        for b in &histo_data {
            if max_value < *b {
                max_value = *b;
            }
            if min_value > *b && b > &0 {
                min_value = *b;
            }
        }

        let height: usize = if self.do_log {
            (((max_value as f64).log(log_base) - log_min_display) * self.yscale) as usize + 1
        } else {
            (max_value as f64 * self.yscale) as usize + 1
        };

        if max_value == 0 {
            println!("No bucket has any values. This will be an empty histogram");
        } else {
            println!(
                "Timeline will consist of {} buckets, with min {}, max {} packets",
                histo_data.len(),
                min_value,
                max_value
            );
            assert!(
                height <= 661,
                format!("height is greater than 416, but {}", height)
            );
        }
        let height = 661;

        println!(
            "Histogram will have dimensions {}x{} and color {:?}",
            width, height, self.color,
        );

        let mut histo: Vec<u8> = vec![255; 4 * width * height];
        for i in 0..histo_data.len() {
            // println!("{}", histo_data[i]);
            // distance in pixels from botton to top of the bucket
            let value: i64 = if self.do_log {
                if histo_data[i] > 0 {
                    (((histo_data[i] as f64).log(log_base) - log_min_display) * self.yscale) as i64
                } else {
                    continue;
                }
            } else {
                (histo_data[i] as f64 * self.yscale) as i64
            };
            if value < 0 {
                continue;
            }
            for y in 0..(value as usize) {
                let index = (i + (width * (height - y - 1))) * 4;
                histo[index + 0] = self.color.0;
                histo[index + 1] = self.color.1;
                histo[index + 2] = self.color.2;
            }
        }

        println!("{:?}", &path);
        let file = File::create(&path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, width as u32, height as u32);

        encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&histo).unwrap(); // Save
        Ok(path.to_path_buf())
    }
}

fn get_bucket(width: usize, first: SystemTime, last: SystemTime, t: SystemTime) -> Option<usize> {
    let part = match t.duration_since(first) {
        Ok(r) => r.as_nanos() as f64,
        Err(_) => return None,
    };
    let whole = last.duration_since(first).unwrap().as_nanos() as f64;
    let ratio = part / whole;
    let res = (width as f64 * ratio) as usize;
    if res >= width {
        // println!("{}/{}: {} {}", part, whole, ratio, res);
        Some(width - 1)
    } else {
        Some(res)
    }
}
