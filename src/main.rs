use std::fs::{File, read};
use std::io::{self, BufRead};
use std::path::Path;
use pdf_extract::extract_text_from_mem;

const SOLUTION_FILE: &str = "WiSe23/fake solution.tex";
const TASK_FILE: &str = "WiSe23/fake task.pdf";

const SOLUTION_ID: &str = "begin{exercise}";
const TASK_ID: &str = "Exercise";

fn convert_to_text(filename: &str) -> String {
    //! Convert PDF to text
    //!
    //! # Arguments
    //! - `filename` - Path to file
    //!
    //! # Returns
    //! - `String` - Text
    let bytes: Vec<u8> = read(filename).unwrap();
    let text: String = extract_text_from_mem(&bytes).unwrap();
    text.replace(" ", "")
}

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
        if line.contains(&SOLUTION_ID) {
            // Increment count when an exercise environment begins
            count += 1;
        }
    }
    count
}

fn count_tasks_assigned(text: String) -> i32 {
    //! Count number of tasks assigned
    //!
    //! # Arguments
    //! - `text` - Text
    //!
    //! # Returns
    //! - `i32` - Number of tasks assigned
    text.matches(&TASK_ID).count() as i32
}

fn main() {
    let task_text: String = convert_to_text(TASK_FILE);
    let num_tasks: i32 = count_tasks_assigned(task_text);
    println!("Number of exercises assigned: {}", num_tasks);

    let solution_lines: io::Lines<io::BufReader<File>> = read_lines(SOLUTION_FILE).unwrap();
    let num_solutions: i32 = count_exercises_completed(solution_lines);
    println!("Number of exercises completed: {}", num_solutions);
}
