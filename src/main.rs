use clap::Parser;
use csv::WriterBuilder;
use std::fs::{canonicalize, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file path
    #[arg(short, long, value_parser, value_name = "FILE_PATH", required_unless_present = "file")]
    file: Option<String>,

    /// Target number
    #[arg(short, long, value_parser, value_name = "TARGET", required_unless_present = "target", allow_negative_numbers = true)]
    target: Option<f64>,

    /// Input file path (as positional argument)
    #[arg(value_name = "FILE_PATH", last(true))]
    file_pos: Option<String>,

    /// Target number (as positional argument)
    #[arg(value_name = "TARGET", last(true), allow_negative_numbers = true)]
    target_pos: Option<f64>,
}

/// 从文件中读取数据并转换为 f64 类型的向量
fn read_numbers_from_file(file_path: &str) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut numbers = Vec::new();
    let mut skip_header = true;

    for line in reader.lines() {
        let line = line?;
        if skip_header {
            skip_header = false;
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(number_str) = parts.get(0) {
            if let Ok(number) = f64::from_str(number_str) {
                numbers.push(number);
            }
        }
    }

    Ok(numbers)
}

/// 使用回溯算法查找第一个可能的组合
fn find_first_combination(nums: &[f64], target: f64) -> Option<Vec<f64>> {
    let mut result = None;
    let mut path = Vec::new();

    backtrack_first(nums, target, 0, &mut path, &mut result);
    result
}

/// 回溯函数
fn backtrack_first(
    nums: &[f64],
    target: f64,
    start: usize,
    path: &mut Vec<f64>,
    result: &mut Option<Vec<f64>>,
) {
    if let Some(ref mut _res) = result {
        // 已经找到了一个解，直接返回
        return;
    }

    let sum: f64 = path.iter().sum();
    if sum == target {
        *result = Some(path.clone());
        return;
    } else if sum > target.abs() {
        // 如果超过了目标值的绝对值，则直接返回
        return;
    }

    for i in start..nums.len() {
        path.push(nums[i]);
        backtrack_first(nums, target, i + 1, path, result);
        path.pop(); // 回溯
        if result.is_some() {
            break;
        }
    }
}

/// 将组合写入 CSV 文件，每个组合作为一列
fn write_combinations_to_csv(
    combinations: &[Vec<f64>],
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .delimiter(b',') // 使用逗号作为分隔符
        .from_path(output_file)?;

    if let Some(combination) = combinations.first() {
        // 获取组合的最大长度，用于确定需要写多少行
        let max_length = combination.len();

        for i in 0..max_length {
            let mut record = Vec::new();
            for comb in combinations {
                if i < comb.len() {
                    record.push(comb[i].to_string());
                } else {
                    record.push(String::from("")); // 填充空格
                }
            }
            wtr.write_record(&record)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

fn main() {
    let args = Args::parse();

    let file_path = args
        .file
        .unwrap_or_else(|| args.file_pos.expect("--file or FILE_PATH is required"));
    let target = args
        .target
        .unwrap_or_else(|| args.target_pos.expect("--target or TARGET is required"));

    // 获取绝对路径
    let absolute_file_path = match canonicalize(&file_path) {
        Ok(abs_path) => abs_path,
        Err(_) => PathBuf::from(&file_path),
    };

    // 获取父路径
    let parent_path = match absolute_file_path.parent() {
        Some(parent) => parent,
        None => {
            eprintln!("Failed to get parent directory of the file.");
            return;
        }
    };

    let output_file = parent_path.join("result.csv");

    let start_time = Instant::now();

    match read_numbers_from_file(&absolute_file_path.to_str().unwrap_or(&file_path)) {
        Ok(nums) => {
            let first_combination = find_first_combination(&nums, target);
            println!(
                "First combination that sums up to {}: {:?}",
                target, first_combination
            );

            if let Some(combination) = first_combination {
                match write_combinations_to_csv(&[combination], &output_file.to_str().unwrap_or(""))
                {
                    Ok(_) => println!("Combination written to {}", output_file.display()),
                    Err(e) => eprintln!("Failed to write combination to CSV: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Failed to read numbers from file: {}", e),
    }

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
    let runtime = format!("{elapsed_time:.2}");
    println!("done, elapsed time: {} s.", runtime);
}