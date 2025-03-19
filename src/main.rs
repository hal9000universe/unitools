use std::env::join_paths;
use std::fs::{File, read_dir};
use std::io::{self, BufRead};
use std::path::Path;

use chrono::prelude::*;
use lopdf::Document;
use plotters::prelude::*;
use walkdir::{DirEntry, WalkDir};

const TASK_IDS: [&str; 0] = [];
const SOLUTION_ID: &str = "begin{exercise}";

const TASK_IDENTIFIERS: [&str; 0] = [];

const SOLUTION_IDENTIFIERS: [&str; 1] = [
    ".tex"
];

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with("."))
        .unwrap_or(false)
}

fn get_last_monday() -> String {
    //! Get the date of the last monday
    //!
    //! # Returns
    //! - `DateTime<Local>` - Date of the last monday as "year-month-day"
    let today: DateTime<Local> = Local::now();
    let last_monday: DateTime<Local> =
        today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    last_monday.format("%Y-%m-%d").to_string()
}

fn get_next_monday() -> String {
    let today: DateTime<Local> = Local::now();
    let next_monday: DateTime<Local> =
        today + chrono::Duration::days(7 - today.weekday().num_days_from_monday() as i64);
    next_monday.format("%Y-%m-%d").to_string()
}

fn convert_to_text(filename: &String) -> String {
    //! Convert PDF to text
    //!
    //! # Arguments
    //! - `filename` - Path to file
    //!
    //! # Returns
    //! - `String` - Text
    let text: String = match Document::load(filename) {
        Ok(doc) => {
            let mut text: String = String::from("");
            for (idx, _page) in doc.get_pages().iter().enumerate().map(|(x, y)| ((x + 1) as u32, y)) {
                text += doc.extract_text(&[idx]).unwrap().as_str();
            }
            return text
        },
        Err(_) => String::from(""),
    };
    text
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

fn count_exercises_completed(lines: io::Lines<io::BufReader<File>>) -> usize {
    //! Count number of exercises completed
    //!
    //! # Arguments
    //! - `lines` - Lines iterator
    //!
    //! # Returns
    //! - `usize` - Number of exercises completed
    let mut count: usize = 0;
    for line in lines {
        let line: String = line.unwrap();
        if line.contains(&SOLUTION_ID) {
            // Increment count when an exercise environment begins
            count += 1;
        }
    }
    count
}

fn count_tasks_assigned(text: String) -> usize {
    //! Count number of tasks assigned
    //!
    //! # Arguments
    //! - `text` - Text
    //!
    //! # Returns
    //! - `usize` - Number of tasks assigned
    let text = text.replace(" ", "").as_str().trim().to_lowercase();
    let mut count: usize = 0;
    for task_id in TASK_IDS.iter() {
        count += text.matches(task_id).count();
    }
    count
}

fn visualize_todos(
    save_file: &String,
    num_tasks: usize,
    num_solutions: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    //! Creates bar charts of the number of exercises assigned and completed
    //!
    //! # Returns
    //! - `Result<(), Box<dyn std::error::Error>>` - Result
    let num_todo: usize = num_tasks - num_solutions;

    let root = BitMapBackend::new(save_file, (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    // build chart
    let mut chart = ChartBuilder::on(&root)
        .caption("Progress", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .build_cartesian_2d(0..3, 0..num_tasks + 1)
        .unwrap();

    chart
        .configure_mesh()
        .y_desc("Number of exercises")
        .axis_desc_style(("sans-serif", 25))
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()
        .unwrap();

    chart
        .draw_series(
            Histogram::vertical(&chart)
                .style(RED.mix(0.5).filled())
                .data(vec![(0, num_todo)]),
        )
        .unwrap()
        .label("todo")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.mix(0.5)));

    chart
        .draw_series(
            Histogram::vertical(&chart)
                .style(GREEN.mix(0.5).filled())
                .data(vec![(1, num_solutions)]),
        )
        .unwrap()
        .label("done")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN.mix(0.5)));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();

    // save chart to file
    root.present().expect("Unable to save todo.png");
    println!("Result has been saved to {}", save_file);
    Ok(())
}

fn make_mondays(semester: &String, monday: &String) {
    //! Creates directories for mondays if they do not exist
    for course in read_dir(semester).unwrap() {
        let course_path: String = course.unwrap().path().to_str().unwrap().to_string();
        // check if semester is a file
        if course_path.contains(".") {
            continue;
        }
        // check if monday directory exists
        let monday_path: String = join_paths(&[&course_path, &monday])
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .replace(":", "/");
        if !Path::new(&monday_path).exists() {
            println!("Creating directory {:?}", monday_path);
            std::fs::create_dir_all(monday_path).unwrap();
        }
    }
}

fn search(semester: &String) {
    let save_file: String = String::from("todo.png");
    // iterate through semester directories
    let last_monday: String = get_last_monday();
    // make monday directories
    make_mondays(&semester, &last_monday);
    println!("Processing semester {:?}", &semester);
    for course in read_dir(&semester).unwrap() {
        let course_path: String = course.unwrap().path().to_str().unwrap().to_string();
        // check if semester is a file
        if course_path.contains(".") {
            continue;
        }
        println!("Processing course {:?}", course_path);
        for week in read_dir(course_path).unwrap() {
            let week_path: String = week.unwrap().path().to_str().unwrap().to_string();
            // check if week is a file
            if week_path.contains(".") {
                continue;
            }
            println!("Processing week {:?}", week_path);
            {
                let save_file: String = join_paths(&[&week_path, &save_file])
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .replace(":", "/");
                // find task and solution files
                let mut num_tasks: usize = 0;
                let mut num_solutions: usize = 0;
                for entry in WalkDir::new(week_path)
                    .into_iter()
                    .filter_entry(|e| is_not_hidden(e))
                {
                    let entry: DirEntry = entry.unwrap();
                    let path: String = entry.path().to_str().unwrap().to_string().to_lowercase();
                    println!("Processing file {:?}", &path);

                    for task_identifier in TASK_IDENTIFIERS.iter() {
                        if path.contains(task_identifier) {
                            println!("Found task file {:?}", &path);
                            num_tasks = count_tasks_assigned(convert_to_text(&path));
                            break
                        }
                    }

                    for solution_identifier in SOLUTION_IDENTIFIERS.iter() {
                        if path.contains(solution_identifier) {
                            println!("Found solution file {:?}", &path);
                            num_solutions = count_exercises_completed(read_lines(&path).unwrap());
                            break
                        }
                    }
                }
                // visualize todos
                visualize_todos(&save_file, num_tasks, num_solutions).unwrap();
            }
        }
    }
}

fn mathematics(semester: &String) {
    search(&semester);
}

fn main() {
    let semester: String = String::from("LMU/mathematics/SoSe25");
    let physics: String = String::from("LMU/physics/Aufgaben");
    make_mondays(&semester, &get_next_monday());
    make_mondays(&physics, &get_next_monday());
}