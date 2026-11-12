use bevy::prelude::*;

use crate::board_chooser::{ NumsSolvedRes, SelectedCellRes }; // NumsSolvedRes, NumsMutRes, NumsFixedRes
use crate::states::*;


pub struct SolverPlugin;

impl Plugin for SolverPlugin {
    fn build(&self, app: &mut App) {
        app // .insert_resource(NumsSolvedRes {
            // solved_data: [0; 81],})
        .insert_resource(AvailLenRes {
            len_values: [0; 81],
        })
        // .add_systems(OnEnter(GameState::SolveBoard), avail_lengths)
        .add_systems(OnEnter(GameState::SolveBoard), solve_board) //  CONSIDER:  (solve_board, avail_lengths, populate_all_temps)
        // .add_systems(OnEnter(GameState::SolveBoard), related_nums) // related_nums
        // .add_systems(OnEnter(GameState::SolveBoard), available_nums_by_index)
        // .add_systems(OnEnter(GameState::SolveBoard), avail_lengths)
        // .add_systems(OnEnter(GameState::SolveBoard), show_solved)
        .add_systems(OnExit(GameState::SolveBoard), show_solved);
    }
}


#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AvailLenRes {
    pub len_values: [usize; 81],
}


impl AvailLenRes {
    pub fn new(length: [usize; 81]) -> AvailLenRes {
        AvailLenRes { len_values: length }
    }
}       


// System to initiate solving
pub fn solve_board(
    mut next_state: ResMut<NextState<GameState>>,
    mut solved_board: ResMut<NumsSolvedRes>,
) -> (){
    let mut working_board = solved_board.data.clone();

    if solve_recursive(&mut working_board) {
        info!("Board Solved Successfully");
        // Update the resource with the solved board
        solved_board.data = working_board;
        next_state.set(GameState::SpawnBoard);
    } else {
        error!("No solution found for this Sudoku Puzzle");
        next_state.set(GameState::Menu);
    }
}


// Recursive solver that works on a mutable array
pub fn solve_recursive(board: &mut [u8; 81]) -> bool {
    // Find the first empty cell
    for i in 0..81 {
        if board[i] == 0 {
            // Try each available number
            for available_num in available_nums_static(board, i) {
                board[i] = available_num;
                if solve_recursive(board) {
                    return true;
                }
                board[i] = 0; // Backtrack
            }
            return false; // No valid number found for this cell
        }
    }
    true // All cells filled - puzzle solved
}



// Helper function that works with a static array instead of a Resource
pub fn available_nums_static(board: &[u8; 81], ndx: usize) -> Vec<u8> {
    let related_cells: Vec<usize> = related_cells_by_index(ndx);
    let related_cell_values = related_cells
        .into_iter()
        .filter(|index| board[*index] != 0)
        .map(|index| board[index])
        .collect::<Vec<u8>>();

    let mut avail_nums = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    avail_nums.retain(|val| !related_cell_values.contains(val));
    avail_nums
}


pub fn show_solved(
    mut commands: Commands,
    solved_board: If<Res<NumsSolvedRes>>,
) -> () {
    let board = &solved_board.data.clone();
    eprintln!("Print Solved Board from solver.rs");
    for i in 0..9 {
        let start = i * 9;
        let end = start + 9;
        eprintln!("   {:?}", &board[start..end]);
    }
    commands.insert_resource(solved_board.clone());
    println!("\n");
    
}


// ? ? Remove this and just use available_nums_by_index() in solve_recursive() ? ?
// Consider if available_nums_by_index() should be modified to work with a static array instead of a Resource, to avoid borrowing issues in solve_recursive()
// Also Consider using this  & related_cells_by_index() function in available_nums_by_index() to find related cells and their values, instead of passing the entire board as a Resource, to avoid borrowing issues in solve_recursive()
// Consider versions from nav_board.rs (or nnav_board_values) vs versions from solver.rs - which should be used where, and should they be merged into one version that can work with both Resources and static arrays, 
// to avoid code duplication and borrowing issues in solve_recursive() ?
//
/// Available numbers for a specific cell specified by index (usize) - this version works with a Resource and is used in the Bevy systems
/// Find available numbers for a specific cell specified by Res<SelectedCellRes>
pub fn available_nums_by_select(
    solved_board: ResMut<NumsSolvedRes>,
    // current_board: &Res<NumsMutRes>,
    ndx: Res<SelectedCellRes>,
) -> Vec<u8> {
    let related_cells: Vec<usize> = related_cells_by_index(ndx.0);
    let related_cell_values = related_cells
        .into_iter()
        .filter(|ndx| solved_board.data[*ndx] != 0)
        .map(|ndex| solved_board.data[ndex])
        .collect::<Vec<u8>>();

    let mut avail_nums = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    avail_nums.retain(|val| !related_cell_values.contains(val));
    return avail_nums;
}


// Find available numbers for a specific index(usize)
pub fn available_nums_by_index(
    solved_board: &Res<NumsSolvedRes>,
    // current_board: &ResMut<NumsMutRes>,
    ndx: usize,
) -> Vec<u8> {
    // let mut working_board: Vec<u8> = NumsFixedRes { fixed_data: working_board.fixed_data.clone() }.fixed_data;
    // let index = ndx;
    let related_cells: Vec<usize> = related_cells_by_index(ndx);
    let related_cell_values = related_cells
        .into_iter()
        .filter(|index| solved_board.data[*index as usize] != 0)
        .map(|index| solved_board.data[index as usize])
        .collect::<Vec<u8>>();
    // for related_vec_values: Vec<usize> in &related_cell_values.iter() {
    //     related_vec_values.len() =
    // }

    let mut avail_nums = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    avail_nums.retain(|val| !related_cell_values.contains(val));
    return avail_nums;
}
//

//  X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X
/*
pub fn avail_lengths(
    mut commands: Commands,
    // current_board: ResMut<NumsMutRes>,
    solved_board: ResMut<NumsSolvedRes>,
    len_values: ResMut<AvailLenRes>,
) -> () {
    let mut avail_lengths: [usize; 81] = [0; 81];
    for index in 0..81 {
        // let available_nums = available_nums_by_index(&current_board, index);
        avail_lengths[index] = available_nums_by_index(&solved_board, index).len();
        // commands.insert_resource(AvailLenRes{len_values: avail_lengths.clone()});
    }
    commands.insert_resource(AvailLenRes{len_values: len_values.len_values.clone()});
    //len_values = *avail_lengths;
    eprintln!("Available Lengths per Cell: {:?}", avail_lengths);
}
*/



//  X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X X
pub fn related_cells_by_index(index: usize) -> Vec<usize> {
    let row_ndxes = (index / 9 * 9..index / 9 * 9 + 9)
        .filter(|r| *r != index)
        .collect::<Vec<usize>>();

    let col_ndxes: Vec<usize> = (0..9)
        .map(|i| i * 9 + (index % 9))
        .filter(|c| *c != index)
        .collect();
    //

    // let cube_base_row = ndx / 27 * 3;  &  let cube_base_col = ndx % 9 / 3 * 3;
    let cube_rows = (index / 27 * 3..index / 27 * 3 + 3)
        .filter(|r| *r != index / 9)
        .collect::<Vec<usize>>();

    let cube_cols = (index % 9 / 3 * 3..index % 9 / 3 * 3 + 3)
        .filter(|c| *c != index % 9)
        .collect::<Vec<usize>>();

    let cube_ndxes = cube_rows
        .iter()
        .flat_map(|cube_row| {
            cube_cols
                .iter()
                .map(move |cube_col| cube_row * 9 + cube_col)
        })
        .collect::<Vec<usize>>();

    // chain() to combine row_ndxes, col_ndxes, and cube_ndxes into a single vector
    let related_cells = row_ndxes
        .into_iter()
        .chain(col_ndxes)
        .chain(cube_ndxes)
        .collect::<Vec<usize>>();

    return related_cells;
}



/*
pub fn board_generator(
    zero_board: [0_u8; 81],
    ramd

) -> [u8; 81] {
    // Placeholder for board generation logic
    // This function should generate a valid Sudoku board and return it as an array of 81 u8 values
    [0; 81] // Replace with actual generated board
}
*/