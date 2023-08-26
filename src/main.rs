use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

const READ_FILE: &str = "tex/test.tex";

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    //! Read file line by line
    //!
    //! # Arguments
    //! - `filename` - Path to file
    //!
    //! # Returns
    //! - `io::Result<io::Lines<io::BufReader<File>>>` - Lines iterator
    let file: File = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn count_exercises_completed(lines: io::Lines<io::BufReader<File>>) -> i32 {
    //! Count number of exercises completed
    //!
    //! # Arguments
    //! - `lines` - Lines iterator
    //!
    //! # Returns
    //! - `i32` - Number of exercises completed
    let mut count: i32 = 0;
    for line in lines {
        let line: String = line.unwrap();
        if line.contains("begin{exercise}") {
            // Increment count when an exercise environment begins
            count += 1;
        }
    }
    count
}

fn main() {
    let lines: io::Lines<io::BufReader<File>> = read_lines(READ_FILE).unwrap();
    let num_exercises: i32 = count_exercises_completed(lines);
    println!("Number of exercises completed: {}", num_exercises);
}
