use std::{fs, path::Path};

use super::Space;

pub fn write_to_file(file_str: &str, space: &Space) -> Result<(), Box<dyn std::error::Error>> {
    let mut content: String = String::new();
    for y in 0..space.y_dim() {
        for x in 0..space.x_dim() {
            if space.get_cell(x, y).unwrap().is_alive() {
                content.push('1');
            } else {
                content.push('0');
            }
            if x == space.x_dim() - 1 {
                content.push('\n');
            }
        }
    }
    let file_path = Path::new(file_str);
    fs::write(file_path, content).expect("Failed to write to file");
    Ok(())
}

pub fn read_from_file(file_str: &str) -> Result<Space, Box<dyn std::error::Error>> {
    let file_path = Path::new(file_str);
    let content = fs::read_to_string(file_path).expect("Failed to read file");
    let lines: Vec<&str> = content.lines().collect();
    let x_dim = lines[0].len() as u16;
    let y_dim = lines.len() as u16;
    let mut space = Space::new(x_dim, y_dim);
    let mut y = 0;
    for line in lines {
        let mut x = 0;
        for c in line.chars() {
            if c == '1' {
                space.revive_cell(x, y);
            }
            x += 1;
        }
        y += 1;
    }
    space.save_state(space.displayed_time);
    Ok(space)

}
