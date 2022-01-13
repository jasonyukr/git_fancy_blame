use std::io::{BufRead, BufReader};
use std::env;
use std::fs::File;
use std::collections::HashMap;

/*
ANSI 256color to true-color
===========================
 NAME       FG/BG     RGB
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

struct RGB {
    r: u32,
    g: u32,
    b: u32
}

fn generate_gradation(start: &RGB, end: &RGB, steps: u32) -> Vec<RGB> {
    // The number of colors to compute
    let len = steps;

    // Alpha blending amount
    let mut alpha = 0.0;

    let mut gradation: Vec<RGB> = Vec::new();

    for _i in 0..len {
        let red: f32;
        let green: f32;
        let blue: f32;
        alpha = alpha + (1.0 / len as f32);

        red = end.r as f32 * alpha + (1.0 - alpha) * start.r as f32;
        green = end.g as f32 * alpha + (1.0 - alpha) * start.g as f32;
        blue = end.b as f32 * alpha + (1.0 - alpha) * start.b as f32;

        let rgb = RGB {
            r: red as u32,
            g: green as u32,
            b: blue as u32
        };
        gradation.push(rgb)
    }
    return gradation;
}

fn main() {
    // Table for true-color gradation
    // (back_start.r, back_start.g, back_start.b, back_end.r, back_end.g, back_end.b, fore.r, fore.g, fore.b)
    let gradation_table =
        [(0x70, 0xc0, 0xb1, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xc3, 0x97, 0xd8, 0xc5, 0xc8, 0xc6 ,0x3c, 0x3e, 0x3f),
         (0x7a, 0xa6, 0xda, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xe7, 0xc5, 0x47, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xb9, 0xca, 0x4a, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0xd5, 0x4e, 0x53, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f),
         (0x8a, 0xbe, 0x87, 0xc5, 0xc8, 0xc6, 0x3c, 0x3e, 0x3f)];

    let mut gradation_idx = 0;

    let mut exec_name: String = String::from("");
    let mut revlist_filename: String = String::from("");
    let mut blame_filename: String = String::from("");
    let mut bat_filename: String = String::from("");

    // parse argument
    let mut idx_mode = false;
    for arg in env::args() {
        if idx_mode {
            gradation_idx = arg.parse::<usize>().unwrap();
            if gradation_idx >= gradation_table.len() {
                gradation_idx = 0;
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

    let mut revlist_map = HashMap::new();
    if revlist_file.is_ok() {
        let reader = BufReader::new(revlist_file.unwrap());
        for (index, line) in reader.lines().enumerate() {
            let line = line.unwrap();
            revlist_map.insert(line, index as usize);
        }
    } else {
        return;
    }

    let fore = RGB {
        r: gradation_table[gradation_idx].6,
        g: gradation_table[gradation_idx].7,
        b: gradation_table[gradation_idx].8
    };
    let back_start = RGB {
        r: gradation_table[gradation_idx].0,
        g: gradation_table[gradation_idx].1,
        b: gradation_table[gradation_idx].2
    };
    let back_end = RGB {
        r: gradation_table[gradation_idx].3,
        g: gradation_table[gradation_idx].4,
        b: gradation_table[gradation_idx].5
    };
    let gradation = generate_gradation(&back_start, &back_end, revlist_map.len() as u32);

    let mut bat_lines = vec![];
    if bat_file.is_ok() {
        let reader = BufReader::new(bat_file.unwrap());
        for (_, line) in reader.lines().enumerate() {
            let line = line.unwrap();
            bat_lines.push(line);
        }
    } else {
        return;
    }

    if blame_file.is_ok() {
        let reader = BufReader::new(blame_file.unwrap());
        for (index, line) in reader.lines().enumerate() {
            let line = line.unwrap();

            // get hash-value
            let hash;
            match line.find(' ') {
                Some(found) => hash = &line[..found],
                None => hash = &line
            }

            // get matching index from hash-value
            let matching_idx;
            if hash == "0000000" {
                // special case: local change
                matching_idx = 0;
            } else {
                match revlist_map.get(hash) {
                    Some(found) => matching_idx = *found,
                    None => matching_idx = revlist_map.len() - 1
                }
            }

            // get current gradation color. default is back_end
            let mut back = RGB {
                r: back_end.r,
                g: back_end.g,
                b: back_end.b
            };
            let rgb = gradation.get(matching_idx);
            if !rgb.is_none() {
                back.r = rgb.unwrap().r;
                back.g = rgb.unwrap().g;
                back.b = rgb.unwrap().b;
            }

            println!("│\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}\x1b[0m│{}", fore.r, fore.g, fore.b, back.r, back.g, back.b, line, bat_lines[index]);
        }
    } else {
        return;
    }
}
