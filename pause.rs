//  Create Button Hover System First,  then, Quit (and start the Game),  Later, Choose Board Difficulty
//  Reduce Font size in Pause Menu, consider increase Button width & adjust all Spacing

use bevy::app::AppExit;
use bevy::prelude::*;

// use crate::cells::spawn_cells;
use crate::board_chooser::despawn_screen;
use crate::sudoku_constants::{ SPLASH_TEXT_1, TEXT_LIGHT_BLUE, NORMAL_BUTTON, HOVERED_BUTTON, 
    HOVERED_PRESSED_BUTTON, PRESSED_BUTTON, TEXT_SHADOW_WHITE };  // TEXT_BRIGHT_BLUE_2, 

//  //  //  use crate::constants::*; // TEXT_LIGHT_BLUE, NORMAL_BUTTON, BLACK, LIGHT_BLUE;
use crate::states::*;
// use crate::cells::OnSpawnCells;


/* ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////   Plugin that displays a Startup Splash Screen with Bevy logo & text   //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////   Switching to the main menu after Final Text Message    ////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////// */
pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app // .insert_resource(SelectedOption(0))  // ? SelectedCellOption;  electedOption is a resource to track the currently selected menu option, initialized to 0 (first option)
            .add_systems(OnEnter(GameState::Menu), pause_menu_setup)
            .add_systems(
                Update,
                (menu_action, button_hover_system).run_if(in_state(GameState::Menu)),
            )
            .add_systems(
                OnEnter(GameState::SolveBoard),
                despawn_screen::<OnPauseMenuScreen>,
            )
            .add_systems(
                OnEnter(GameState::SpawnBoard),
                despawn_screen::<OnPauseMenuScreen>,
            );
        // .add_systems(OnExit(GameState::Menu), despawn_screen::<OnSpawnCells>));
    }
}

#[derive(Resource, Component, Copy, Clone)]
pub struct SelectedOption; // Tag component to indicate a selected button Resource for substate option selection

// Tag component used to tag entities added on the main menu screen
#[derive(Component)]
struct OnPauseMenuScreen;

// All actions that can be triggered from a button click
#[derive(Component)]
pub enum MenuButtonAction {
    NewGame,
    ResumeGame,
    SaveGame,
    ShowSolution,
    Quit,
}

pub fn pause_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) -> () {
    // Common style for all buttons on the screen
    let font_handle_fcode_b: Handle<Font> = asset_server.load("fonts/FiraCode-Bold.ttf");
    let button_node = Node {
        // width: percent(100),
        // height: percent(100),
        width: px(375), // Button size - width  // 300  60 X 5, - height  // #65 5 X
        height: px(75),
        border: UiRect::all(px(5.0)),
        border_radius: BorderRadius::all(px(20.0)),
        margin: UiRect::all(px(20)), // # 10.0   // Margin between buttons, all sides
        padding: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    // BackgroundColor(Color::srgba(0.99, 0.99, 0.99, 0.99));
    BorderColor::all(Color::srgba(0.60, 0.60, 0.90, 0.99));
    let button_text_font = TextFont {
        font: font_handle_fcode_b,
        font_size: 33.0,
        ..default()
    };
    let box_shadow = BoxShadow(vec![ShadowStyle {
        color: HOVERED_BUTTON, // NORMAL_BUTTON, Color::WHITE.with_alpha(0.5), Color = Color::srgb(0.0, 0.0, 0.7);
        x_offset: px(0.0),     //  px(shadow.x_offset), // 20.0
        y_offset: px(0.0),     // px(shadow.y_offset),
        spread_radius: px(5.0), // 15.0 px(shadow.spread),
        blur_radius: px(5.0),  // 10.0  // px(shadow.blur),
                               // samples: 4,  // Samples => format!("{}", shadow.samples),
    }]);
    commands.spawn((
        DespawnOnExit(GameState::Menu),
        Node {
            // width: px(300),
            // height: px(65),
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            // border: UiRect::all(px(3.0)),
            // border_radius: BorderRadius::all(px(5.0)),
            ..default()
        },
        OnPauseMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            // BackgroundColor(BLACK.into()), // BLACK, CRIMSON
            children![
                // Display the game name
                (
                    Text::new("< GAME MENU >"), //  "|MAIN MENU|", "Bevy Game Menu UI"
                    TextFont {
                        font_size: 60.0, // 60.0
                        ..default()
                    },
                    TextColor(SPLASH_TEXT_1.into()), // LIGHT_BLUE, YELLOW, MENU_TITLE
                    Node {
                        margin: UiRect::bottom(px(40.)), // 100.0, 320.0, 220.0, 180.0, 80.0, #40.0 -raise to 100
                        ..default()
                    },
                    TextShadow {
                        // color: TEXT_BRIGHT_BLUE_2.into(),
                        color: TEXT_SHADOW_WHITE.into(),
                        // color: SPLASH_TEXT_1.into(),  // TEXT_SHADOW_WHITE, TEXT_SHADOW_0, TEXT_BRIGHT_BLUE_2
                        offset: Vec2::new(2.0, 0.50), //  //  //  Shadow offset (x, y)
                                                      // blur_radius: 4.0,           //  //  // Shadow blur radius
                    },
                ),
                (
                    Button,
                    button_node.clone(),
                    box_shadow.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::NewGame,
                    children![
                        // (ImageNode::new(right_icon), button_icon_node.clone()),
                        (
                            Text::new("New Game"),
                            button_text_font.clone(),
                            TextColor(TEXT_LIGHT_BLUE),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    box_shadow.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::ResumeGame,
                    children![
                        // (ImageNode::new(wrench_icon), button_icon_node.clone()),
                        (
                            Text::new("Resume Game"), // Go to Board Chooser
                            button_text_font.clone(),
                            TextColor(TEXT_LIGHT_BLUE),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    box_shadow.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::SaveGame,
                    children![
                        // (ImageNode::new(wrench_icon), button_icon_node.clone()),
                        (
                            Text::new("Save Game"),
                            button_text_font.clone(),
                            TextColor(TEXT_LIGHT_BLUE),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    box_shadow.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::ShowSolution,
                    children![
                        // (ImageNode::new(wrench_icon), button_icon_node.clone()),
                        (
                            Text::new("Show Solution"),
                            button_text_font.clone(),
                            TextColor(TEXT_LIGHT_BLUE),
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
                            Text::new("Quit"),
                            button_text_font,
                            TextColor(TEXT_LIGHT_BLUE.into()),
                        ),
                    ]
                ),
            ]
        )],
    ));
}

// next_state.set(GameState::ChooseBoard);

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
    mut app_exit_writer: MessageWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    // Exit the App
                    app_exit_writer.write(AppExit::Success);
                }
                MenuButtonAction::ResumeGame => {
                    // Resume the Existing Game
                    next_state.set(GameState::SpawnBoard);
                    //     game_state.set(PauseState::Game);
                    //     menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::SaveGame => {
                    // Save the Current Game State to a text file
                    next_state.set(GameState::SolveBoard)
                    // MenuButtonAction::SettingsDisplay => {
                    //     menu_state.set(MenuState::SettingsDisplay);
                }
                MenuButtonAction::NewGame => {
                    next_state.set(GameState::ChooseBoard);
                    // next_state.set(GameState::SpawnBoard);
                }
                // MenuButtonAction::ShowSolution => menu_state.set(MenuState::Main),
                MenuButtonAction::ShowSolution => {
                    next_state.set(GameState::SolveBoard);
                    //     menu_state.set(MenuState::Settings);
                } // _ => {
                  // }
            }
        }
    }
}

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Generic system that takes a component as a parameter, and will recursively despawn all entities with that component
/*
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
*/
