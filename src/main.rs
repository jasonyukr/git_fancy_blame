use std::io::{BufRead, BufReader};
use std::env;
use std::fs::File;
use std::collections::HashMap;

/*
ANSI 256color to true-color
===========================
 NAME       FG/BG     Rgb
---------------------------
white      37m/47m   c5c8c6
red        31m/41m   cc6666
green      32m/42m   b5bd68
yellow     33m/43m   f0c674
blue       34m/44m   81a2be
magenta    35m/45m   b294bb
cyan       36m/46m   8abe87
gray       90m/100m  666666
Br white   97m/107m  eaeaea
Br red     91m/101m  d54e53
Br green   92m/102m  b9ca4a
Br yellow  93m/103m  e7c547
Br blue    94m/104m  7aa6da
Br magenta 95m/105m  c397d8
Br cyan    96m/106m  70c0b1
===========================
*/

#[derive(Debug, Copy, Clone)]
struct Rgb {
    r: u32,
    g: u32,
    b: u32
}

fn get_grad(start: &Rgb, end: &Rgb, steps: u32) -> Vec<Rgb> {
    let mut alpha = 0.0;
    let mut grad: Vec<Rgb> = Vec::new();

    for _i in 0..steps {
        let red: f32;
        let green: f32;
        let blue: f32;
        alpha = alpha + (1.0 / steps as f32);

        red = end.r as f32 * alpha + (1.0 - alpha) * start.r as f32;
        green = end.g as f32 * alpha + (1.0 - alpha) * start.g as f32;
        blue = end.b as f32 * alpha + (1.0 - alpha) * start.b as f32;

        let rgb = Rgb {
            r: red as u32,
            g: green as u32,
            b: blue as u32
        };
        grad.push(rgb)
    }
    return grad;
}

fn main() {
    // Table for true-color gradation
    // (back_start_color.r, back_start_color.g, back_start_color.b, back_end_color.r, back_end_color.g, back_end_color.b, fore_color.r, fore_color.g, fore_color.b)
    let grad_table =
        [(0x70, 0xc0, 0xb1, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xc3, 0x97, 0xd8, 0xc5, 0xc8, 0xc6 ,0x3c, 0x3e, 0x3f),
         (0x7a, 0xa6, 0xda, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xe7, 0xc5, 0x47, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xb9, 0xca, 0x4a, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xd5, 0x4e, 0x53, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0x8a, 0xbe, 0x87, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f)];

    let mut grad_idx = 0;

    let mut exec_name: String = String::from("");
    let mut revlist_filename: String = String::from("");
    let mut blame_filename: String = String::from("");
    let mut bat_filename: String = String::from("");

    // parse argument
    let mut idx_mode = false;
    for arg in env::args() {
        if idx_mode {
            if let Ok(i) = arg.parse::<usize>() {
                grad_idx = i;
                if grad_idx >= grad_table.len() {
                    grad_idx = 0;
                }
            }
            idx_mode = false;
            continue;
        }
        if arg == "-g" || arg == "--g" {
            idx_mode = true
        } else {
            if exec_name == "" {
                exec_name = arg.clone();
            } else if revlist_filename == "" {
                revlist_filename = arg.clone();
            } else if blame_filename == "" {
                blame_filename = arg.clone();
            } else if bat_filename == "" {
                bat_filename = arg.clone();
            }
        }
    }
    if revlist_filename == "" || blame_filename == "" || bat_filename == "" {
        return;
    }

    let revlist_file = File::open(revlist_filename);
    let blame_file = File::open(blame_filename);
    let bat_file = File::open(bat_filename);

    // load revlist file
    let mut revlist_map = HashMap::new();
    if let Ok(file) = revlist_file {
        let reader = BufReader::new(file);
        for (index, line) in reader.lines().enumerate() {
            if let Ok(ln) = line {
                revlist_map.insert(ln, index as usize);
            }
        }
    } else {
        return;
    }

    let fore_color = Rgb {
        r: grad_table[grad_idx].6,
        g: grad_table[grad_idx].7,
        b: grad_table[grad_idx].8
    };
    let back_start_color = Rgb {
        r: grad_table[grad_idx].0,
        g: grad_table[grad_idx].1,
        b: grad_table[grad_idx].2
    };
    let back_end_color = Rgb {
        r: grad_table[grad_idx].3,
        g: grad_table[grad_idx].4,
        b: grad_table[grad_idx].5
    };
    let grad = get_grad(&back_start_color, &back_end_color, revlist_map.len() as u32);

    // load bat file
    let mut bat_lines = vec![];
    if let Ok(file) = bat_file {
        let reader = BufReader::new(file);
        for ln in reader.lines() {
            if let Ok(line) = ln {
                bat_lines.push(line);
            }
        }
    } else {
        return;
    }

    // load blame file and process each line
    if let Ok(file) = blame_file {
        let reader = BufReader::new(file);
        for (index, ln) in reader.lines().enumerate() {
            let line;
            match ln {
                Ok(data) => line = data,
                Err(_) => continue
            }

            // check bat line first
            if index >= bat_lines.len() {
                return;
            }

            /*
            * According to git blame --help
            *
            * --abbrev=<n>
            *      Instead of using the default 7+1 hexadecimal digits as the abbreviated object name, use <n>+1 digits.
            *      Note that 1 column is used for a caret to mark the boundary commit.
            *
            *  We get the <n>+1 length hash-raw value and then make <n> length hash value.
            */

            // split hash-raw (<n>+1 length) and remaining
            let hash_raw;
            let remaining;
            if let Some(i) = line.find(' ') {
                hash_raw = &line[..i];
                remaining = &line[i..];
            } else {
                hash_raw = &line;
                remaining = "";
            }

            // make hash (<n> length) value
            let mut hash: String = hash_raw.to_string();
            let hash_len = hash.chars().count();
            if hash_len > 1 {
                if hash.chars().nth(0) == Some('^') {
                    hash = (&hash[1..]).to_string();
                } else {
                    hash = (&hash[..hash_len-1]).to_string();
                }
            }

            // get matching index from hash-value
            let mut matching_idx = 0;
            if hash != "000000000000" { // 00000... is local change
                if let Some(found) = revlist_map.get(&hash) {
                    matching_idx = *found;
                } else {
                    matching_idx = revlist_map.len() - 1;
                }
            }

            // get current gradation color from matching index
            let mut back_color = &back_end_color;
            if let Some(color) = grad.get(matching_idx) {
                back_color = color;
            }

            println!("│\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}{}\x1b[0m│{}",
                    fore_color.r, fore_color.g, fore_color.b,
                    back_color.r, back_color.g, back_color.b,
                    hash, remaining, bat_lines[index]);
        }
    }
}
