#[cfg(test)]

mod tests {
    use std::time::Instant;
    use crate::space::{Space, io};

    #[test]
    fn space_works() {
        let mut space = Space::new(100, 100);
        assert_eq!(100, space.x_dim());
        assert_eq!(100, space.y_dim());
        {
            // borrow mutable cell and revive within this scope
            let cell = space.get_cell_mut(0, 0).unwrap();
            assert_eq!(false, cell.is_alive());
            cell.revive();
            assert_eq!(true, cell.is_alive());
        }
        // check if cell is actually revived in space
        assert_eq!(true, space.get_cell(0, 0).unwrap().is_alive());
        {
            // kill cell with borrowed mutable
            let cell = space.get_cell_mut(0, 0).unwrap();
            assert_eq!(true, cell.is_alive());
            cell.kill();
        }
        // check if its actually dead
        assert_eq!(false, space.get_cell(0, 0).unwrap().is_alive());
        space.revive_random_cells(100);
        assert_eq!(100, space.get_num_alive_cells());
        space.kill_all_cells();
        assert_eq!(0, space.get_num_alive_cells());
    }

    #[test]
    fn revive_random_cells_works() {
        let mut space = Space::new(100, 100);
        space.revive_random_cells(100);
        assert_eq!(100, space.get_num_alive_cells());
    }

    #[test]
    fn build_space_from_array_works() {
        let array: Vec<Vec<u8>> = vec![
            vec![0, 1, 0, 0],
            vec![1, 0, 0, 0],
            vec![0, 0, 0, 1],
        ];
        let space = Space::build_from_array(&array);
        assert_eq!(4, space.x_dim());
        assert_eq!(3, space.y_dim());
        assert_eq!(false, space.get_cell(0, 0).unwrap().is_alive());
        assert_eq!(true, space.get_cell(1, 0).unwrap().is_alive());
        assert_eq!(true, space.get_cell(0, 1).unwrap().is_alive());
        assert_eq!(false, space.get_cell(1, 1).unwrap().is_alive());
        assert_eq!(false, space.get_cell(1, 2).unwrap().is_alive());
        assert_eq!(true, space.get_cell(3, 2).unwrap().is_alive());
    }

    #[test]
    fn get_cell_out_of_bounds_panics() {
        let space = Space::new(10, 10);
        let result = space.get_cell(10, 10);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Index out of bounds!");
    }

    #[test]
    fn get_moore_neighbourhood_works() {
        let space = Space::new(3, 3);
        let cell = space.get_cell(1, 1).unwrap();
        let neighbors = space.get_neighbors_vec(cell);
        assert_eq!(neighbors.len(), 8);
    }

    #[test]
    fn save_and_load_states_work() {
        let mut space = io::read_from_file("resources/glider.space").unwrap();
        let cloned_space = space.clone();
        let alive_cells_of_gen_0 = cloned_space.get_alive_cells();
        for _ in 1..2 {
            space.compute_conways_game_of_life_multithreaded();
        }
        space.load_state(0);
        let alive_cells_of_loaded_gen_0 = space.get_alive_cells();
        for i in 0..alive_cells_of_gen_0.len() {
            assert_eq!(alive_cells_of_gen_0[i], alive_cells_of_loaded_gen_0[i]);
        }
    }

    #[test]
    fn conways_game_of_life_rule_1_works() {
        let mut space = Space::build_from_array(
            &vec![vec![0, 0, 0],
                        vec![0, 1, 1],
                        vec![0, 0, 0]]
        );
        println!("----------------------------------");
        println!("Rule 1: Dying is wonderful");
        println!("----------------------------------");
        println!("Generation: 0");
        space.print_state();
        println!("Generation: 1");
        space.compute_conways_game_of_life_multithreaded();
        space.print_state();
        assert_eq!(false, space.check_cell_is_alive(1, 1));
        assert_eq!(false, space.check_cell_is_alive(1, 1));
    }

    #[test]
    fn conways_game_of_life_rule_2_works() {
        let mut space = Space::build_from_array(
    &vec![vec![1, 0, 0],
                vec![0, 1, 0],
                vec![0, 0, 1]]
        );
        println!("----------------------------------");
        println!("Rule 2: Staying Alive");
        println!("----------------------------------");
        println!("Generation: 0");
        space.print_state();
        println!("Generation: 1");
        space.compute_conways_game_of_life_multithreaded();
        space.print_state();
        assert_eq!(false, space.check_cell_is_alive(0, 0));
        assert_eq!(true, space.check_cell_is_alive(1, 1));
        assert_eq!(false, space.check_cell_is_alive(2, 2));
    }

    #[test]
    fn conways_game_of_life_rule_3_works() {
        let mut space = Space::build_from_array(
    &vec![vec![1, 1, 0],
                vec![0, 1, 1],
                vec![0, 1, 0]]
        );
        println!("----------------------------------");
        println!("Rule 3: Overpopulation");
        println!("----------------------------------");
        println!("Generation: 0");
        space.print_state();
        println!("Generation: 1");
        space.compute_conways_game_of_life_multithreaded();
        space.print_state();
        assert_eq!(6, space.get_num_alive_cells());
        assert_eq!(false, space.get_cell(1, 1).unwrap().is_alive());
    }

    #[test]
    fn conways_game_of_life_rule_4_works() {
        let mut space = Space::build_from_array(
    &vec![vec![1, 0, 0],
                vec![0, 0, 1],
                vec![0, 1, 0]]
        );
        println!("----------------------------------");
        println!("Rule 4: New Life");
        println!("----------------------------------");
        println!("Generation: 0");
        space.print_state();
        println!("Generation: 1");
        space.compute_conways_game_of_life_multithreaded();
        space.print_state();
        assert_eq!(true, space.check_cell_is_alive(1, 1));
        assert_eq!(1, space.get_num_alive_cells())
    }

    #[test]
    fn conways_game_of_life_oscillator_works() {
        let mut space = Space::build_from_array(
    &vec![vec![0, 1, 0],
                vec![0, 1, 0],
                vec![0, 1, 0]]
        );
        println!("----------------------------------");
        println!("Oscillator");
        println!("----------------------------------");
        println!("Generation: 0");
        space.print_state();
        println!("Generation: 1");
        space.compute_conways_game_of_life_multithreaded();
        space.print_state();
        assert_eq!(true, space.check_cell_is_alive(0, 1));
        assert_eq!(true, space.check_cell_is_alive(1, 1));
        assert_eq!(true, space.check_cell_is_alive(2, 1));
        println!("Generation: 2");
        space.compute_conways_game_of_life_multithreaded();
        space.print_state();
        assert_eq!(true, space.check_cell_is_alive(1, 0));
        assert_eq!(true, space.check_cell_is_alive(1, 1));
        assert_eq!(true, space.check_cell_is_alive(1, 2));
    }

    #[test]
    fn io_write_to_file_works() {
        let space = Space::build_from_array(&vec![
            vec![0, 1, 0, 0],
            vec![1, 0, 0, 0],
            vec![0, 0, 0, 1],
            vec![0, 0, 0, 0],
            vec![0, 1, 0, 0]
        ]);
        let result = io::write_to_file("resources/io_test.space", &space);
        assert_eq!(true, result.is_ok());
    }

    #[test]
    fn io_read_from_file_works() {
        let space = io::read_from_file("resources/io_test.space").unwrap();
        assert_eq!(true, space.get_cell(1, 0).unwrap().is_alive());
        assert_eq!(true, space.get_cell(0, 1).unwrap().is_alive());
        assert_eq!(true, space.get_cell(3, 2).unwrap().is_alive());
        assert_eq!(true, space.get_cell(1, 4).unwrap().is_alive());
        assert_eq!(4, space.get_num_alive_cells());
    }

    #[test]
    fn kill_all_cells_works() {
        let mut space = Space::new(200, 300);
        space.revive_random_cells(1200);
        let start = Instant::now();
        space.kill_all_cells();
        let duration = start.elapsed();
        let duration_ms = duration.as_millis();
        println!("Killing all cells in {:?} ms", duration_ms);
        assert_eq!(0, space.get_num_alive_cells());
    }

    #[test]
    fn cell_aging_works() {
        println!("----------------------------------");
        println!("Cell aging");
        println!("----------------------------------");
        let mut space = Space::build_from_array(&vec![
            vec![0, 0, 0],
            vec![0, 1, 0],
            vec![0, 0, 0],
        ]);
        assert_eq!(true, space.check_cell_is_alive(1, 1));
        println!("Energy(1,1) in t=0: {}", space.get_cell(1, 1).unwrap().get_state());
        assert_eq!(255, space.get_cell(1, 1).unwrap().get_state());
        for i in 1..16 {
            space.compute_conways_game_of_life_multithreaded();
            assert_eq!(false, space.check_cell_is_alive(1, 1));
            println!("Energy(1,1) in t={}: {}", i, space.get_cell(1, 1).unwrap().get_state());
            assert_eq!(255 - (17 * i), space.get_cell(1, 1).unwrap().get_state());
        }
        assert_eq!(0, space.get_cell(1, 1).unwrap().get_state());
    }

    #[test]
    fn full_hd_grid_performance_test() {
        println!("----------------------------------");
        println!("Performance Test");
        println!("----------------------------------");
        let mut space = Space::new(200, 110);
        space.revive_random_cells(50000);
        let start = Instant::now();
        let mut duration = start.elapsed().as_millis();
        let mut iterations = 0;
        while duration < 1000 {
            space.compute_conways_game_of_life_multithreaded();
            duration = start.elapsed().as_millis();
            iterations += 1;
        }
        println!("A grid of the size 200x110 with a Cell size of 9.5 must be able to calculate at least 30 iterations per second.");
        println!("iterations per 1 s: {}", iterations);
        assert!(iterations >= 30);
    }
}
