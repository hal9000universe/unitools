use std::env::join_paths;
use std::fs::{File, read, read_dir};
use std::io::{self, BufRead};
use std::path::Path;
use pdf_extract::extract_text_from_mem;
use plotters::prelude::*;
use walkdir::{WalkDir, DirEntry};
use chrono::prelude::*;

const SOLUTION_ID: &str = "begin{exercise}";
const TASK_ID: &str = "Exercise";

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
    let last_monday: DateTime<Local> = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    last_monday.format("%Y-%m-%d").to_string()
}

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

fn visualize_todos(task_file: &str, solution_file: &str, save_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    //! Creates bar charts of the number of exercises assigned and completed
    //! 
    //! # Returns
    //! - `Result<(), Box<dyn std::error::Error>>` - Result
    let total_num_tasks: i32 = count_tasks_assigned(convert_to_text(task_file));
    let total_num_solutions: i32 = count_exercises_completed(read_lines(solution_file).unwrap());
    let num_todo: i32 = total_num_tasks - total_num_solutions;

    let root = BitMapBackend::new(save_file, (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    // build chart
    // name first bar "done", color: green
    // name second bar "todo", color: red
    // y axis: "number of exercises"
    let mut chart = ChartBuilder::on(&root)
        .caption("Progress", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .build_cartesian_2d(0..3, 0..total_num_tasks + 1)
        .unwrap();

    chart
        .configure_mesh()
        .y_desc("number of exercises")
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
                .data(vec![(1, total_num_solutions)]),
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

fn main() {
    let  save_file: String = String::from("todo.png");
    // iterate through semester directories 
    let last_monday: String = get_last_monday();
    for semester in read_dir("semesters").unwrap() {
        let semester_path: String = semester.unwrap().path().to_str().unwrap().to_string();
        // check if semester is file
        if semester_path.contains(".") {
            continue;
        }
        println!("Processing semester {:?}", semester_path);
        for course in read_dir(semester_path).unwrap() {
            let course_path: String = course.unwrap().path().to_str().unwrap().to_string();
            // check if course is file
            if course_path.contains(".") {
                continue;
            }
            println!("Processing course {:?}", course_path);
            for week in read_dir(course_path).unwrap() {
                let week_path: String = week.unwrap().path().to_str().unwrap().to_string();
                // check if week is file
                if week_path.contains(".") {
                    continue;
                }
                println!("Processing week {:?}", week_path);
                if week_path.contains(&last_monday) {
                    let save_file: String = join_paths(&[week_path.clone(), save_file.clone()]).unwrap().to_str().unwrap().to_string().replace(":", "/");
                    // find task and solution files
                    let mut task_file: String = String::from("");
                    let mut solution_file: String = String::from("");
                    for entry in WalkDir::new(week_path.clone()).into_iter().filter_entry(|e| is_not_hidden(e)) {
                        let entry: DirEntry = entry.unwrap();
                        let path: String = entry.path().to_str().unwrap().to_string();
                        if path.contains("task") {
                            println!("Found task file {:?}", path);
                            task_file = path;
                        }
                        else if path.contains("solution") {
                            println!("Found solution file {:?}", path);
                            solution_file = path;
                        }
                    }
                    // visualize todos
                    visualize_todos(&task_file, &solution_file, &save_file).unwrap();
                }
            }
        }
    }
}