extern crate bincode;
extern crate csv;
extern crate pcarp;
extern crate png;
extern crate svg;
extern crate xml;
extern crate xz2;
#[macro_use]
extern crate serde_derive;

mod cachable;
mod datagram;
mod histogram;
mod plakat;
mod template;

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use std::boxed::Box;

use histogram::{Histogram, HistogramData};

fn mk_histogram_data(
    data_file: &'static Path,
    filter: String,
    filter_description: String,
) -> Box<HistogramData> {
    Box::new(HistogramData {
        filter: Some(filter),
        filter_description: Some(filter_description),
        // Monday, January 14, 2019 12:00:00 AM GMT+01:00
        start_time: SystemTime::UNIX_EPOCH + Duration::new(1547420400, 0),
        // Sunday, January 20, 2019 11:59:59 PM GMT+01:00
        end_time: SystemTime::UNIX_EPOCH + Duration::new(1548025199, 999999999),
        width: 6000,
        do_pps: false,
        data_file: data_file,
    })
}

fn mk_histogram(
    data_files: &Vec<&'static Path>,
    filter: String,
    filter_description: String,
    color_str: String,
) -> Histogram {
    histogram::Histogram {
        do_log: true,
        // yscale: 0.0000001, // nolog
        yscale: 100.0, // with log
        color: (
            u8::from_str_radix(&color_str[..2], 16).unwrap(),
            u8::from_str_radix(&color_str[2..4], 16).unwrap(),
            u8::from_str_radix(&color_str[4..6], 16).unwrap(),
        ),
        data: {
            let mut v: Vec<Box<HistogramData>> = Vec::new();
            for p in data_files {
                v.push(mk_histogram_data(
                    p,
                    filter.clone(),
                    filter_description.clone(),
                ));
            }
            v
        },
    }
}

fn main() {
    // Network dumps of my laptop
    let data_files_laptop = vec![
        // Path::new("./path/to/your.pcapng"),
    ];
    // Network dumps of my smartphone
    let data_files_swift = vec![
        // Path::new("./path/to/your.pcapng"),
    ];
    let mut p = plakat::Plakat::new(7016, 9933);
    let datags = vec![
        ("first_packet", 0x66),
        ("last_packet", 0x66),
        ("dns-01", 0x00),
        ("dns-02", 0x00),
        ("dns-03", 0x00),
        ("dns-04", 0x00),
        ("dns-05", 0x00),
        ("dns-06", 0x00),
        ("dns-07", 0x00),
        ("dns-08", 0x00),
        ("dns-09", 0x00),
        ("dns-10", 0x00),
        ("dns-11", 0x00),
        ("dns-12", 0x00),
        ("dns-13", 0x00),
        ("dns-14", 0x00),
        ("dns-15", 0x00),
        ("dns-16", 0x00),
        ("dns-17", 0x00),
        ("dns-18", 0x00),
    ];
    for (d, c) in datags {
        let datag = datagram::Datagram {
            packet_path: PathBuf::from(format!("./raw_data/{}.pcapng", d)),
            gray_value: c,
        };
        p.elements
            .insert(format!("datag_{}", d.to_string()), Box::new(datag));
    }
    let histos: Vec<(&str, &str, &str)> = vec![
        // color, label, tcpdump filter
        ("000000", "none", ""),
        ("000000", "dns", "udp.port==53 || tcp.port==53"),
        ("000000", "http", "tcp.port==80"),
        ("000000", "https", "tcp.port==443"),
        ("000000", "udp", "udp"),
        ("000000", "dhcp", "udp.port==67 || udp.port==68"),
        ("000000", "imap", "tcp.port==993"),
        ("000000", "smtp", "tcp.port==587"),
        ("000000", "ssh", "tcp.port==22"),
    ];
    for (color, name, filter) in histos {
        for (suffix, data_file) in
            vec![("laptop", &data_files_laptop), ("swift", &data_files_swift)]
        {
            p.elements.insert(
                format!("histo_{}_{}", name.clone(), suffix),
                Box::new(mk_histogram(
                    &data_file,
                    filter.to_string().clone(),
                    name.to_string(),
                    color.to_string(),
                )),
            );
        }
    }
    template::fill_generated_data_in_template(&p);
}
