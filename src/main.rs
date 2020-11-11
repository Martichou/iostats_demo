use std::{
    io::{self, Write, prelude::*, SeekFrom},
    thread,
    time::Duration,
    fs::File,
    collections::HashMap,
};

// This structure will hold our data for the disks
struct IoStats {
    pub mb_read: f64,
    pub mb_wrtn: f64,
}

// We won't handle any error case in this guide
fn main() {
    // 2048 is for mb
    // One sector is 512b and 1 sector is typically 512b
    // So we keep it in mind and when we'll read /proc/diskstats
    // We'll divide the number of sector read by 512 and then by 2048 for mb
    // And compute the difference to obtain mb/s.
    let fctr = 2048.0;
    // Hashmap of previous drives stats to compute difference from
    let mut prev: HashMap<String, IoStats> = HashMap::new();
    // Open the file we'll use to get the stats
    let mut fd = File::open(&"/proc/diskstats").unwrap();
    loop {
        // Create the curr Hashmap, allow us to compare with the prev one
        let mut curr: HashMap<String, IoStats> = HashMap::new();
        // Create the output string
        let mut output = String::new();
        // Add the header string to the output
        output.push_str("\nDevice          mb_reads/s      mb_wrtn/s\n\n");
        // Collecting info/data
        {
            // Create a new empty string
            let mut io_data = String::new();
            // Read the content of the file (diskstats) to the io_data string
            fd.read_to_string(&mut io_data).unwrap();
            // Iterate over each line (each disk)
            for line in io_data.lines() {
                // Split field (separated by whitespace) and collect them without specific type
                let fields = line.split_whitespace().collect::<Vec<_>>();
                // If the are less than 14 fields, the file is missing data
                // see (https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats)
                if fields.len() < 14 {
                    panic!("Not enough data from diskstats");
                }
                let ds = IoStats {
                    mb_read: fields[5].parse::<f64>().unwrap() / fctr,
                    mb_wrtn: fields[9].parse::<f64>().unwrap() / fctr,
                };
                // If prev already contains the info we compute the diff to get mb/s
                // Else we add to the print line the "fake" data.
                if prev.contains_key(fields[2]) {
                    // Get the object from the hashmap
                    let pds = prev.get(fields[2]).unwrap();
                    // Construct speed line and append it to curr hashmap
                    let mb_read_s = ds.mb_read - pds.mb_read;
                    let mb_wrtn_s = ds.mb_wrtn - pds.mb_wrtn;
                    // Add the line, formatted with color and spacing
                    output.push_str(&format!("\x1b[0;32m{:16}\x1b[0m\x1b[0;34m{:10.2}{:15.2}\x1b[0m\n", fields[2], mb_read_s, mb_wrtn_s));
                    // Insert the current disk data to the curr HashMap
                    // the curr will later be saved as prev
                    curr.insert(fields[2].to_owned(), ds);
                } else {
                    // Add the line with fake data and formatting
                    output.push_str(&format!("\x1b[0;32m{:16}\x1b[0m\x1b[0;34m{:10.2}{:15.2}\x1b[0m\n", fields[2], 0.00, 0.00));
                }
            }
            // Move the cursor to the start of the file
            fd.seek(SeekFrom::Start(0)).unwrap();
        }
        // Print the result
        writeln!(io::stdout().lock(), "{}", output);
        // Save current as previous for the next loop
        prev = curr;
        // Wait for 1 seconds to respect the mb/s
        thread::sleep(Duration::from_secs(1));
    }
}
