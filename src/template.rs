use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use xml::reader::{EventReader, XmlEvent};

use plakat::Plakat;

/// Fill data in template.
///
/// The template file keeps:
/// - positions of each element
///
/// It gets filled with:
/// - the SVG elements that are defined in code
/// - new image paths
///
pub fn fill_generated_data_in_template(p: &Plakat) {
    let mut image_links_to_change: HashMap<String, (String, String)> = HashMap::new();

    let file = File::open(p.template_path).unwrap();
    let file = BufReader::new(file);

    let parser = EventReader::new(file);
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                let xml::name::OwnedName { local_name, .. } = name;
                // println!("Found local_name {}", local_name);
                if local_name != "image" {
                    continue;
                }
                let (id, xlink) = {
                    let mut res_id: Option<String> = None;
                    let mut res_xlink: Option<String> = None;
                    for a in attributes {
                        let xml::attribute::OwnedAttribute { name: n, value: v } = a;
                        let xml::name::OwnedName {
                            local_name: n_str,
                            namespace: _,
                            prefix,
                        } = n;
                        if n_str == "id" {
                            res_id = Some(v);
                        } else if prefix == Some("xlink".to_string()) && n_str == "href" {
                            res_xlink = Some(v);
                        }
                    }
                    if res_id.is_some() && res_xlink.is_some() {
                        (res_id.unwrap(), res_xlink.unwrap())
                    } else {
                        continue;
                    }
                };
                if p.elements.contains_key(&id) {
                    println!("Going to change image with id: {}", &id);
                    image_links_to_change.insert(
                        id.clone(),
                        (
                            xlink,
                            p.elements
                                .get(&id)
                                .unwrap()
                                .png_cached()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string(),
                        ),
                    );
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    let mut template_string = std::fs::read_to_string("template.svg").unwrap();
    for (_, (old, new)) in &image_links_to_change {
        println!("Replacing {} with {} in template.svg", old, new);
        template_string = template_string.replace(old, new);
    }
    std::fs::write("template.svg", template_string).unwrap();
}
