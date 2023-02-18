use rustdx::file::gbbq::Factor;
use std::error::Error;
use std::process;

fn example() -> Result<(), Box<dyn Error>> {
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(std::fs::File::open("factor.csv")?);
    // for result in rdr.deserialize() {
    //     let record: Factor = result?;
    //     println!("{:?}", record);
    // }
    let hm: std::collections::HashMap<_, _> = rdr
        .deserialize()
        .filter_map(|f| f.ok())
        .map(|f: Factor| (f.code.clone(), f))
        .collect();
    println!("{:?}", hm);
    Ok(())
}

fn main() {
    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
