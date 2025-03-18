use crate::space::cell::Cell;
use std::{error::Error, fmt};
use linked_hash_map::LinkedHashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;


pub(crate) mod cell;
pub mod io;
#[derive(Clone)]
#[allow(dead_code)]
pub struct Space {
    cells: Vec<Vec<Cell>>,
    pub states_hash_map: LinkedHashMap<usize, Vec<(u16, u16)>>,
    pub displayed_time: usize
}

impl Space {
    pub fn new(x_dim: u16, y_dim: u16) -> Space {
        let mut cells: Vec<Vec<Cell>> = Vec::with_capacity(x_dim as usize);
        for x in 0..x_dim {
            let mut column : Vec<Cell> = Vec::with_capacity(y_dim as usize);
            for y in 0..y_dim {
                column.push(Cell::new(x, y, 0));
            }
            cells.push(column);
        }
        let mut states_hashmap: LinkedHashMap<usize, Vec<(u16, u16)>> = LinkedHashMap::new();
        states_hashmap.insert(0, vec![]);
        Space{cells, states_hash_map: states_hashmap, displayed_time: 0 }
    }

    #[allow(dead_code)]
    pub(crate) fn build_from_array(array: &Vec<Vec<u8>>) -> Space {
        let x_dim = array[0].len() as u16;
        let y_dim = array.len() as u16;
        let mut space = Space::new(x_dim, y_dim);
        for x in 0..x_dim {
            for y in 0..y_dim {
                if array[y as usize][x as usize] == 1 {
                    space.revive_cell(x, y);
                }
            }
        }
        space
    }

    pub fn save_state(&mut self, time: usize) {
        let alive_cells = self.get_alive_cells();
        let mut indices: Vec<(u16, u16)> = Vec::new();
        for alive_cell in &alive_cells {
            indices.push((alive_cell.x, alive_cell.y));
        }
        self.states_hash_map.insert(time, indices);
        self.displayed_time = time;
    }

    pub fn load_state(&mut self, time: usize) {
        let cloned_space = self.clone();
        self.kill_all_cells();
        let alive_tuples = &cloned_space.states_hash_map[&time];
        for alive_tuple in alive_tuples {
            self.revive_cell(alive_tuple.0, alive_tuple.1);
        }
        self.displayed_time = time;

    }

    pub(crate) fn flat_mut(&mut self) -> Vec<&mut Cell> {
        self.cells.iter_mut().flat_map(|row| row.iter_mut()).collect()
    }

    pub(crate) fn flat(&self) -> Vec<&Cell> {
        self.cells.iter().flat_map(|row| row.iter()).collect()
    }

    pub fn x_dim(&self) -> u16 {
        self.cells.len() as u16
    }

    pub fn y_dim(&self) -> u16 {
        self.cells[0].len() as u16
    }

    pub(crate) fn revive_cell(&mut self, x: u16, y: u16) {
        let cell = self.get_cell_mut(x, y).unwrap();
        cell.revive();
    }

    pub fn let_cell_age(&mut self, x: u16, y: u16) {
        let cell = self.get_cell_mut(x, y).unwrap();
        cell.age();
    }

    pub fn kill_all_cells(&mut self) {
        let alive_cells = self.get_cells_with_energy_mut();
        for cell in alive_cells {
            cell.kill();
        }
    }

    #[allow(dead_code)]
    pub fn check_cell_is_alive(&self, x: u16, y: u16) -> bool {
        self.get_cell(x, y).unwrap().is_alive()
    }

    pub fn revive_random_cells(&mut self, num_cells: usize) {
        let mut rng = thread_rng();
        let mut flat_cells = self.flat_mut();
        let mut indices: Vec<usize> = (0..flat_cells.len()).collect();
        indices.shuffle(&mut rng);
        let num_cells = num_cells.min(flat_cells.len());
        for i in 0..num_cells {
            let index = indices[i];
            flat_cells.get_mut(index).unwrap().revive();
        }
    }

    pub fn get_cell_mut(&mut self, x: u16, y: u16) -> Result<&mut Cell, OutOfBoundsError> {
        if x < self.x_dim() && y < self.y_dim() {
            Ok(&mut self.cells[x as usize][y as usize])
        } else {
            Err(OutOfBoundsError::new("Index out of bounds!"))
        }
    }

    pub fn get_cell(&self, x: u16, y: u16) -> Result<&Cell, OutOfBoundsError> {
        if x < self.x_dim() && y < self.y_dim() {
            Ok(&self.cells[x as usize][y as usize])
        } else {
            Err(OutOfBoundsError { message: "Index out of bounds!".to_string() })
        }
    }


    #[allow(dead_code)]
    pub fn get_num_alive_cells(&self) -> usize {
        let mut flat_cells = self.flat();
        let mut num_alive_cells = 0;
        while !flat_cells.is_empty() {
            let cell = flat_cells.pop().unwrap();
            if cell.is_alive() {
                num_alive_cells += 1;
            }
        }
        num_alive_cells
    }

    pub fn get_alive_cells(&self) -> Vec<&Cell> {
        let flat_cells = self.flat();
        let mut alive_cells = Vec::new();
        for cell in flat_cells {
            if cell.is_alive() {
                alive_cells.push(cell);
            }
        }
        alive_cells
    }

    pub fn get_cells_with_energy(&self) -> Vec<&Cell> {
        let flat_cells = self.flat();
        let mut cells_with_energy = Vec::new();
        for cell in flat_cells {
            if cell.get_state() > 0 {
                cells_with_energy.push(cell);
            }
        }
        cells_with_energy
    }

    pub fn get_cells_with_energy_mut(&mut self) -> Vec<&mut Cell> {
        let flat_cells = self.flat_mut();
        let mut cells_with_energy = Vec::new();
        for cell in flat_cells {
            if cell.get_state() > 0 {
                cells_with_energy.push(cell);
            }
        }
        cells_with_energy
    }

    #[allow(dead_code)]
    pub fn print_state(&self) {
        let mut string: String = String::new();
        for y in 0..self.y_dim() {
            for x in 0..self.x_dim() {
                if self.get_cell(x, y).unwrap().is_alive() {
                    string.push('1');
                } else {
                    string.push('0');
                }
                if x == self.x_dim() - 1 {
                    string.push('\n');
                }
            }
        }
        println!("{}", string);
    }

    pub fn get_neighbors_vec(&self, cell: &Cell) -> Vec<&Cell> {
        let mut neighbors_vec = Vec::new();
        let cell_0_1 = self.get_cell(cell.x, cell.y + 1);
        if cell_0_1.is_ok() {
            neighbors_vec.push(cell_0_1.unwrap());
        }
        let cell_1_1 = self.get_cell(cell.x + 1, cell.y + 1);
        if cell_1_1.is_ok() {
            neighbors_vec.push(cell_1_1.unwrap());
        }
        let cell_1_0 = self.get_cell(cell.x + 1, cell.y);
        if cell_1_0.is_ok() {
            neighbors_vec.push(cell_1_0.unwrap());
        }
        if cell.y > 0 {
            let cell_1_m1 = self.get_cell(cell.x + 1, cell.y - 1);
            if cell_1_m1.is_ok() {
                neighbors_vec.push(cell_1_m1.unwrap());
            }
            let cell_0_m1 = self.get_cell(cell.x, cell.y - 1);
            if cell_0_m1.is_ok() {
                neighbors_vec.push(cell_0_m1.unwrap());
            }
            if cell.x > 0 {
                let cell_m1_m1 = self.get_cell(cell.x - 1, cell.y - 1);
                if cell_m1_m1.is_ok() {
                    neighbors_vec.push(cell_m1_m1.unwrap());
                }
            }
        }
        if cell.x > 0 {
            let cell_m1_0 = self.get_cell(cell.x - 1, cell.y);
            if cell_m1_0.is_ok() {
                neighbors_vec.push(cell_m1_0.unwrap());
            }
            let cell_m1_1 = self.get_cell(cell.x - 1, cell.y + 1);
            if cell_m1_1.is_ok() {
                neighbors_vec.push(cell_m1_1.unwrap());
            }
        }
        neighbors_vec
    }

    #[allow(dead_code)]
    pub fn compute_conways_game_of_life_single_threaded(&mut self) {
        let current_state = self.clone();
        for x in 0..self.x_dim() {
            for y in 0..self.y_dim() {
                let current_cell = current_state.get_cell(x, y).unwrap();
                let num_alive_neighbors = Self::count_alive_neighbours(&current_state, current_cell);
                if current_cell.is_alive() {
                    if num_alive_neighbors > 3 || num_alive_neighbors < 2 {
                        self.let_cell_age(x, y);
                    }
                } else if !current_cell.is_alive() {
                    if num_alive_neighbors == 3 {
                        self.revive_cell(x, y);
                    } else {
                        if current_cell.get_state() > 0 && current_cell.get_state() < 255 {
                            self.let_cell_age(x, y);
                        }
                    }
                }
            }
        }
    }

    fn count_alive_neighbours(space: &Space, cell: &Cell) -> usize {
        let mut num_alive_neighbors: usize = 0;
        let neighbors = space.get_neighbors_vec(cell);
        for neighbor in neighbors {
            if neighbor.is_alive() {
                num_alive_neighbors += 1;
            }
        }
        num_alive_neighbors
    }

    fn get_changes_by_conways_game_of_life_rules_par(cells: Vec<& Cell>, state_current: &Space) -> Vec<(u16, u16, CellAction)> {
        let changes: Vec<(u16, u16, CellAction)> = cells
            .par_iter()
            .filter_map(|cell| {
                let cell_current = state_current.get_cell(cell.x, cell.y).unwrap();
                let num_alive_neighbors = Self::count_alive_neighbours(&state_current, cell_current);
                if cell_current.is_alive() {
                    if num_alive_neighbors > 3 || num_alive_neighbors < 2 {
                        Some((cell.x, cell.y, CellAction::Age))
                    } else {
                        None
                    }
                } else if num_alive_neighbors == 3 {
                    Some((cell.x, cell.y, CellAction::Revive))
                } else if cell_current.get_state() > 0 && cell_current.get_state() < 255 {
                    Some((cell.x, cell.y, CellAction::Age))
                } else {
                    None
                }
            })
            .collect();
        changes
    }

    pub fn compute_conways_game_of_life_multithreaded(&mut self) {
        let state_current = self.clone();
        let flat: Vec<&Cell> = self.flat();
        let changes = Self::get_changes_by_conways_game_of_life_rules_par(flat, &state_current);
        for (x, y, action) in changes {
            match action {
                CellAction::Age => self.let_cell_age(x, y),
                CellAction::Revive => self.revive_cell(x, y),
            }
        }
    }
}

enum CellAction {
    Age,
    Revive,
}

#[derive(Debug)]
pub struct OutOfBoundsError {
    pub message: String,
}

impl OutOfBoundsError {
    fn new(message: &str) -> OutOfBoundsError {
        OutOfBoundsError { message: message.to_string() }
    }
}

impl fmt::Display for OutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for OutOfBoundsError {}

