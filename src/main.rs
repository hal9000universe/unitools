use std::fs::{File, read};
use std::io::{self, BufRead};
use std::path::Path;
use pdf_extract::extract_text_from_mem;
use plotters::prelude::*;

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

// TODO: implement directory traversal
// TODO: implement dates, change style, refactor
fn visualize_todos() -> Result<(), Box<dyn std::error::Error>> {
    //! Creates bar charts of the number of exercises assigned and completed
    //! 
    //! # Returns
    //! - `Result<(), Box<dyn std::error::Error>>` - Result
    let total_num_tasks: i32 = count_tasks_assigned(convert_to_text(TASK_FILE));
    let total_num_solutions: i32 = count_exercises_completed(read_lines(SOLUTION_FILE).unwrap());

    let save_file: &str = "WiSe23/todo.png";

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
                .data(vec![(0, total_num_tasks)]),
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
    visualize_todos().unwrap();
}
