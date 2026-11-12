///  Responsible for displaying the board selection menu & initializing multiple board data Resources
///  Following the introduction Splash Screens   Board Chooser Plugin:  Responsible for displaying the board selection menu
///  The Menu allows for selecting the desired difficulty level:  Determines the "start_board" used to initialize multiple board data Resources
///  THE Menu Transitions to the SolveBoard state to solve the board and then the SpawnBoard state to display the solved board & main game board screen
///  Sets the start_board:  start_bd
//

//  #####################################################################################
use bevy::prelude::*;
use rand::prelude::*;

// use crate::setup_board::AllTempsRes;
use crate::setup_board::AllTempsRes;
use crate::states::*;
use crate::sudoku_constants::{
    EASY, EXTREME, HARD, HOVERED_BUTTON, HOVERED_PRESSED_BUTTON, MEDIUM, NORMAL_BUTTON,
    PRESSED_BUTTON, SPLASH_TEXT_1, TEXT_LIGHT_BLUE, TEXT_SHADOW_WHITE,
}; // TEXT_BRIGHT_BLUE_2, SPLASH_TEXT_1 };

// use crate::nav_board::available_nums_by_board_and_index;

// use crate::solver::*;
// use crate::pause::*;

pub struct BoardChoicePlugin;
// start_board, menu_action, button_hover_system,
impl Plugin for BoardChoicePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectedCellRes(0))
            // .insert_resource(AllTempsRes::new([0; 9].to_vec()))  // This is the correct way to insert the AllTempsRes Resource with an array of 81 empty vectors of Type => [Vec<u8>; 81]
            // .insert_resource(AllTempsRes{ temps.data: [&[0; 9].to_vec(); 81] })    // default(),  should be an Array of 81 empty vectors of Type => [Vec<u8>; 81]
            //
            .add_systems(OnEnter(GameState::ChooseBoard), choose_board_setup)
            .add_systems(
                OnEnter(GameState::Easy), // Use of closure to wrap start_board in a closure to pass 2 additional Rust parameters to Bevy
                |commands: Commands, next_state: ResMut<NextState<GameState>>| {
                    start_board(commands, EASY, next_state);
                },
            )
            .add_systems(
                OnEnter(GameState::Medium),
                |commands: Commands, next_state: ResMut<NextState<GameState>>| {
                    start_board(commands, MEDIUM, next_state);
                },
            )
            .add_systems(
                OnEnter(GameState::Hard),
                |commands: Commands, next_state: ResMut<NextState<GameState>>| {
                    start_board(commands, HARD, next_state);
                },
            )
            .add_systems(
                OnEnter(GameState::Extreme),
                |commands: Commands, next_state: ResMut<NextState<GameState>>| {
                    start_board(commands, EXTREME, next_state);
                },
            )
            // .add_systems(Update, menu_action)
            .add_systems(
                Update,
                (menu_action, button_hover_system).run_if(in_state(GameState::ChooseBoard)),
            )
            // .add_systems(
            //     OnExit(GameState::ChooseBoard),
            //     calc_all_temps_res
            // );
            .add_systems(
                OnExit(GameState::ChooseBoard),
                despawn_screen::<OnChooseBoardScreen>,
            );
    }
}

// Core board resources (stateful model/control):  For game board and user interactions with the board
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct HoveredCellRes(pub Option<usize>);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditMode {
    #[default]
    Navigate, // no active edit
    TempNotes,  // entering temps
    MainNumber, // entering mutable main number
    LockedCell, // selected fixed clue
}

// BoardRevisions: increment after any mutation, cheap trigger for “refresh UI/messages.”
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct BoardRevision(pub u64);

// Resource(Tuple Struct) for tracking the selected cell
#[derive(Component, Resource, Debug, Deref, DerefMut, Copy, Clone, Hash)]
pub struct SelectedCellRes(pub usize);

impl SelectedCellRes {
    // Example of creating a new SelectedCellRes instance
    pub fn new(ndx: usize) -> SelectedCellRes {
        SelectedCellRes(ndx)
    }
}

impl Default for SelectedCellRes {
    fn default() -> SelectedCellRes {
        SelectedCellRes(0)
    }
}

//  This is the IMMUTABLE ("Fixed"), = "start_board"
#[derive(Resource, Deref, DerefMut, Debug, Copy, Clone, PartialEq, Eq)]
pub struct NumsFixedRes {
    pub fixed_data: [u8; 81],
}

impl NumsFixedRes {
    pub fn new(fixed_board: [u8; 81]) -> NumsFixedRes {
        NumsFixedRes {
            fixed_data: fixed_board,
        }
    }
}

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////   Mut Nums Data Resource   //////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////   "current_board"   //////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */

#[derive(Resource, Deref, DerefMut, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NumsMutRes {
    pub mutable_data: [u8; 81],
}

impl NumsMutRes {
    pub fn new(mutable_board: [u8; 81]) -> NumsMutRes {
        NumsMutRes {
            mutable_data: mutable_board,
        }
    }
}

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////   Solved Data Resource   ////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */

#[derive(Resource, Debug, Deref, DerefMut, Clone, PartialEq, Eq)]
pub struct NumsSolvedRes {
    pub data: [u8; 81],
}

// Implement the immutable SolvedBoardRes Resource generated by solving the start_board data
// Consider saving to a text file for later recovery, to avoid having to solve the board again when returning to the Main Menu and then back to the Game Menu
//  possibly eliminate the SavedBoardRes, Resource below if the SolvedBoardRes can be used for this purpose as well,
// since it is immutable and contains the solved board data that can be saved to a text file for later recovery and display
impl NumsSolvedRes {
    pub fn new(solved_board: [u8; 81]) -> NumsSolvedRes {
        NumsSolvedRes { data: solved_board }
    }
}

/*
impl NumsSolvedRes {
    pub fn new() -> Self {
        NumsSolvedRes { solved_data: [0; 81] }
    }
}
*/

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////   Saved Data Resource   /////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */

//  This is the immutable ("saved_board"), for a saved game,
// saved to a text file for later recovery and conversion to "current board"(NumsMutRes)
// " CHECK impl SavedBoardRes below, specifically '&' in &[u8; 81] ! ! ! "
#[derive(Resource, Debug, Deref, Clone, PartialEq, Eq)]
pub struct SavedBoardRes {
    pub data: [u8; 81], // change to saved_data: [u8; 81]  later, Also consider Vec<u8>
}

impl SavedBoardRes {
    pub fn new(saved_board: [u8; 81]) -> SavedBoardRes {
        SavedBoardRes { data: saved_board }
    }
}

// ?? change to all_temps, all_temps_data  below ??
//  ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ////////////////////////   Cell Temps Data Resource   ///////////////////////////////////////////////////////////////
// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

//    pub fn generate_board(bds: [&str; 4], mut commands: Commands, mut next_state: ResMut<NextState<GameState>>)
pub fn start_board(
    mut commands: Commands,
    // bds: [&str; 4],
    bds: [&str; 4], // change to 10 for easy, and 4 for medium, hard, extreme
    // all_temps: ResMut<AllDataRes>,
    mut next_state: ResMut<NextState<GameState>>,
) -> () {
    let mut start_bd: [u8; 81] = [0; 81];
    let mut rng = rand::rng();
    let bd_index: usize = rng.random_range(0_usize..bds.len()); // Obtain "start_board" using a random index to apply to the chosen board string category array
    let str_bd = bds[bd_index]; // Apply random index to return a random start_board as &str
    eprintln!(
        "Random Board Index: {:?}, \nBoard String: {:?}",
        bd_index, str_bd
    ); // Print the selected board index and string for debugging    

    str_bd.chars().enumerate().for_each(|(i, c)| {
        // Convert the random board of Type=&str to Type: [u8; 81] array
        let num = c.to_digit(10).unwrap() as u8;
        if num != 0 {
            start_bd[i] = num;
        } //else {
        //start_bd[i] = 0;
        //}
    });
    commands.insert_resource(NumsFixedRes {
        fixed_data: start_bd.clone(),
    });
    commands.insert_resource(NumsMutRes {
        mutable_data: start_bd.clone(),
    });
    commands.insert_resource(NumsSolvedRes {
        data: start_bd.clone(),
    }); // CONSIDER:  solved_data
    commands.insert_resource(SavedBoardRes {
        data: start_bd.clone(),
    }); // CONSIDER:  saved_data
    //
    //  commands.insert_resource(AllTempsRes { temps_data: std::array::from_fn(|_| vec![]) });
    //
    commands.insert_resource(AllTempsRes {
        temps_data: std::array::from_fn(|_| Vec::with_capacity(9)),
    }); // This is the correct way to initialize the AllTempsRes Resource with an array of 81 empty (or all zero) vectors of Type => [Vec<u8>; 81]

    next_state.set(GameState::SolveBoard);

    /*
    pub fn calc_initial_temps_once(
        mut commands: Commands,
        mut all_temps: ResMut<AllTempsRes>,
        mut current_board: &NumsMutRes,
    ) -> () {
        // after commands.insert_resource(AllTempsRes { ... })
        // for i in (0..81).iter().filter(|&&i| current_board.mutable_data[i] == 0) {
        let mut avail_nums = [[0_u8; 9]; 81];
        (0..81).iter()
            .filter(|&&i| current_board.mutable_data[i] == 0)
            // .map(|&i| all_temps: [vec![u8; 9]; 81] = available_nums_by_board_and_index(current_board, i)),
            // if current_board.mutable_data[i] == 0 {
                 // your existing fn
                // all_temps.temps_data.clone()[i] = avail_nums;
                all_temps.temps_data[i] = avail_nums[i].clone(); // Update the temps_data for the current index
                commands.insert_resource(AllTempsRes { temps_data: all_temps.temps_data.clone() }); // Insert the updated AllTempsRes back into resources
    }
    */
}

// Tag component used to tag entities added on the choose_board screen
#[derive(Component)]
pub struct OnChooseBoardScreen;

#[derive(Resource, Component, Copy, Clone)]
pub struct SelectedOption; // Tag component to indicate a selected button Resource (Below) for substate option selection

fn button_hover_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

pub fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    // app_exit_writer: MessageWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    // quitting the App using Bevy's Struct MessageWriter and Messenging System here vs Return to Main Menu
                    // app_exit_writer.send(AppExit);
                    //  OR
                    next_state.set(GameState::Menu); // Return to Main Menu
                }
                MenuButtonAction::Easy => {
                    // Resume the Existing Game
                    next_state.set(GameState::Easy);
                }
                MenuButtonAction::Medium => {
                    next_state.set(GameState::Medium);
                    // MenuButtonAction::SettingsDisplay => {
                    //     menu_state.set(MenuState::SettingsDisplay);
                }
                MenuButtonAction::Hard => {
                    next_state.set(GameState::Hard);
                    // next_state.set(GameState::ChooseBoard);
                }
                // MenuButtonAction::ShowSolution => menu_state.set(MenuState::Main),
                MenuButtonAction::Extreme => {
                    next_state.set(GameState::Extreme);
                    //     menu_state.set(MenuState::Settings);
                } // _ => { }  // Consider saving the Current Game State to a text file
            }
        }
    }
}

// All actions that can be triggered from a button click
#[derive(Component)]
pub enum MenuButtonAction {
    Easy,
    Medium,
    Hard,
    Extreme,
    Quit, // return to PauseMenu
}
//

// //////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn choose_board_setup(mut commands: Commands, asset_server: Res<AssetServer>) -> () {
    // Common style for all buttons on the screen
    let font_handle_fcode_b: Handle<Font> = asset_server.load("fonts/FiraCode-Bold.ttf");
    let button_node = Node {
        width: px(375),
        height: px(75),
        border: UiRect::all(px(5.0)),
        border_radius: BorderRadius::all(px(20.0)),
        margin: UiRect::all(px(20)),
        padding: UiRect::all(Val::Px(20.0)), // ? try thid later - The amount of space between the edges of a node and its contents
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    // BackgroundColor(Color::srgba(0.99, 0.99, 0.99, 0.99));
    BorderColor::all(Color::srgba(0.60, 0.60, 0.90, 0.99));
    // let button_icon_node = Node {  // Button Icon is not used, so delete this 'assets: Res<AssetServer>'
    let button_text_font = TextFont {
        font: font_handle_fcode_b,
        font_size: 33.0,
        ..default()
    };
    let box_shadow = BoxShadow(vec![ShadowStyle {
        color: HOVERED_BUTTON, // NORMAL_BUTTON, Color::WHITE.with_alpha(0.5), Color = Color::srgb(0.0, 0.0, 0.7);
        x_offset: px(0.0),     //  px(shadow.x_offset)
        y_offset: px(0.0),     // px(shadow.y_offset)
        spread_radius: px(5.0), // shadow.spread
        blur_radius: px(5.0),  // shadow.blur
    }]);
    commands.spawn((
        DespawnOnExit(GameState::ChooseBoard),
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        OnChooseBoardScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                // justify_content: JustifyContent::Center,
                justify_self: JustifySelf::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            // BackgroundColor(BLACK.into()),
            children![
                // Display the Choose Board Menu
                (
                    // Text::new("|   CHOOSE   |\n| DIFFICULTY |"), // "Bevy Game Menu UI"
                    Text::new("< CHOOSE LEVEL >"),
                    TextFont {
                        font_size: 60.0, // 48.0
                        ..default()
                    },
                    TextColor(SPLASH_TEXT_1.into()), // MENU_TITLE, LIGHT_BLUE, YELLOW
                    Node {
                        // margin: UiRect::all(px(80.)),  // 20, 50
                        margin: UiRect::bottom(px(40.)), // 180.), 320, 220
                        // justify_content: JustifyContent::Center,
                        justify_self: JustifySelf::Center,
                        ..default()
                    },
                    TextShadow {
                        color: TEXT_SHADOW_WHITE.into(), // SPLASH_TEXT_1.into(), TEXT_SHADOW
                        offset: Vec2::new(2.0, 0.5),
                    },
                ),
                (
                    Button,
                    button_node.clone(),
                    box_shadow.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Easy,
                    children![
                        // (ImageNode::new(right_icon), button_icon_node.clone()),
                        (
                            Text::new("Easy"),
                            button_text_font.clone(),
                            TextColor(TEXT_LIGHT_BLUE.into()),
                            // TextColor(TEXT_COLOR),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    box_shadow.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Medium,
                    children![
                        // (ImageNode::new(wrench_icon), button_icon_node.clone()),
                        (
                            Text::new("Medium"), // Go to Board Chooser
                            button_text_font.clone(),
                            TextColor(TEXT_LIGHT_BLUE.into()),
                            // TextColor(TEXT_COLOR),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    box_shadow.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Hard,
                    children![
                        // (ImageNode::new(wrench_icon), button_icon_node.clone()),
                        (
                            Text::new("Hard"),
                            button_text_font.clone(),
                            TextColor(TEXT_LIGHT_BLUE.into()),
                            // TextColor(TEXT_COLOR),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    box_shadow.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Extreme,
                    children![
                        // (ImageNode::new(wrench_icon), button_icon_node.clone()),
                        (
                            Text::new("Extreme"),
                            button_text_font.clone(),
                            TextColor(TEXT_LIGHT_BLUE.into()),
                            // TextColor(TEXT_COLOR),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    box_shadow.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Quit,
                    children![
                        // (ImageNode::new(exit_icon), button_icon_node.clone()),
                        (
                            Text::new("MAIN MENU"),
                            button_text_font,
                            TextColor(TEXT_LIGHT_BLUE.into()),
                            // TextColor(TEXT_COLOR.into()),
                        ),
                    ]
                ),
            ]
        )],
    ));
}

pub fn print_board_simple(bd: &[u8; 81]) {
    println!("    -----------------------");
    for i in 0..9 {
        eprint!("   | ");
        for j in 0..9 {
            let val = bd[i * 9 + j];
            if val == 0 {
                eprint!(". "); // "+ "
            } else {
                eprint!("{val} ");
            }
            if j == 2 || j == 5 {
                eprint!("| ");
            }
            if j == 8 {
                eprint!("|");
            }
        }
        eprintln!();
        if i == 2 || i == 5 {
            eprintln!("   |-------+-------+-------|");
        }
    }
    println!("    -----------------------\n\n");
}

pub fn show_solve(solved_board: ResMut<NumsSolvedRes>) -> () {
    let board = &solved_board.data;
    eprintln!("Print Solved Board from board_chooser.rs:\n");
    for i in 0..9 {
        let start = i * 9;
        let end = start + 9;
        eprintln!("   {:?}", &board[start..end]);
    }
    println!("\n");
    // eprintln!("Solved_Board: {:?}", &solved_nums.data.clone());
}

/// Generic system that takes a component as a parameter, and will recursively despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}

/*
pub enum BoardDifficulty {
    Easy,
    Medium,
    Hard,
    Extreme,
}
*/

/*
pub fn calc_all_temps_res(
    mut commands: Commands,
    current_board: NumsMutRes,
    all_temps: AllTempsRes,
) -> () {
    let mut all_temps: [Vec<u8>; 81] = std::array::from_fn(|_| Vec::with_capacity(9));
    for ndx in 0..81_usize {
        if current_board.mutable_data[ndx] == 0 {
            let temps = calculate_cell_temps(ndx, current_board);
            all_temps[ndx] = temps;
        } else {
            all_temps[ndx] = [0; 9].to_vec().with_capacity(9);
        }
    }
    // all_temps
}
*/
