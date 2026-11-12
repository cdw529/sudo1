//
use bevy::picking::events::{Click, Out, Over, Pointer};
use bevy::prelude::*;
// use bevy::picking::{ Pickable, events::{Click, Out, Over, Pointer} };
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;

use crate::board_chooser::{
    BoardRevision, EditMode, HoveredCellRes, NumsFixedRes, NumsMutRes, SelectedCellRes,
}; // NumsSolvedRes
use crate::setup_board::AllTempsRes;
use crate::solver::related_cells_by_index;
use crate::states::*;
use crate::sudoku_constants::{
    BORDER_FIXED_NUMS, BORDER_HOVER_NUMS, BORDER_HOVER_TMPS, BORDER_MUT_NUMS, BORDER_NORM_NUMS,
    BORDER_NORM_TMPS, BORDER_WHITE,
};

pub struct HoverSystemsPlugin;

impl Plugin for HoverSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredCellRes>()
            .init_resource::<EditMode>()
            .init_resource::<BoardRevision>()
            .add_message::<SudokuIntent>()
            .add_message::<SudokuOutcome>()
            .configure_sets(
                Update,
                (
                    InputProcessingSet::CollectInput,
                    InputProcessingSet::ApplyRules,
                    InputProcessingSet::ProjectUi,
                )
                    .chain()
                    .run_if(
                        in_state(GameState::NavBoard)
                            .or(in_state(GameState::EditTemps))
                            .or(in_state(GameState::NoUpdate))
                            .or(in_state(GameState::EditPerms0))
                            .or(in_state(GameState::EditPerms1)),
                    ),
            )
            .add_systems(
                Update,
                (collect_cell_interaction_intents, collect_keyboard_intents)
                    .in_set(InputProcessingSet::CollectInput),
            )
            .add_systems(
                Update,
                apply_sudoku_intents.in_set(InputProcessingSet::ApplyRules),
            )
            .add_systems(
                Update,
                project_cell_borders.in_set(InputProcessingSet::ProjectUi),
            );
    }
}
//

//
// Consider renaming to "InputPipelineSet" or "InteractionProcessingSet"
// RENAMED "InputProcessingSet" from "HoverSystemsSet" to better reflect the purpose and scope,
// which encompasses the entire flow of processing player input and updating the game state accordingly,
// including collecting input, applying game rules, and projecting UI changes based on player actions.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputProcessingSet {
    CollectInput, // pointer + keyboard => SudokuIntent ? InputEvents, InputIntents
    ApplyRules, // consume intents, mutate board/resources  ? InputsToWrites, InputActions, GaePlayActions, ApplyActions, Inputs2Actions, InputsToActions
    ProjectUi,  // border/highlight/message projections  ? UpdateUi, UiMessagesm
}

// single intent stream for gameplay input:  To be used by the various systems that need to respond to player input,
// such as nav_board_system(), edit_perms1(), edit_temps(), etc.
// This will allow for a more organized and centralized handling of player input,
// and can help to avoid conflicts between different systems trying to read the same input events directly from the keyboard or mouse.
// CONSIDER:  name change: "PlayerInput" or "GameplayIntent",  whether to split into separate intent streams for pointer and keyboard input, if needed, to further organize and manage different types of player input more effectively.
#[derive(Message, Debug, Clone, Copy)]
pub enum SudokuIntent {
    // ? InputActions, PlayerIntent, GameplayIntent
    SelectCell(usize),
    HoverCell(Option<usize>),
    EnterMainMode,
    EnterDigit(u8),
    ToggleTemp(u8),
    ClearCell,
    DeselectCell,
    RequestMenu,
    RequestSolve,
}

// Result stream for gameplay actions:
// This will allow for a more organized and centralized handling of the outcomes of player actions,
// such as whether a cell was selected, whether the board state changed, or whether a move was invalid
// To be used by the various systems that need to respond to the outcomes of player actions,
#[derive(Message, Debug, Clone, Copy)]
pub enum SudokuOutcome {
    CellSelected { ndx: usize, editable: bool },
    BoardChanged,
    InvalidMove { ndx: usize, digit: u8 },
}

// cell_interaction_system() + collect_keyboard_intents() => collect_cell_interaction_intents() => apply_sudoku_intents() => project_cell_borders()
// Collects player input from both pointer interactions and keyboard events,
// and translates them into high-level gameplay intents that can be consumed by other systems to update the game state and UI accordingly.
pub fn collect_cell_interaction_intents(
    hovered: Res<HoveredCellRes>,
    mut intents: MessageWriter<SudokuIntent>,
    interaction_query: Query<(&Interaction, &Ndx), (Changed<Interaction>, With<Button>)>,
) {
    let mut pressed_cell = None;
    let mut hovered_cell = None;
    let mut clear_hover = false;

    for (interaction, ndx) in &interaction_query {
        match *interaction {
            Interaction::Pressed => {
                pressed_cell = Some(ndx.0);
            }
            Interaction::Hovered => {
                // hovered_cell != Some(ndx.0);
                hovered_cell = Some(ndx.0);
            }
            Interaction::None => {
                if hovered.0 == Some(ndx.0) {
                    clear_hover = true;
                }
            }
        }
    }
    if let Some(index) = hovered_cell {
        intents.write(SudokuIntent::HoverCell(Some(index)));
    } else if clear_hover {
        intents.write(SudokuIntent::HoverCell(None));
    }

    if let Some(index) = pressed_cell {
        intents.write(SudokuIntent::SelectCell(index));
    }
}

//

fn resolve_cell_target(
    entity: Entity,
    ndx_query: &Query<&Ndx>,
    child_of_query: &Query<&ChildOf>,
) -> Option<(Entity, usize)> {
    let mut current = Some(entity);

    while let Some(entity) = current {
        if let Ok(ndx) = ndx_query.get(entity) {
            return Some((entity, ndx.0));
        }
        current = child_of_query
            .get(entity)
            .ok()
            .map(|child_of| child_of.parent());
    }
    None
}

//
pub fn collect_keyboard_intents(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mode: Res<EditMode>,
    selected: Res<SelectedCellRes>,
    hovered: Res<HoveredCellRes>,
    mut intents: MessageWriter<SudokuIntent>,
) {
    let selected_is_hovered = hovered.0 == Some(selected.0);
    let has_active_edit = !matches!(*mode, EditMode::Navigate) && selected_is_hovered;

    for key in keyboard_input.get_just_pressed() {
        if let Some(digit) = key_to_digit(*key) {
            match *mode {
                EditMode::TempNotes if selected_is_hovered => {
                    intents.write(SudokuIntent::ToggleTemp(digit));
                }
                EditMode::MainNumber | EditMode::LockedCell if selected_is_hovered => {
                    intents.write(SudokuIntent::EnterDigit(digit));
                }
                // EditMode::Navigate => {}
                _ => {}
            }
            continue;
        }
        match key {
            KeyCode::Escape => {
                if has_active_edit {
                    intents.write(SudokuIntent::ClearCell);
                }
            }
            KeyCode::KeyP => {
                if has_active_edit {
                    intents.write(SudokuIntent::EnterMainMode);
                }
            }
            KeyCode::KeyN => {
                if has_active_edit {
                    intents.write(SudokuIntent::DeselectCell);
                }
            }
            KeyCode::KeyM => {
                intents.write(SudokuIntent::RequestMenu);
            }
            KeyCode::KeyS => {
                intents.write(SudokuIntent::RequestSolve);
            }
            _ => {}
        }
    }
}
//

// Consider "process_intents_system", to handle the application of player intents to the game state, such as selecting a cell, entering a digit, toggling temp notes, etc.,
//      and then projecting the resulting changes to the UI, such as updating cell borders, highlighting related cells, displaying messages, etc.
// This could help to further organize and centralize the handling of player input and its effects on the game state and UI,
//      and could allow for more complex interactions and feedback based on player actions.
pub fn apply_sudoku_intents(
    mut intents: MessageReader<SudokuIntent>,
    mut outcomes: MessageWriter<SudokuOutcome>,
    start_board: Res<NumsFixedRes>,
    mut current_board: ResMut<NumsMutRes>,
    mut all_temps: ResMut<AllTempsRes>,
    mut selected: ResMut<SelectedCellRes>,
    mut hovered: ResMut<HoveredCellRes>,
    mut edit_mode: ResMut<EditMode>,
    mut board_revision: ResMut<BoardRevision>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for intent in intents.read() {
        match *intent {
            SudokuIntent::SelectCell(index) => {
                selected.0 = index;
                hovered.0 = Some(index);
                if start_board.fixed_data[index] != 0 {
                    *edit_mode = EditMode::LockedCell;
                    next_state.set(GameState::EditPerms0);
                    outcomes.write(SudokuOutcome::CellSelected {
                        ndx: index,
                        editable: false,
                    });
                } else if current_board.mutable_data[index] != 0 {
                    *edit_mode = EditMode::MainNumber;
                    next_state.set(GameState::EditPerms1);
                    outcomes.write(SudokuOutcome::CellSelected {
                        ndx: index,
                        editable: true,
                    });
                } else if current_board.mutable_data[index] == 0 {
                    *edit_mode = EditMode::TempNotes;
                    next_state.set(GameState::EditTemps);
                    outcomes.write(SudokuOutcome::CellSelected {
                        ndx: index,
                        editable: true,
                    });
                } else {
                    hovered.0 = None;
                    *edit_mode = EditMode::Navigate;
                    next_state.set(GameState::NavBoard);
                    outcomes.write(SudokuOutcome::CellSelected {
                        ndx: index,
                        editable: true,
                    });
                }
            }
            SudokuIntent::HoverCell(index) => {
                hovered.0 = index;
                if *edit_mode != EditMode::Navigate && hovered.0 != Some(selected.0) {
                    *edit_mode = EditMode::Navigate;
                    next_state.set(GameState::NavBoard);
                }
            }
            SudokuIntent::EnterMainMode => {
                let index = selected.0;
                if hovered.0 != Some(selected.0) {
                    *edit_mode = EditMode::Navigate;
                    next_state.set(GameState::NavBoard);
                    continue;
                }

                if start_board.fixed_data[index] != 0 {
                    *edit_mode = EditMode::LockedCell;
                    next_state.set(GameState::EditPerms0);
                } else {
                    *edit_mode = EditMode::MainNumber;
                    next_state.set(GameState::EditPerms1);
                }
            }
            SudokuIntent::EnterDigit(digit) => {
                let index = selected.0;
                if hovered.0 != Some(selected.0) {
                    *edit_mode = EditMode::Navigate;
                    next_state.set(GameState::NavBoard);
                    continue;
                }

                if start_board.fixed_data[index] != 0 {
                    *edit_mode = EditMode::LockedCell;
                    next_state.set(GameState::EditPerms0);
                    outcomes.write(SudokuOutcome::InvalidMove { ndx: index, digit });
                    continue;
                }

                let available_nums = available_nums_by_board_and_index(&current_board, index);
                if available_nums.contains(&digit) {
                    current_board.mutable_data[index] = digit;
                    all_temps.temps_data[index].clear();

                    for related_index in related_cells_by_index(index) {
                        all_temps.temps_data[related_index].retain(|value| *value != digit);
                    }

                    board_revision.0 += 1;
                    outcomes.write(SudokuOutcome::BoardChanged);
                    next_state.set(GameState::SpawnBoard);
                } else {
                    outcomes.write(SudokuOutcome::InvalidMove { ndx: index, digit });
                    next_state.set(GameState::NoUpdate);
                }
            }
            SudokuIntent::ToggleTemp(digit) => {
                let index = selected.0;
                if hovered.0 != Some(selected.0) {
                    *edit_mode = EditMode::Navigate;
                    next_state.set(GameState::NavBoard);
                    continue;
                }
                if start_board.fixed_data[index] != 0 || current_board.mutable_data[index] != 0 {
                    outcomes.write(SudokuOutcome::InvalidMove { ndx: index, digit });
                    continue;
                }
                let available_nums = available_nums_by_board_and_index(&current_board, index);
                if !available_nums.contains(&digit) {
                    outcomes.write(SudokuOutcome::InvalidMove { ndx: index, digit });
                    next_state.set(GameState::NoUpdate);
                    continue;
                }
                let temps = &mut all_temps.temps_data[index];
                if temps.contains(&digit) {
                    temps.retain(|value| *value != digit);
                } else {
                    temps.push(digit);
                    temps.sort_unstable();
                }
                board_revision.0 += 1;
                outcomes.write(SudokuOutcome::BoardChanged);
                next_state.set(GameState::SpawnBoard);
            }
            SudokuIntent::ClearCell => {
                let index = selected.0;
                if hovered.0 != Some(selected.0) {
                    *edit_mode = EditMode::Navigate;
                    next_state.set(GameState::NavBoard);
                    continue;
                }

                if start_board.fixed_data[index] != 0 {
                    *edit_mode = EditMode::LockedCell;
                    next_state.set(GameState::EditPerms0);
                    continue;
                }

                if current_board.mutable_data[index] != 0 {
                    current_board.mutable_data[index] = 0;
                }
                all_temps.temps_data[index].clear();

                board_revision.0 += 1;
                outcomes.write(SudokuOutcome::BoardChanged);
                next_state.set(GameState::SpawnBoard);
            }
            SudokuIntent::DeselectCell => {
                hovered.0 = None;
                *edit_mode = EditMode::Navigate;
                next_state.set(GameState::NavBoard);
            }
            SudokuIntent::RequestMenu => {
                next_state.set(GameState::Menu);
            }
            SudokuIntent::RequestSolve => {
                next_state.set(GameState::SolveBoard);
            }
        }
    }
}

pub fn project_cell_borders(
    selected: Res<SelectedCellRes>,
    hovered: Res<HoveredCellRes>,
    edit_mode: Res<EditMode>,
    start_board: Res<NumsFixedRes>,
    current_board: Res<NumsMutRes>,
    game_state: Res<State<GameState>>,
    mut border_query: Query<(&Ndx, &mut BorderColor)>,
) {
    for (ndx, mut border) in &mut border_query {
        // let index = ndx.0;  // replace index with ndx.0
        let is_selected = *edit_mode != EditMode::Navigate && selected.0 == ndx.0; // index

        let border_color = if is_selected {
            if start_board.fixed_data[ndx.0] != 0 {
                // index
                BORDER_FIXED_NUMS
            } else if current_board.mutable_data[ndx.0] != 0 {
                // index
                BORDER_MUT_NUMS
            } else if current_board.mutable_data[ndx.0] == 0
                && *game_state.get() == GameState::EditPerms1
            {
                //   ? ? REMOVE ? ?
                BORDER_MUT_NUMS // BORDER_WHITE // BORDER_PRESS_MUT
            } else {
                BORDER_WHITE
            }
        } else if hovered.0 == Some(ndx.0) {
            if start_board.fixed_data[ndx.0] != 0 {
                BORDER_HOVER_NUMS
            } else if current_board.mutable_data[ndx.0] != 0 {
                BORDER_MUT_NUMS
            } else {
                BORDER_HOVER_TMPS
            }
        } else if start_board.fixed_data[ndx.0] != 0 || current_board.mutable_data[ndx.0] != 0 {
            BORDER_NORM_NUMS
        } else {
            BORDER_NORM_TMPS
        };

        *border = BorderColor::all(border_color);
    }
}
//

// tuple struct to hold the index of each cell, for use in the nav_board_system() query parameters to identify which cell is being hovered or clicked
#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Ndx(pub usize);

#[derive(Resource, Debug, Deref, DerefMut, Clone)]
pub struct AvailableNumsRes {
    pub avail_nums: Vec<u8>,
}

impl AvailableNumsRes {
    pub fn new(avail_nums: Vec<u8>) -> Self {
        // AvailableNumsRes
        AvailableNumsRes { avail_nums }
    }
}

/*
#[derive(Component)]
pub struct OnNavBoard;  // Tag Component to identify entities spawned in nav_board_system() for Despawn on Exit from GameState::NavBoard
*///

// consider rename to "cell_interaction_system" or "player_input_system", as it handles both pointer and keyboard input related to cell interactions,
//      and translates them into gameplay intents that can be consumed by other systems to update the game state and UI accordingly.
pub fn cell_interaction_system(
    mut clicks: MessageReader<Pointer<Click>>,
    mut overs: MessageReader<Pointer<Over>>,
    mut outs: MessageReader<Pointer<Out>>,
    ndx_query: Query<&Ndx>,
    child_of_query: Query<&ChildOf>,
    start_board: Res<NumsFixedRes>,
    current_board: Res<NumsMutRes>,
    mut selected: ResMut<SelectedCellRes>,
    mut next_state: ResMut<NextState<GameState>>,
    mut border_query: Query<&mut BorderColor>,
) -> () {
    // ── CLICK ─────────────────────────────────────────────────────────────
    for click in clicks.read() {
        if let Some((_, i)) = resolve_cell_target(click.entity, &ndx_query, &child_of_query) {
            selected.0 = i; // keep your existing SelectedCellRes

            if start_board.fixed_data[i] != 0 {
                next_state.set(GameState::EditPerms0);
            } else if current_board.mutable_data[i] != 0 && start_board.fixed_data[i] == 0 {
                // ? ? else if current_board.mutable_data[i] != 0 {
                next_state.set(GameState::EditPerms1);
            } else {
                next_state.set(GameState::EditTemps);
            }
        }
    }
    // ── HOVER (visual feedback) ───────────────────────────────────────────
    for over in overs.read() {
        if let Some((cell_entity, i)) =
            resolve_cell_target(over.entity, &ndx_query, &child_of_query)
        {
            if let Ok(mut border) = border_query.get_mut(cell_entity) {
                if start_board.fixed_data[i] != 0 {
                    *border = BorderColor::all(BORDER_HOVER_NUMS);
                } else if current_board.mutable_data[i] != 0 {
                    *border = BorderColor::all(BORDER_WHITE);
                } else {
                    *border = BorderColor::all(BORDER_HOVER_TMPS);
                }
            }
        }
    }
    // ── MOUSE OUT (reset to normal border) ────────────────────────────────
    for out in outs.read() {
        if let Some((cell_entity, i)) = resolve_cell_target(out.entity, &ndx_query, &child_of_query)
        {
            if let Ok(mut border) = border_query.get_mut(cell_entity) {
                if start_board.fixed_data[i] != 0 {
                    *border = BorderColor::all(BORDER_NORM_NUMS);
                } else if current_board.mutable_data[i] != 0 {
                    *border = BorderColor::all(BORDER_NORM_NUMS);
                } else {
                    *border = BorderColor::all(BORDER_NORM_TMPS);
                }
            }
        }
    }
}

pub fn edit_perms0(
    mut commands: Commands,
    // mut next_state: ResMut<NextState<GameState>>,
    // mut state: Res<State<GameState>>,
) {
    commands.set_state(GameState::NavBoard);
}

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ///////////////////////   EDIT CELL (TEMPS) FUNCTION   /////////////////////////////////////////////////////////////
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn edit_temps(
    mut commands: Commands,
    ndx: ResMut<SelectedCellRes>,
    mut all_temps: ResMut<AllTempsRes>,
    current_board: ResMut<NumsMutRes>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    // mut displayed_temps: ResMut<AllTempsDisplayedRes>,
    // mut all_temps: ResMut<AllTempsDisplayedRes>,
) -> () {
    let index = ndx.0;
    let avail_nums = available_nums_by_board_and_index(&current_board, ndx.0);
    let mut temps_hold = all_temps.temps_data[index].clone(); // Temporarly assign value of temps Vec<u8> to temps_hold before assigning back ad update for  AllTempsRes
    // let mut displayed_temps: Vec<usize> = temps_2.iter().map(|&num| num as usize).collect();
    for key in keyboard_input.get_just_pressed() {
        if let Some(digit) = key_to_digit(*key) {
            eprintln!("\nedit_temps, Ln: 310 => Pressed digit: = {:?}", digit);
            if avail_nums.contains(&digit) && !temps_hold.contains(&digit) {
                eprintln!(
                    "edit_temps Ln: 282:  add digit to temps if digit is valid & not yet assigned to Temps"
                );
                temps_hold.push(digit);
                temps_hold.sort();
                all_temps.temps_data[index] = temps_hold.clone(); // // Assign temps_2 (? temps_hold, rather than temps_2),  back to AllTempsRes at index
                // temps_data[index] = temps_hold.clone();
                // displayed_temps.push(index); // Add index of Cell to displayed_temps to trigger display of Temps for this cell in spawn_cells system // spawn_board_frame
                eprintln!(
                    "\nedit_temps:  Line 320 => Updated Temps at index {}: to {:?}",
                    index, all_temps.temps_data[index]
                );
                commands.insert_resource(AllTempsRes {
                    temps_data: all_temps.temps_data.clone(),
                }); // Update the AllTempsRes Resource

                // next_state.set(GameState::EditTemps);  //  GameState::SpawnBoard, GameState::NavBoard
            }
        }
        // next_state.set(GameState::NavBoard);   // See also 326,
        next_state.set(GameState::EditTemps);
    }
}

//  Check next_state.set() ln 282 & 285 above
fn key_to_digit(key: KeyCode) -> Option<u8> {
    match key {
        KeyCode::Digit1 | KeyCode::Numpad1 => Some(1),
        KeyCode::Digit2 | KeyCode::Numpad2 => Some(2),
        KeyCode::Digit3 | KeyCode::Numpad3 => Some(3),
        KeyCode::Digit4 | KeyCode::Numpad4 => Some(4),
        KeyCode::Digit5 | KeyCode::Numpad5 => Some(5),
        KeyCode::Digit6 | KeyCode::Numpad6 => Some(6),
        KeyCode::Digit7 | KeyCode::Numpad7 => Some(7),
        KeyCode::Digit8 | KeyCode::Numpad8 => Some(8),
        KeyCode::Digit9 | KeyCode::Numpad9 => Some(9),
        _ => None,
    }
}

///  Run on exit Perms1:  Updates the AllDataRes Resource for related_cells following addition of a NEWLY ADDED fixed_number)
//   Only updates cells that need to change by removing the newly added number if it currently exists in their temps list
pub fn update_related_temps(
    mut commands: Commands,
    ndx: ResMut<SelectedCellRes>,
    current_board: ResMut<NumsMutRes>,
    mut all_temps: ResMut<AllTempsRes>,
    // mut next_state: ResMut<NextState<GameState>>
) -> () {
    let indx = ndx.0;
    // let index_num = current_board.mutable_data[indx];
    // let related_cell_indices = related_cells(&ndx);
    let num = current_board.mutable_data[indx];
    // for related_cell_index in related_cell_indices {
    for related_cell_index in related_cells(&ndx) {
        let mut temps = all_temps.temps_data[related_cell_index].clone();
        for i in 0..temps.len() {
            // C
            if temps[i] == num {
                temps[i] = 0;
            }
        }
        temps.sort();
        // temps.truncate(6); // TRUNCATE the length of temps to a maximum of 6, to prevent excessive temps from being added to any cell, which could cause display issues in spawn_cells system // spawn_board_frame
        //
        all_temps.temps_data[related_cell_index] = temps; // Consider:  temps.clone(),   Update the temps for the related cell index in AllTempsRes
    }
    commands.insert_resource(AllTempsRes {
        temps_data: all_temps.temps_data.clone(),
    });
    // next_state.set(GameState::SpawnBoard);  // SpawnBoard, NavBoard,
    // next_state.set(GameState::NavBoard);   //  ? ? ?  NEXT_STATE  ? ? ?
    // Consider whether to set next_state to SpawnBoard or NavBoard, only if something waw actually updated
    // or perhaps let t system that called update_related_temps() to set the next_state, as needed, to avoid unnecessary state changes and redraws of the board
}

// //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ///////////////////////   Edit Perms1 Function   //////////////////////////////////////////////////////////////////                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             n                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          n
// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn edit_perms1(
    // BORDER_WHITE - OnEnter(GameState::EditPerms1)
    mut commands: Commands,
    mut current_board: If<ResMut<NumsMutRes>>, // If<ResMut<NumsMutRes>>
    ndx: ResMut<SelectedCellRes>,
    mut all_temps: If<ResMut<AllTempsRes>>,
    keyboard_input: If<Res<ButtonInput<KeyCode>>>,
    mut next_state: If<ResMut<NextState<GameState>>>,
) -> () {
    let index: usize = ndx.0;
    let related_indices = related_cells(&ndx);
    let previous_num = current_board.mutable_data[ndx.0]; // Capture the number being removed from current_board before resetting
    let avail_nums = available_nums_by_board_and_index(&current_board, ndx.0); // Must be in this list to be VALID
    for key in keyboard_input.get_just_pressed() {
        if let Some(digit) = key_to_digit(*key) {
            // eprintln!("edit_perms1, Ln 431 => Replaced previous num with 'Digit': Pressed digit: {:?}", digit);
            if avail_nums.contains(&digit) {
                current_board.mutable_data[ndx.0] = digit; // Replace 'previous_num' with 'digit' at 'indx' Cell
                commands.insert_resource(NumsMutRes {
                    mutable_data: current_board.mutable_data.clone(),
                }); // Update NumsMutRes Resource
                all_temps.temps_data[index] = vec![]; // Clear temps for Cell at 'index = ndx.0')
                for related_index in &related_indices {
                    // ?? Consider using related_indices.into_iter() and .retain() in place of 'for_loop'
                    if !all_temps.temps_data[*related_index].contains(&previous_num) {
                        if avail_nums.contains(&previous_num)
                            && all_temps.temps_data[*related_index].len() != 0
                        {
                            all_temps.temps_data[*related_index].push(previous_num); // Add back the 'previous_num' to any related_cell temps that do not contain 'removed_num'
                            // all_temps.temps_data[*related_index].sort();  // ?? Perhaps sort() is done just befor redrawing the game_board at cells.rs::spawn_cells ?? // spawn_board_frame

                            commands.insert_resource(AllTempsRes {
                                temps_data: all_temps.temps_data.clone(),
                            }); // Update  "AllTempsRes" Resource
                            next_state.set(GameState::SpawnBoard);
                        }
                    }
                }
            }
            // update_related_temps(commands, ndx, current_board, all_temps, next_state);
            // next_state.set(GameState::SpawnBoard);  // See also 458, 460
            // next_state.set(GameState::NavBoard);
        }
        next_state.set(GameState::SpawnBoard); // Redraw board,  // See also 455, 460
    }
    // next_state.set(GameState::SpawnBoard);  // ? CHANGE REDRAW BOARD.   // See also 458, 455
}

// #######################################################################

//  //////////////////////////////////////////////////////////////////////////////////////////////////////////////////   //
//  /////////////////////////////////   KEY PRESS Commands  //////////////////////////////////////////////////////////   //
//  //////////////////////////////////////////////////////////////////////////////////////////////////////////////////   //
pub fn press_key_commands(
    mut commands: Commands,
    ndx: ResMut<SelectedCellRes>,
    start_board: If<Res<NumsFixedRes>>,
    mut current_board: ResMut<NumsMutRes>,
    mut all_temps: ResMut<AllTempsRes>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) -> () {
    // Capture the number to be removed from current_board before resetting current_num to '0'
    // let removed_num = current_board.mutable_data[ndx.0];
    let related_indices = related_cells(&ndx);
    for key in keyboard_input.get_just_pressed() {
        // if *key == KeyCode::Tab {
        //     next_state.set(GameState::ShowTemps)
        // }
        if *key == KeyCode::KeyN {
            next_state.set(GameState::SpawnBoard) // ?? GameState::NavBoard ??
        } else if *key == KeyCode::KeyP {
            eprintln!("KeyP pressed from press_key_commands() Ln 508 => Switch to EditPerms1");
            next_state.set(GameState::EditPerms1);
            break;
            // observer in spawn_cells() will handle the rest of the transition to EditPerms1, including setting the border color to BORDER_WHITE for the selected cell, and allowing the player to enter a new main number for the selected cell, while also updating related cells temps as needed, and then transitioning back to NavBoard after the player enters a valid main number or presses Escape to clear the cell.  See also 282, 285, 326, 431, 455, 458, 460
        } else if *key == KeyCode::KeyM {
            next_state.set(GameState::Menu);
        } else if *key == KeyCode::Escape {
            if start_board.fixed_data[ndx.0] != 0 {
                next_state.set(GameState::SpawnBoard); // ?? GameState::NavBoard ??
            } else if start_board.fixed_data[ndx.0] == 0 && current_board.mutable_data[ndx.0] != 0 {
                // Perhaps just last term alone
                let removed_num = current_board.mutable_data[ndx.0]; // Capture the number being removed from current_board for selected cell (ndx)
                current_board.mutable_data[ndx.0] = 0; // // Reset the cell_num to 0,    // update current_board by resetting the cell num to 0
                commands.insert_resource(NumsMutRes {
                    mutable_data: current_board.mutable_data.clone(),
                }); // update current_board (NumsMutRes)

                all_temps.temps_data[ndx.0] = vec![]; // clear temps (vec![]), for selected cell (ndx) 

                for related_index in &related_indices {
                    // ?? Consider using related_indices.into_iter() and .retain() in place of 'for_loop'
                    if !all_temps.temps_data[*related_index].contains(&removed_num) {
                        let avail_nums =
                            available_nums_by_board_and_index(&current_board, *related_index);
                        if avail_nums.contains(&removed_num)
                            && all_temps.temps_data[*related_index].len() != 0
                        {
                            all_temps.temps_data[*related_index].push(removed_num); // Add back the 'removed_num' to any related_cell temps that do not contain 'removed_num'
                            all_temps.temps_data[*related_index].sort(); // ?? Consider remove, ? done just befor redrawing the game_board at cells.rs::spawn_cells ?? // spawn_board_frame
                        }
                    }
                }
                commands.insert_resource(AllTempsRes {
                    temps_data: all_temps.temps_data.clone(),
                }); // Update the AllTempsRes resource
            // next_state.set(GameState::SpawnBoard);
            } else {
                all_temps.temps_data[ndx.0] = vec![0; 9]; // Cell is already 0 (no board change):  So CLEAR TEMPS:   THEN update ALLTempsRes Resource to trigger redraw of board
                commands.insert_resource(AllTempsRes {
                    temps_data: all_temps.temps_data.clone(),
                }); // next_state.set(GameState::SpawnBoard);
            }
        }
        next_state.set(GameState::SpawnBoard); // ?? GameState::NavBoard ??
    }
    // next_state.set(GameState::SpawnBoard); // for lines 553, 550, 544, 517, 508          // 439, 441, 443, 445, 447, 449, 451, 453
}

pub fn related_cells(ndx: &ResMut<SelectedCellRes>) -> Vec<usize> {
    // board: &[u8; 81], ndx: usize
    // let sel_cell = ndx.value;
    let row_ndxes = (ndx.0 / 9 * 9..ndx.0 / 9 * 9 + 9)
        .filter(|r| *r != ndx.0)
        .collect::<Vec<usize>>();

    let col_ndxes: Vec<usize> = (0..9)
        .map(|i| i * 9 + (ndx.0 % 9))
        .filter(|c| *c != ndx.0)
        .collect();
    //

    // let cube_base_row = ndx / 27 * 3;  &  let cube_base_col = ndx % 9 / 3 * 3;
    let cube_rows = (ndx.0 / 27 * 3..ndx.0 / 27 * 3 + 3)
        .filter(|r| *r != ndx.0 / 9)
        .collect::<Vec<usize>>();

    let cube_cols = (ndx.0 % 9 / 3 * 3..ndx.0 % 9 / 3 * 3 + 3)
        .filter(|c| *c != ndx.0 % 9)
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

pub fn available_nums_by_board_and_index(
    current_board: &NumsMutRes,
    // board: &[u8; 81],
    ndx: usize,
    //index: usize
) -> Vec<u8> {
    // let index = ndx.0;
    let mut avail_nums = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let related_cells: Vec<usize> = related_cells_by_index(ndx);
    let related_cell_values: Vec<u8> = related_cells
        .into_iter()
        .filter(|ndx| current_board.mutable_data[*ndx] != 0)
        .map(|ndx| current_board.mutable_data[ndx])
        .collect::<Vec<u8>>();

    // let mut avail_nums = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    avail_nums.retain(|val| !related_cell_values.contains(val));
    // eprintln!("From nav_board.rs,  available nums by board and index:  index: {:?}, available_nums = {:?}", ndx, avail_nums);
    return avail_nums;
}

//
//

//
//

//
//

//
//

//
//

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////   PROGRAM NOTES   /////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */

//
//

//
//

//
//

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////   Start Player Interactions   /////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */

//  Handling Player Interactions with Sudoku Game Board
//
//  This module contains the systems and resources related to handling player interactions with the Sudoku board,
//      including selecting and hovering over cells, entering digits, toggling temporary notes, and managing the edit modes for cells.
//  It also defines the intent and outcome message types for player actions,
//      and the logic for applying those intents to update the game state and UI accordingly.
//
//
//  //  "including collecting input intents from pointer and keyboard events, applying those intents to update the game state, and projecting the appropriate UI feedback (such as cell borders) based on the current state of the board and the player's interactions"
//
//
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ///////////////  PLAYER INTERACTIONS with the SUDOKU BOARD  ////////////////////////////////////////////////////////
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ///////////////////////////  HOVER SYSTEMS  ////////////////////////////////////////////////////////////////////////
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

//  1    HoverSystemsPlugin   //  This plugin is responsible for managing the systems related to player interactions with the Sudoku board,
//  2    cell_interaction_system()  // Collects player input from both pointer interactions and keyboard events, and translates them into high-level gameplay intents that can be consumed by other systems to update the game state and UI accordingly.
//  3    collect_cell_interaction_intents()  = ( collect_keyboard_intents() => collect_cell_interaction_intents() )
//  4      ==>
//  5    apply_sudoku_intents()
//  6    project_cell_borders()

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// /////////////////////////   SYSTEM STEPS   /////////////////////////////////////////////////////////////////////////
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
//      1   HoverSystemsPlugin   //  This plugin is responsible for managing the systems related to player interactions with the Sudoku board,
//      2   collect_cell_interaction_intents()  = ( collect_keyboard_intents() => collect_cell_interaction_intents() )
//      3   collect_keyboard_intents()
//      4      =>
//      5   apply_sudoku_intents()
//              This system reads the stream of SudokuIntents generated by the input collection systems, and
//                  applies the corresponding changes to the game state, such as updating the board data, changing the edit mode, or
//                  transitioning to different game states based on the player's actions.
//              It also generates SudokuOutcome messages to communicate the results of those actions
//                  to other systems that may need to respond to them, such as by updating the UI or triggering animations.
//
//      => project_cell_borders()
//
//
//      6   project_cell_borders()

//// player interactions with the Sudoku board
//          HoverSystemsPlugin      //  This plugin is responsible for managing the systems related to player interactions with the Sudoku board,
//          "cell_interaction_system()  // Collects player input from both pointer interactions and keyboard events, and
//              translates them into high-level gameplay intents that can be consumed by other systems to update the game state and UI accordingly "
//
//
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ///////////////////////   (INVOLVED) SYSTEM FUNCTIONS   ////////////////////////////////////////////////////////////
/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

//      HoverSystemsPlugin      //  This plugin is responsible for managing the systems related to player interactions with the Sudoku board,
//      cell_interaction_system() + resolve_cell_target() *    // Collects player input from both pointer interactions & keyboard events, and
//              translates them into high-level gameplay intents that can be consumed by other systems to update the game state and UI accordingly "
// 
//      collect_cell_interaction_intents() + 
apply_sudoku_intents() => project_cell_borders()
*/

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// ///////////////////////////////   End Player Interactions   ///////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */

/*
extern crate bevy_picking
This crate provides 'picking' capabilities for the Bevy game engine, allowing pointers to interact with entities using hover, click, and drag events.

Overview
In the simplest case, this plugin allows you to click on things in the scene. However, it also allows you to express more complex interactions,
like detecting when a touch input drags a UI element and drops it on a 3d mesh rendered to a different camera.

Pointer events bubble up the entity hierarchy and can be used with observers, allowing you to succinctly express rich interaction behaviors
by attaching pointer callbacks to entities:

world.spawn(MyComponent)
    .observe(|mut event: On<Pointer<Click>>| {
        // Read the underlying pointer event data
        println!("Pointer {:?} was just clicked!", event.pointer_id);
        // Stop the event from bubbling up the entity hierarchy
        event.propagate(false);
    });

    // You can also use the pointer events in a system, which allows you to query for other components on the entity that was interacted with:

    fn my_system(mut events: EventReader<Pointer<Click>>, query: Query<&MyComponent>) {  // (MessageReader<Pointer<Click>>, Query<&MyComponent>)
        for event in events.iter() {
            if let Ok(component) = query.get(event.target) {
                println!("Clicked on an entity with MyComponent: {:?}", component);
            }
        }
    }


    world.spawn_empty()
    .observe(|event: On<Pointer<Over>>| {
        println!("I am being hovered over");
    });
Observers give us three important properties:

They allow for attaching event handlers to specific entities,
they allow events to bubble up the entity hierarchy,
and they allow events of different types to be called in a specific order.

The order in which interaction events are received is extremely important, and you can read more about it on the docs for the dispatcher system:
pointer_events. This system runs in PreUpdate in PickingSystems::Hover.
All pointer-event observers resolve during the sync point between pointer_events and update_interactions.

Events Types
The events this module defines fall into a few broad categories:

Hovering and movement: Over, Move, and Out.
Clicking and pressing: Press, Release, and Click.
Dragging and dropping: DragStart, Drag, DragEnd, DragEnter, DragOver, DragDrop, DragLeave.
When received by an observer, these events will always be wrapped by the Pointer type, which contains general metadata about the pointer event.

*/

/*
extern crate bevy_picking
This crate provides 'picking' capabilities for the Bevy game engine, allowing pointers to interact with entities using hover, click, and drag events.

Overview
In the simplest case, this plugin allows you to click on things in the scene. However, it also allows you to express more complex interactions,
like detecting when a touch input drags a UI element and drops it on a 3d mesh rendered to a different camera.

Pointer events bubble up the entity hierarchy and can be used with observers,
allowing you to succinctly express rich interaction behaviors by attaching pointer callbacks to entities:

world.spawn(MyComponent)
    .observe(|mut event: On<Pointer<Click>>| {
        // Read the underlying pointer event data
        println!("Pointer {:?} was just clicked!", event.pointer_id);
        // Stop the event from bubbling up the entity hierarchy
        event.propagate(false);
    });
At its core, this crate provides a robust abstraction for computing picking state regardless of pointing devices, or what you are hit testing against.
It is designed to work with any input, including mouse, touch, pens, or virtual pointers controlled by gamepads.


Expressive Events
Although the events in this module (see events) can be listened to with normal MessageReaders, using observers is often more expressive, with less boilerplate.
This is because observers allow you to attach event handling logic to specific entities, as well as make use of event bubbling.

When events are generated, they bubble up the entity hierarchy starting from their target,
until they reach the root or bubbling is halted with a call to On::propagate. See Observer for details.


This allows you to run callbacks when any children of an entity are interacted with, and leads to succinct, expressive code:

fn setup(mut commands: Commands) {
    commands.spawn(Transform::default())
        // Spawn your entity here, e.g. a `Mesh3d`.
        // When dragged, mutate the `Transform` component on the dragged target entity:
        .observe(|drag: On<Pointer<Drag>>, mut transforms: Query<&mut Transform>| {
            let mut transform = transforms.get_mut(drag.entity).unwrap();
            transform.rotate_local_y(drag.delta.x / 50.0);
        })
        .observe(|click: On<Pointer<Click>>, mut commands: Commands| {
            println!("Entity {} goes BOOM!", click.entity);
            commands.entity(click.entity).despawn();
        })
        .observe(|over: On<Pointer<Over>>, mut greetings: MessageWriter<Greeting>| {
            greetings.write(Greeting);
        });
}
Modularity
Mix and Match Hit Testing Backends
The plugin attempts to handle all the hard parts for you, all you need to do is tell it when a pointer is hitting any entities.
Multiple backends can be used at the same time! Use this simple API to write your own backend in about 100 lines of code.

Input Agnostic
Picking provides a generic Pointer abstraction, which is useful for reacting to many different types of input devices.
Pointers can be controlled with anything, whether it's the included mouse or touch inputs,
or a custom gamepad input system you write yourself to control a virtual pointer.


Robustness
In addition to these features, this plugin also correctly handles multitouch, multiple windows, multiple cameras, viewports,
and render layers. Using this as a library allows you to write a picking backend that can interoperate with any other picking backend.

Getting Started
TODO: This section will need to be re-written once more backends are introduced.

Next Steps
To learn more, take a look at the examples in the examples. You can read the next section to understand how the plugin works.

The Picking Pipeline
This plugin is designed to be extremely modular. To do so, it works in well-defined stages that form a pipeline,
where events are used to pass data between each stage.

Pointers (pointer)
The first stage of the pipeline is to gather inputs and update pointers. This stage is ultimately responsible for generating PointerInput events.
The provided crate does this automatically for mouse, touch, and pen inputs. If you wanted to implement your own pointer,
controlled by some other input, you can do that here. The ordering of events within the PointerInput stream is meaningful
for events with the same PointerId, but not between different pointers.

Because pointer positions and presses are driven by these events, you can use them to mock inputs for testing.

After inputs are generated, they are then collected to update the current PointerLocation for each pointer.

Backend (backend)
A picking backend only has one job: reading PointerLocation components, and producing PointerHits.
You can find all documentation and types needed to implement a backend at backend.

You will eventually need to choose which picking backend(s) you want to use. This crate does not supply any backends,
and expects you to select some from the other bevy crates or the third-party ecosystem.

It's important to understand that you can mix and match backends! For example, you might have a backend for your UI,
and one for the 3d scene, with each being specialized for their purpose. Bevy provides some backends out of the box,
but you can even write your own. It's been made as easy as possible intentionally; the bevy_mod_raycast backend is 50 lines of code.

Hover (hover)
The next step is to use the data from the backends, combine and sort the results, and determine what each cursor is hovering over,
producing a HoverMap. Note that just because a pointer is over an entity, it is not necessarily hovering that entity.
Although multiple backends may be reporting that a pointer is hitting an entity, the hover system needs to determine which entities
are actually being hovered by this pointer based on the pick depth, order of the backend, and the optional Pickable component of the entity.
In other words, if one entity is in front of another, usually only the topmost one will be hovered.

Events (events)
In the final step, the high-level pointer events are generated, such as events that trigger when a pointer hovers or clicks an entity.
These simple events are then used to generate more complex events for dragging and dropping.

Because it is completely agnostic to the earlier stages of the pipeline, you can easily extend the plugin with arbitrary backends
and input methods, yet still use all the high level features.
//

// Focus System  vs  NavBoard System:  Consider whether to combine into one system, or keep separate for clarity and ease of maintenance.
If combined, consider renaming to something like "cell_interaction_system" or "cell_selection_system" to reflect the broader scope of
handling both focus and navigation interactions with the cells on the board.

// Focus System  vs  Picking System:  Event reader
*/

/*
// "sudo1::nav_board::nav_board_system`: Parameter if `Res<'_, NumsFixedRes>` failed validation, then Resource does not exist"
//  Try mut current_board: If<ResMut<NumsMutRes>>,  See systen parameters, 4 & 5 lines below, (line 72 & 73, currently)
/*
pub fn nav_board_system(
    mut commands: Commands,
    start_board: If<Res<NumsFixedRes>>,
    mut current_board: ResMut<NumsMutRes>,
    mut next_state: ResMut<NextState<GameState> >,
    mut interaction_query: Query<(&Interaction, &mut BorderColor, &Ndx), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut border_color, ndx) in &mut interaction_query {
        let index = ndx.0;
        let is_fixed = start_board.fixed_data[index] != 0;
        let is_mutable = current_board.mutable_data[index] != 0;
        let start_bd_num = start_board[index];
        let current_board = &mut current_board.mutable_data;
        let current_bd_num = current_board[index];

        match interaction {
            Interaction::Pressed => {
                commands.insert_resource(SelectedCellRes(ndx.0));
                if is_fixed {
                    BorderColor::all(BORDER_FIXED_NUMS); // black
                    next_state.set(GameState::EditPerms0);
                } else if is_mutable {
                    BorderColor::all(BORDER_MUT_NUMS); // red
                    next_state.set(GameState::EditPerms1);
                } else {
                    BorderColor::all(BORDER_EDIT_TEMPS); // cyan
                    next_state.set(GameState::EditTemps);
                }
            }
            Interaction::Hovered => {
                commands.insert_resource(SelectedCellRes(ndx.0));
                if start_bd_num != 0 {
                    BorderColor::all(BORDER_MUT_NUMS);
                } else if current_bd_num != 0 {
                    BorderColor::all(BORDER_WHITE);
                } else {
                    BorderColor::all(BORDER_HOVER_TMPS);
                    next_state.set(GameState::NavBoard); // SpawnBoard, UpdateCell, NavBoard
                }
            }
            Interaction::None => {
                if current_board[ndx.0] == 0 {
                    // if **num == 0 {
                    BorderColor::all(BORDER_NORM_TMPS);

                } else {
                    BorderColor::all(BORDER_NORM_NUMS);

                }
            }
        }
    }
}
*/

//

*/

//
// }
/*
    for click in clicks.iter() {
        if let Ok(ndx) = ndx_query.get(click.entity) {
            selected.0 = ndx.0;
            if start_board.fixed_data[ndx.0] != 0 {
                next_state.set(GameState::EditPerms0);
            } else if current_board.mutable_data[ndx.0] != 0 {
                next_state.set(GameState::EditPerms1);
            } else {
                next_state.set(GameState::EditTemps);
            }
        }
    }
    for over in overs.iter() {
        if let Ok(ndx) = ndx_query.get(over.entity) {
            selected.0 = ndx.0;
            // Update border color to indicate hover state
            if start_board.fixed_data[ndx.0] != 0 {
                BorderColor::all(BORDER_MUT_NUMS); // Red
            } else if current_board.mutable_data[ndx.0] != 0 {
                BorderColor::all(BORDER_WHITE); // White
            } else {
                BorderColor::all(BORDER_HOVER_TMPS); // Cyan
            }
        }
    }
    for out in outs.iter() {
        if let Ok(ndx) = ndx_query.get(out.entity) {
            // Reset border color to normal state
            if current_board.mutable_data[ndx.0] == 0 {
                BorderColor::all(BORDER_NORM_TMPS); // Light Blue-Grey
            } else {
                BorderColor::all(BORDER_NORM_NUMS); // Medium Red
            }
        }
    }
*/

//

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */

//

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////   Order of Systems Execution   ///////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */

//    Ref:  https://bevy-cheatbook.github.io/programming/system-order.html

/*
            Controlling Execution Order
    To ensure that systems execute in a specific order, Bevy provides methods such as .before() and .after().
    This allows developers to set dependencies between systems, ensuring that one system runs before or after another as needed.

        Method Description
    .before()	Ensures the specified system runs before another system.
    .after()	Ensures the specified system runs after another system.
    Example of System Ordering

    Here’s a simple example of how to use these methods:
    (rust)

    fn main() {
    App::new()
        .add_systems(Update, (
            player_movement.after(input_handling),
            enemy_movement.before(player_movement),
        ))
        .run();
    };
*/
//

//
/*
    The order of system execution in Bevy can significantly impact performance,
        as systems that do not conflict over data can run in parallel, maximizing CPU usage
    However, if systems require mutable access to the same data, they cannot run simultaneously,
        which can lead to performance bottlenecks if not managed properly
*/
//

/*  System Parameters Ordering:  Consider the order of system parameters for readability and consistency across the codebase
        See also:  https://bevy-cheatbook.github.io/features/systems/parameters.html#system-parameters-ordering
*/

//
/*
Use a Set(s) for all your 'input handling systems' and a Set for all your 'gameplay systems'

In order to run your 'input handling systems' before your 'gameplay systems'
    and to ensure that all player input is collected and processed before any 'game logic systems', that depend on that input logic, before theyt are run
This helps maintain a clear separation of concerns and ensures that the 'game state' is updated in a predictable and consistent manner based on player interactions
    See also:  https://bevy-cheatbook.github.io/features/systems/

A Set for 'input handling systems' => order them to run before gameplay systems (a Set for all your 'gameplay systems') */

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// //////////////////////   Ordering System Parameters   //////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */
//
/*
        Grouping Parameters
    Parameters can be organized into tuples, which helps manage complexity and keeps the code clean.
    This is particularly useful when a system needs multiple types of data.
*/

//
/*
    Consider ordering system parameters in a consistent way across all systems in the codebase, to improve readability and maintainability

    For example, you might choose to always order parameters as follows:

        1. Commands
        2. Resources (Res, ResMut, If<Res>, If<ResMut>)
        3. Queries (Query, QuerySet)
        4. EventReaders and EventWriters (MessageReader, MessageWriter)

    This way, when reading a system function signature, you can quickly identify the types of parameters being used and their purpose within the system
*/

/*
    Note: The order of system parameters is important for readability and consistency across the codebase

    An 'Order of system functions arguements':

    fn bevy_system (
        arg_1,
        arg_2,
        arg_3,
        arg_4,
        arg_5,
        arg_6
    ) -> () {
            // system logic here
    };

    Parameter Order:
        1 Commands,
        2 Res<>,
        3 ResMut<>,
        4 Query<>,
        5 EventReader<>, (MessageReader)
        6 EventWriter<>  (MessageWriter)


    Order of events is important, and you can read more about it on the docs for the dispatcher system:  pointer_events

    By following a consistent order, developers can quickly understand the purpose of each parameter and how it interacts with the system's logic:

    fn bevy_system (
        1 Commands,
        2 Res,
        3 ResMut,
        4 Query,
        5 EventReader, (MessageReader)
        6 EventWriter  (MessageWriter)
    ) -> {
        system logic here
    };


    - This System runs in PreUpdate
    - All pointer-event observers resolve during the sync point between pointer_events and update_interactions
*/

//

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// /////////////////////////   Dispatcher System   ////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/*
    I assume you want how to run systems with Bevy 0.18's Dispatcher (scheduling) API. Below is a concise guide with examples for the common patterns: creating a Dispatcher, adding systems, ordering, labels, and running it manually.

            Creating and running a Dispatcher

    Use bevy_ecs::prelude::{World, Schedule, SystemStage, IntoSystem, StageLabel, ScheduleRunnerSettings} for a schedule-based approach, or bevy_ecs::schedule::Dispatcher for the Dispatcher API.
    Example (manual Dispatcher run using Schedule + World — recommended simple approach):
    (rust)

    use bevy_ecs::prelude::*;

    fn hello() {
        println!("hello");
    }

    fn main() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_stage("update", SystemStage::single_threaded().with_system(hello));
        // Run the schedule once:
        schedule.run(&mut world);
    }

            Using Dispatcher directly (more manual control)
    Example (manual Dispatcher run):
    (rust)

    use bevy_ecs::prelude::*;
    use bevy_ecs::schedule::{Dispatcher, SystemStage};

    fn a() { println!("a"); }
    fn b() { println!("b"); }

    fn main() {
        let mut world = World::new();
        // Build stages then create a dispatcher
        let mut dispatcher = Dispatcher::builder()
            .add_stage("pre", SystemStage::parallel().with_system(a))
            .add_stage("main", SystemStage::parallel().with_system(b))
            .build();
        // Initialize (runs startup systems if any)
        dispatcher.setup(&mut world);
        // Run dispatcher: executes all stages in order
        dispatcher.dispatch(&mut world);
        // Apply command buffers
        world.run_default_schedule(); // or call dispatcher.flush/other as needed
    }

//

//

            System addition and ordering

    Label systems with functions/structs or use system labels:
        .with_system(my_system.label("A"))
        .with_system(other_system.after("A"))

    Use Stage ordering: stages run in the order you add them to the Schedule/Dispatcher.
    For fine-grained ordering inside a stage, use system labels and .before/.after.

    //

            Commands, setup, and resources

    When adding systems that use Commands or Resources, call dispatcher.setup(&mut world) before dispatch/dispatching so system resources (like resource IDs) are registered.

    Use world.insert_resource(...) to add resources the systems expect.

    //

            Parallel vs single-threaded
    SystemStage::parallel() runs systems in parallel where possible

    SystemStage::single_threaded() runs them sequentially

    Choose based on mutability/conflicts


            Common pitfalls
    Forgetting dispatcher.setup(&mut world) leads to missing system init (especially for Commands)

    Mut borrow conflicts: ensure systems declare correct Query mutability to allow parallel execution

    For repeated runs, call dispatcher.dispatch(&mut world) each frame and then dispatcher.flush(&mut world)
        or ensure command buffers are applied (setup handles some init).
*/

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// /////////////////////////   Bevy Observer System   /////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */
/*
#[derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, FromReflect)]  // Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, FromReflect, Resource, Deref, DerefMut
pub struct Trigger<'w, E, : Bundle = ()> {
    event: &'w mut E,
    propagate: &'w mut bool,
    pub entity: Entity,
    pub components: Option<&'w T>,
}
*/
