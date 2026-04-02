// Build script used to generate code containing data structures.

use std::fs::File;
use std::io::{BufWriter, Write};

const SHAPE_FILE: &str = "data/ne_110m_coastline/ne_110m_coastline.shp";
const DATA_FILE: &str = "src/data.rs";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for file in ["build.rs", SHAPE_FILE, DATA_FILE] {
        println!("cargo:rerun-if-changed={file}");
    }

    let file = File::create(DATA_FILE)?;
    let mut file = BufWriter::new(file);
    file.write_all("// WARNING: This file is code generated!\n\n".as_bytes())?;
    write_data(&mut file, SHAPE_FILE, "COASTLINE")?;

    Ok(())
}

fn write_data(
    file: &mut BufWriter<File>,
    shape_file: &str,
    data_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    file.write_all(format!("pub const {data_name}: &[&[(f64, f64)]] = &[\n").as_bytes())?;

    let mut reader = shapefile::Reader::from_path(shape_file)?;
    for shape_record in reader.iter_shapes_and_records() {
        let (shape, _record) = shape_record?;
        match shape {
            shapefile::Shape::Polyline(polyline) => {
                file.write_all("    &[\n".as_bytes())?;
                for part in polyline.parts() {
                    for point in part {
                        file.write_all(
                            format!("        ({}f64, {}f64),\n", point.x, point.y).as_bytes(),
                        )?;
                    }
                }
                file.write_all("    ],\n".as_bytes())?;
            }
            shapefile::Shape::Polygon(polygon) => {
                file.write_all("    &[\n".as_bytes())?;
                for ring in polygon.rings() {
                    match ring {
                        shapefile::PolygonRing::Outer(points) => {
                            for point in points {
                                file.write_all(
                                    format!("        ({}f64, {}f64),\n", point.x, point.y)
                                        .as_bytes(),
                                )?;
                            }
                        }
                        shapefile::PolygonRing::Inner(_) => {}
                    }
                }
                file.write_all("    ],\n".as_bytes())?;
            }
            _ => file.write_all(format!("!!!ERROR({shape})!!!").as_bytes())?,
        }
    }
    file.write_all("];\n".as_bytes())?;

    Ok(())
}
