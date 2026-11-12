/// This module displays Text Messages to the Player, Each Triggered on Entering a Specific Game State
/// /// Text Messages are displayed using Bevy's UI Text System in Either the Upper Text Box or the Lower Text Boxes:  
/// Text Messages in the Upper Text Box provide Player Instructions While Text Messages in the Lower Text Box provide Game Information
/// Text Messages are Color-Coded for Clarity & are Despawned Recursively on Exiting the Specific Game State
///
/// CODING:
///   Each Text Message System Spawns a Node with the "OnMessageScreen" Marker Component & Text Messages are Spawned as Children of this Node.
///   Text Messages are Styled with Custom Font, Font Size, Color, and Layout and may contain special features such as shadowing, etc

/*
    "Example of Game States with Text Messages:
        GameState::NavBoard: "Select a CELL"
        GameState::EditTemps: "Add or Remove Tempory Numbers for this Cell"
        GameState::EditPerms0: "This Cell Number is Not Editable"
        GameState::EditPerms1: "Add or Change the Main Cell Number"
        GameState::UpdateCell: "Available Cell Numbers"
*/

///  Consider using BEVY'S Message System for Text Display Messages, instead of spawning Text Nodes Directly
///  Also consider:TextBox Plugin for Text Display instead of Custom Text Display Systems
use bevy::prelude::*;

// use crate::board_resources::*;
// use crate::board_chooser::despawn_screen;
use crate::board_chooser::*; // despawn_screen
use crate::nav_board::available_nums_by_board_and_index;
// use crate::nav_board_old::available_nums_by_board_and_index;
use crate::sudoku_constants::{TEXT_BRIGHT_BLUE, TEXT_BRIGHT_GREEN, TEXT_BRIGHT_YELLOW}; // SPLASH_TEXT_1, TEXT_LIGHT_BLUE, NORMAL_BUTTON, HOVERED_BUTTON, HOVERED_PRESSED_BUTTON, PRESSED_BUTTON, TEXT_SHADOW_WHITE };  // TEXT_BRIGHT_BLUE_2,
// use crate::setup_board::*;  // { "window_root", "upper_panel", "top_text_box",  };  // { "grid_panel", "grid_panel", "grid_panel_container",  }
use crate::states::*;

// pub const TEXT_BRIGHT_BLUE: Color = Color::rgba(0.0, 0.0, 1.0);
// pub const TEXT_BRIGHT_GREEN: Color ;
// pub const TEXT_BRIGHT_YELLOW: Color ;

pub struct MessagesPlugin;

impl Plugin for MessagesPlugin {
    fn build(&self, app: &mut App) {
        let msg_states = [
            GameState::NavBoard,
            GameState::EditPerms0,
            GameState::EditPerms1,
            GameState::EditTemps,
            GameState::UpdateCell,
            GameState::NoUpdate,
            GameState::PrintBoard,
        ];

        for state in msg_states {
            app
                // .add_systems(Update, lower_text.run_if(in_state(GameState::EditTemps)))
                .add_systems(OnEnter(state), (show_text, show_text_2, lower_text))
                .add_systems(OnExit(state), despawn_screen::<OnMessageScreen>);
        }
    }
}

// 	Component for the Message Screen
#[derive(Component, Copy, Clone, Debug)]
pub struct OnMessageScreen;

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////   TEXT SYSTEM DISPLAY - show_text_Group_1   ////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */
/// 	Display Messages in the Upper Text Box Display  -  Player Information   //
pub fn show_text(
    // Upper Text Box Messages, 1st Level
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    state: Res<State<GameState>>,
) {
    let font_handle: Handle<Font> = asset_server.load("fonts/FiraCode-Bold.ttf");
    commands
        .spawn((
            Node {
                position_type: PositionType::Relative,
                top: Val::Px(32.0), // + (LINE_WTH_4 + BOARD_MAR + LINE_WTH_3 + TEXT_MAR + LINE_WTH_2)), // (15.0 + 17.0) offset from top of Main Text Box
                left: Val::Px(0.0),
                align_items: AlignItems::Start, // Center
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            OnMessageScreen,
        ))
        //
        //  //  //  UPPER TEXT PANEL - FIRST LEVEL MESSAGES - Player Instructions for Each Game State
        //
        .with_children(|parent| {
            let msg = match state.get() {
                GameState::NavBoard => "\nSelect a Cell to Edit".to_string(), // "Choose, Click or Select a CELL".to_string(),
                // GameState::EditPerms0 => "\nChoose Another Cell to Edit".to_string(), // "\nThis Cell is Not Editable".to_string(), "\nThis Cell Cannot be Edited".to_string(),
                // GameState::EditPerms0 => "This Cell is Locked".to_string(),
                GameState::EditPerms0 => "This Cell is Uneditable".to_string(),
                GameState::EditPerms1 => "Choose a Main Cell Number".to_string(),
                GameState::EditTemps => "Select Temporary Numbers to Toggle \n\n(Display up to 6 Temps)".to_string(), // \n\n Display up to 6 Temps".to_string(), // "Edit Temporary Numbers for this Cell".to_string() // "Add/Remove Temporary Numbers for this Cell".to_string()
                GameState::UpdateCell => "Update Cell".to_string(),
                // GameState::NoUpdate => "\nChoose Another CELL Number, 'NoUpdate'".to_string(), // "Select a CELL".to_string(),
                GameState::NoUpdate => "\nNot an Available Temporary Number, 'NoUpdate'".to_string(),
                GameState::PrintBoard => "Show Solution".to_string(),
                // "Add or Remove Temporary Numbers\n or".to_string() // + &"" // "Add/Remove Temporary Numbers for this Cell"
                // + "\n  Press \"P\" to Edit the Main Cell Number"
                // + &"\n\nPress \"ESC\":  Delete Temporary Numbers for this Cell"

                //     "Press \"P\" to Edit the Main Cell Number".to_string() +
                //     //    + &"\n\nPress \"ESC\" to Delete Mutable Main Cell Number"
                //     &"\n\nPress \"ESC\" to Delete the Main Cell Number".to_string()
                // }
                _ => "".to_string(),
            };
            parent.spawn((
                Text::new(msg),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(TEXT_BRIGHT_GREEN),
                TextLayout {
                    justify: Justify::Center,
                    ..default()
                },
            ));
        });
}

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////   TEXT SYSTEM DISPLAY - show_text_Group_2   ////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */
// 	Display Part-2 Messages in the Upper Text Box Display  -  Player Information   //
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn show_text_2(
    // Upper Text Box Messages, 2nd Level
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    state: ResMut<State<GameState>>,
) {
    let font_handle: Handle<Font> = asset_server.load("fonts/FiraCode-Bold.ttf");
    commands
        .spawn((
            Node {
                position_type: PositionType::Relative,
                top: Val::Px(110.0),
                left: Val::Px(0.0),
                // align_items: AlignItems::Start,  // Center
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            OnMessageScreen,
        ))
        //
        //  //  //  UPPER PANEL-SECOND LEVEL MESSAGES - Additional Player Instructions for Each Game State
        //
        .with_children(|parent| {
            let msg_2 = match state.get() {
                GameState::NavBoard => "".to_string(),
                // GameState::EditPerms0 => "This Cell is Locked".to_string(), // "This Cell is Not Editable".to_string(),
                GameState::EditPerms0 => "\n\nChoose Another Cell to Edit".to_string(),
                // GameState::EditPerms1 => "Press Esc to Remove Main Cell Number".to_string(), // "This Cell is Editable".to_string(), // "Main Cell Number is Editable".to_string(),
                GameState::EditPerms1 => "Press Esc to Remove Main Cell Number \nPress 'P' to Edit Main Cell Number".to_string(), // "Press 'P' to Edit Main Cell Number".to_string(),
                GameState::EditTemps => "Press 'P' to Edit Main Cell Number".to_string(), // "Press Esc to Remove all Temporary Numbers for this Cell".to_string() // "Press Esc to Remove all Temps".to_string(), // "Press \"ESC\" to Delete Temporary Nums for this Cell".to_string()
                GameState::UpdateCell => "Update Cell".to_string(),
                GameState::NoUpdate => "Choose a Different Temporary Number".to_string(),
                // GameState::NoUpdate => "This Main Cell Number is Not Available".to_string(),
                // GameState::NoUpdate => "Not a Valid Choice".to_string(),
                // GameState::NoUpdate => "".to_string(),

                // GameState::EditPerms0 | GameState::EditPerms1 | GameState::NoUpdate => {
                //     // "\n\nPress \"N\" ==> To Return to Board".to_string() // "To Return to Game",  Press \"N\" ==> To Deselect This Cell"
                //     // "\n\nPress \"N\":  To Deselect This Cell".to_string()
                // }
                // GameState::EditPerms1 => "\n\nPress \"N\" ==> Return to Gameboard".to_string(),
                // GameState::EditTemps => { "\nPress \"ESC\" to Delete All Temporary & Main Numbers".to_string()  // to Delete All Temporary & Main Numbers // " to Delete Temporary Nums"
                // + &"\nPress \"N\" to Deselect This Cell".to_string() // "Press Temp Number Again to Remove it"
                // }
                _ => "".to_string(),
            };
            parent.spawn((
                Text::new(msg_2),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(TEXT_BRIGHT_BLUE),
                TextLayout {
                    justify: Justify::Center,
                    ..default()
                },
            ));
        });
}

/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////   TEXT SYSTEM DISPLAY - Lower Text Box MessageDisplay  /////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */
// 	Display Messages in the Upper Text Box Display  -  Player Information   //
pub fn lower_text(
    // Lower Text Box Messages
    mut commands: Commands,
    // start_board: Res<NumsFixedRes>,
    ndx: ResMut<SelectedCellRes>,
    // mut_nums: ResMut<NumsMutRes>,
    current_board: If<ResMut<NumsMutRes>>,
    asset_server: If<Res<AssetServer>>,
    state: ResMut<State<GameState>>,
    panel_query: Query<(Entity, &Name)>,
) {
    let font_handle_fcb: Handle<Font> = asset_server.load("fonts/FiraCode-Bold.ttf");
    let values = available_nums_by_board_and_index(&current_board, ndx.0);
    //
    //  //  LOWER PANEL MESSAGES - Additional Player Instructions for Each Game State
    //
    let lower_msg = match state.get() {
        // GameState::NavBoard => format!("Available Cell Numbers ==> {:?}", values),
        GameState::EditPerms0 => "\nCell Not Editable".to_string(), // "Choose Another Cell".to_string(),  // This Cell is Not Editable".to_string()
        GameState::EditPerms1 => format!("Possible MainCell Numbers ==> {:?}", values),
        GameState::EditTemps => format!("Available Temps for this Cell  ==> {:?}", values), // "Temps Available for this Cell  ==> {:?}"
        GameState::UpdateCell => "Update this Cell".to_string(), // format!("Available Cell Numbers ==> {:?}", values),
        // GameState::NoUpdate => "Not a Valid Number !".to_string(), // "PRESS Temporary Number Again to Remove".to_string(),
        GameState::NoUpdate => "Not a Valid Choice".to_string(),
        GameState::PrintBoard => "A Solution for this Puzzle is Available".to_string(), // A Solution is Available".to_string()  // "Solved Sudoku Board Printed".to_string(),
        // GameState::NavBoard => format!("Available Cell Numbers_ANY ==> {:?}", values),
        _ => "SUDOKU".to_string(),
        // _ => format!("Available Cell Numbers_ANY ==> {:?}", values),
    };

    if let Some((lower_panel_entity, _)) = panel_query
        .iter()
        .find(|(_, name)| name.as_str() == "bottom_panel_border")
    {
        commands.entity(lower_panel_entity).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    OnMessageScreen,
                ))
                .with_children(|message_parent| {
                    message_parent.spawn((
                        Text::new(lower_msg),
                        TextFont {
                            font: font_handle_fcb.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(TEXT_BRIGHT_YELLOW),
                        TextLayout {
                            justify: Justify::Center,
                            ..default()
                        },
                    ));
                });
        });
    }
}

//  "Sudoku Board Solution Found", "Printed Solved Sudoku Board", "Main Cekk Number Changed", "Tenps for this Cell Cleared",
//

//  NEED MESSAGE TO PRESS A KEY:  to go to the  "Pause Menu"

//  "Click a temporary value to add it as cell's main number", (cell.num)

/*
impl Plugin for MessagesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::NavBoard),
            (show_text, show_text_2, lower_text),
        )
        .add_systems(
            OnExit(GameState::NavBoard),
            despawn_screen::<OnMessageScreen>,
        )
        .add_systems(
            OnEnter(GameState::EditPerms0),
            (show_text, show_text_2, lower_text),
        )
        .add_systems(
            OnEnter(GameState::EditPerms1),
            (show_text, show_text_2, lower_text),
        )
        // .add_systems(
        //     Update, (show_text, show_text_2, lower_text).run_if(in_state(GameState::EditTemps)),
        // ) // NoUpdateCell
        .add_systems(
            OnEnter(GameState::EditTemps),
            (show_text, show_text_2, lower_text),  // .run_if(in_state(GameState::ChooseBoard)
        ) // NoUpdateCell
        .add_systems(
            OnEnter(GameState::UpdateCell),
            (show_text, show_text_2, lower_text),
        )
        .add_systems(
            OnEnter(GameState::NoUpdate),
            (show_text, show_text_2, lower_text),
        )
        .add_systems(
            OnExit(GameState::NavBoard),  // (GameState::NavBoard, GameState::EditPerms0, GameState::EditPerms1, GameState::EditTemps, GameState::UpdateCell, GameState::NoUpdate,)
            despawn_screen::<OnMessageScreen>,
        )
        .add_systems(
            OnExit(GameState::EditPerms0), // (GameState::EditPerms0, GameState::EditPerms1, GameState::EditTemps, GameState::UpdateCell, GameState::NoUpdate,)
            despawn_screen::<OnMessageScreen>,
        )
        .add_systems(
            OnExit(GameState::EditPerms1),
            despawn_screen::<OnMessageScreen>,
        )
        .add_systems(
            OnExit(GameState::EditTemps),
            despawn_screen::<OnMessageScreen>,
        )
        .add_systems(
            OnExit(GameState::UpdateCell),
            despawn_screen::<OnMessageScreen>,
        )
        .add_systems(
            OnExit(GameState::NoUpdate),
            despawn_screen::<OnMessageScreen>,
        );
    }
}

*/
