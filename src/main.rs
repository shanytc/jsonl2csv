use std::fs::File;
use std::io::{BufRead, BufReader};
use anyhow::{bail, Context, Result};
use clap::Parser;
use csv::Writer;
use serde_json::{Value, from_str};

/// Convert a JSON‑Lines file (one JSON object per line) to a CSV file.
#[derive(Parser, Debug)]
#[command(author, version, about = "Convert JSONL to CSV", long_about = None)]
struct Cli {
    input: String,
    output: String,
}

fn json_to_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        // For arrays/objects, fall back to compact JSON representation
        other => other.to_string(),
    }
}

fn main() -> Result<()> {
    // Parse CLI flags
    let cli = Cli::parse();

    // Stream input to keep memory usage low
    let infile = File::open(&cli.input)
        .with_context(|| format!("Cannot open input file: {}", &cli.input))?;
    let reader = BufReader::new(infile);

    let mut wtr = Writer::from_path(&cli.output)
        .with_context(|| format!("Cannot create output file: {}", &cli.output))?;

    let mut headers: Vec<String> = Vec::new();
    let mut header_written = false;

    // Read each line from the input file
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        // Skip blank lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse the JSON object in this line
        let value: Value = from_str(&line)
            .with_context(|| format!("JSON parse error on line {}", idx + 1))?;

        let obj = match value {
            Value::Object(map) => map,
            _ => bail!("Line {} is not a JSON object", idx + 1),
        };

        // Capture header from the first record
        if !header_written {
            headers = obj.keys().cloned().collect();
            wtr.write_record(&headers)?;
            header_written = true;
        }

        // For each following record, output fields in header order.
        // If a field is missing, write an empty string.
        let record: Vec<String> = headers
            .iter()
            .map(|k| obj.get(k).map(json_to_string).unwrap_or_default())
            .collect();

        wtr.write_record(&record)?;
    }

    println!("Conversion from {} to {} successfully completed.", cli.input, cli.output);

    wtr.flush()?;
    Ok(())
}

