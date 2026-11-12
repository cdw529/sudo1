//
use crate::board_chooser::{EditMode, NumsMutRes, despawn_screen};
use crate::nav_board::Ndx;
use crate::states::GameState;
use crate::sudoku_constants::{
    BACKGROUND_NORMAL, BOARD_LINES_BLU_GRY_3, BORDER_NORM_NUMS, BORDER_NORM_TMPS, NUMS_PERM,
    NUMS_TEMP };
//

//
// use bevy::picking::Pickable;
use bevy::ui::widget::Button;
// use bevy::ui::widget;
// use bevy::ui_render::stack_z_offsets::BACKGROUND_COLOR;
use bevy::prelude::*; // BORDER_WHITE
//

//
pub struct SetupBoardPlugin;
//

//
impl Plugin for SetupBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::SpawnBoard),
            (
                despawn_screen::<OnSpawnCells>,
                spawn_board,
                transition_to_nav_board,
            )
                .chain(),
        );
    }
}

pub fn transition_to_nav_board(
    edit_mode: Res<EditMode>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    match *edit_mode {
        EditMode::Navigate => next_state.set(GameState::NavBoard),
        EditMode::TempNotes => next_state.set(GameState::EditTemps),
        EditMode::MainNumber => next_state.set(GameState::EditPerms1),
        EditMode::LockedCell => next_state.set(GameState::EditPerms0),
    }
}

#[derive(Resource, Debug, Clone, Eq, PartialEq, Component)]
pub struct Cell {
    pub ndx: usize,
    pub num: u8,
    pub temps: Vec<u8>,
}

impl Cell {
    pub fn new(ndx: usize, num: u8, temps: Vec<u8>) -> Cell {
        Cell {
            ndx,
            num,
            temps, // Clone the temps vector to ensure the Cell has its own copy of the data, instead of referencing the original vector
        }
    }
}

impl Default for Cell {
    fn default() -> Cell {
        Cell {
            ndx: 0,
            num: 0,
            temps: [0; 9].to_vec(),
        }
    }
}

#[derive(Resource, Clone, Debug, Deref, DerefMut)]
pub struct AllTempsRes {
    pub temps_data: [Vec<u8>; 81],
}

impl AllTempsRes {
    pub fn new(temps: Vec<u8>) -> AllTempsRes {
        AllTempsRes {
            temps_data: std::array::from_fn(|_| temps.clone()), // TODO: Consider using .clamp(6) or .truncate(6)  to limit temps to 6 or fewer, for better display in the cell
        }
        // AllTempsRes { temps_data: std::array::from_fn(|_| temps.clone()) }
        // AllTempsRes {temps_data: std::array::from_fn(|_| Vec::new().with_capacity(9)) }
    }
}

// TODO!    // : Consider initializing
impl Default for AllTempsRes {
    fn default() -> AllTempsRes {
        AllTempsRes {
            temps_data: std::array::from_fn(|_| [0; 9].to_vec()), // TODO: Consider using .clamp(6) or .truncate(6) to limit temps to 6 or fewer, for better display in the cell
        } // Initialize each of the 81 vectors with nine zeros, for each cell  // AllTempsRes { temps_data: std::array::from_fn(|_| Vec::new().with_capacity(9)) }  // Initialize each of the 81 vectors as empty, with capacity for 9 temps each
    }
}
//

//
// Marker Component to identify the initial board setup entities, for despawning later
#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct OnSpawnCells;
//

//
pub fn spawn_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_board: Res<NumsMutRes>,
    all_temps: Res<AllTempsRes>,
) -> () {
    const CUBE_SEPARATOR_PX: f32 = 10.0; // 8.0
    let font_bold: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font_mono: Handle<Font> = asset_server.load("fonts/FiraMono-Medium.ttf");
    commands
        .spawn((
            Node {
                display: Display::Flex,
                position_type: PositionType::Relative, //  Relative, Absolute`
                flex_direction: FlexDirection::Column,
                box_sizing: BoxSizing::BorderBox,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(4.)),
                padding: UiRect::all(Val::Px(4.)),
                width: Val::Percent(100.), // WINDOW_WTH = 766.0   750.0  736.0
                height: Val::Percent(100.), // WINDOW_HT = 1016.0  1000.0  986.0
                //
                grid_template_columns: vec![GridTrack::percent(100.0)], // Create a Single column, full inner width after padding
                // Create three rows: Upper Tezt Panel, Game Grid Panel and Lower Text Panel (in specified proportions)
                grid_template_rows: vec![
                    GridTrack::percent(17.5),
                    GridTrack::flex(1.0), // flex(1.0) --> Game Grid Panel, expands to fill the remaining area between the upper and lower panels
                    GridTrack::percent(7.5),
                ],
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.15, 0.25, 0.99)),
            BorderColor::all(BOARD_LINES_BLU_GRY_3),
            Name::new("window_root"),
            OnSpawnCells,
        ))
        // First of Three Children:  The Upper Text Container
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        box_sizing: BoxSizing::BorderBox,
                        position_type: PositionType::Relative, // Relative, Absolute
                        width: Val::Percent(100.),             // 750.0, 97.91
                        height: Val::Percent(17.5),            // 175.0
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(4.)),
                        border: UiRect {
                            left: Val::Px(3.),
                            right: Val::Px(3.),
                            top: Val::Px(3.),
                            // bottom: Val::Px(0.),
                            bottom: Val::Px(3.),
                        },
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderColor::all(BOARD_LINES_BLU_GRY_3),
                    Name::new("top_panel"), // "top_panel_container"
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            width: Val::Percent(100.0),  // 736.0
                            height: Val::Percent(100.0), // 170.0 -14.0
                            border: UiRect::all(Val::Px(1.0)),
                            padding: UiRect::all(Val::Px(4.0)),
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        BorderColor::all(BOARD_LINES_BLU_GRY_3),
                        Name::new("top_text_box"), // "top_text_container", "top_text_box"
                    ));
                });
            // Second Child of Three Children (panels) for "grid_panel": The main game_board container
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        box_sizing: BoxSizing::BorderBox,
                        position_type: PositionType::Relative,
                        width: Val::Percent(100.00), // 750.0
                        height: Val::Percent(75.00), // 750.0
                        // aspect_ratio: Some(1.0),
                        border: UiRect::all(Val::Px(3.0)),
                        padding: UiRect::all(Val::Px(4.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        // align_self: AlignSelf::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderColor::all(BOARD_LINES_BLU_GRY_3),
                    Name::new("grid_panel"), // "grid_panel_container"
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                // This is the GRID Container for the 9 X 9 game_board, with 11 tracks in Vert & Horz:  9 cell tracks + 2 cube separator tracks
                                display: Display::Grid,
                                box_sizing: BoxSizing::BorderBox,
                                position_type: PositionType::Relative,
                                width: Val::Percent(100.0),  // ~736.00
                                height: Val::Percent(100.0), // ~736.00
                                border: UiRect::all(Val::Px(1.0)),
                                padding: UiRect::all(Val::Px(4.0)), // 8.0// 4.0//
                                justify_items: JustifyItems::Center,
                                align_items: AlignItems::Center,
                                //  This draws 3x3 cube divider lines without changing per-cell borders/outlines.
                                grid_template_columns: vec![
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                    GridTrack::px(CUBE_SEPARATOR_PX),
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                    GridTrack::px(CUBE_SEPARATOR_PX),
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                ],
                                grid_template_rows: vec![
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                    GridTrack::px(CUBE_SEPARATOR_PX),
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                    GridTrack::px(CUBE_SEPARATOR_PX),
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                    GridTrack::flex(1.0),
                                ],
                                ..default()
                            },
                            // BackgroundColor(Color::srgba(0.24, 0.24, 0.34, 1.00)),
                            BackgroundColor(Color::BLACK),
                            BorderColor::all(BOARD_LINES_BLU_GRY_3),
                            Name::new("grid_panel"), // grid_panel_box, "grid_panel_container"
                        ))
                            // GRID_CUBE-SEPARATORS:  Intrinsically spawned Cube-Separators, not Extrinsically located on grid !!
                            // Spawn V1: the VERTICAL-CUBE-SEPARATORS:  Skips "Cell" tracks, Adds separators on the 3px gap tracks at Column-Tracks 4 and 8, only.
                        .with_children(|parent| {
                            for track in 1..=11_i16 {
                                parent.spawn((
                                    Node {
                                        grid_row: GridPlacement::start(track),
                                        grid_column: GridPlacement::start(4),
                                        width: px(10.0),
                                        height: percent(100.0),
                                        border: UiRect {
                                            left: Val::Px(4.75),
                                            right: Val::Px(4.75),
                                            top: Val::Px(0.0),
                                            bottom: Val::Px(0.0),
                                        },
                                        // width: Val::Percent(100.0),
                                        // height: Val::Percent(100.0),
                                        // border: UiRect::all(Val::Px(0.0)),  // Separater line thickness
                                        // padding: UiRect::all(Val::Px(2.0)),  //Separater line padding to prevent overlap with cell borders/outlines
                                        justify_self: JustifySelf::Center,
                                        align_self: AlignSelf::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(1.00, 1.00, 1.00, 1.00)),
                                    // BorderColor::all(None),
                                    Pickable::IGNORE,
                                    Name::new(format!("cube_sep_v1_seg_{track}")),  // "grid_cube_separator(s)",  "grid_panel_container" // "grid_cube_separator(s)",  "grid_panel_container"
                                ));
                                //          Spawn V2: the HORIZONTAL-CUBE-SEPARATORS:  On Row-Tracks 4 and 8
                                parent.spawn((
                                    Node {
                                        grid_row: GridPlacement::start(track),
                                        grid_column: GridPlacement::start(8),
                                        width: px(10.0),
                                        height: percent(100.0),
                                        border: UiRect {
                                            left: Val::Px(4.75),
                                            right: Val::Px(4.75),
                                            top: Val::Px(0.0),
                                            bottom: Val::Px(0.0),
                                        },
                                        // width: Val::Percent(100.0),
                                        // height: Val::Percent(100.0),
                                        // padding: UiRect::all(Val::Px(4.0)),
                                        justify_self: JustifySelf::Center,
                                        align_self: AlignSelf::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(1.00, 1.00, 1.00, 1.00)),
                                    // BorderColor::all(Color::BLACK),
                                    Pickable::IGNORE,
                                    Name::new(format!("cube_sep_v2_seg_{track}")), // "grid_cube_separator(s)"
                                ));
                                // Spawn: HORIZONTAL-CUBE-SEPARATORS:  On Row-Tracks 4 and 8
                                parent.spawn((
                                    Node {
                                        grid_row: GridPlacement::start(4),
                                        grid_column: GridPlacement::start(track),
                                        width: percent(100.0),
                                        height: px(10.0),
                                        border: UiRect {
                                            left: Val::Px(0.00),
                                            right: Val::Px(0.00),
                                            top: Val::Px(4.75),
                                            bottom: Val::Px(4.75),
                                        },
                                        // width: Val::Percent(100.0),
                                        // height: Val::Percent(100.0),
                                        // padding: UiRect::all(Val::Px(4.0)),
                                        justify_self: JustifySelf::Center,
                                        align_self: AlignSelf::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(1.00, 1.00, 1.00, 1.00)),
                                    // BorderColor::all(Color::BLACK),
                                    Pickable::IGNORE,
                                    Name::new(format!("cube_sep_h1_seg_{track}")), // "grid_cube_separator(s)"
                                ));
                                parent.spawn((
                                    Node {
                                        grid_row: GridPlacement::start(8),
                                        grid_column: GridPlacement::start(track),
                                        width: percent(100.0),
                                        height: px(10.0),
                                        border: UiRect {
                                            left: Val::Px(0.00),
                                            right: Val::Px(0.00),
                                            top: Val::Px(4.75),
                                            bottom: Val::Px(4.75),
                                        },
                                        // width: Val::Percent(100.0),
                                        // height: Val::Percent(100.0),
                                        // padding: UiRect::all(Val::Px(4.0)),
                                        justify_self: JustifySelf::Center,
                                        align_self: AlignSelf::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(1.00, 1.00, 1.00, 1.00)),
                                    // BorderColor::all(Color::BLACK),
                                    Pickable::IGNORE,
                                    Name::new(format!("cube_sep_h2_seg_{track}")), // "grid_cube_separator(s)"
                                ));
                            }
                            for i in 0..81 {
                                let num = current_board.mutable_data[i];
                                let temps = all_temps.temps_data[i].clone();
                                let row = i / 9;
                                let col = i % 9;
                                // Convert zero-based(0) Sudoku Board row/col to one-based(1) grid line starts,  // skipping separator tracks at 4 and 8
                                let grid_row_track = (row + 1 + (row / 3)) as i16;
                                let grid_col_track = (col + 1 + (col / 3)) as i16;
                                let mut cell = parent.spawn((
                                    Node {
                                        display: Display::Flex,
                                        grid_row: GridPlacement::start(grid_row_track),
                                        grid_column: GridPlacement::start(grid_col_track),
                                        // left: Val::Px(5.5),
                                        // top: Val::Px(-4.0),
                                        width: Val::Percent(85.0), // percent of cell size
                                        height: Val::Percent(85.0), // 85.0
                                        border: UiRect::all(Val::Px(5.0)),
                                        border_radius: BorderRadius::all(Val::Px(2.0)),
                                        padding: UiRect::all(Val::Px(1.00)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    Button, // Makes cells "PICKABLE"
                                    // Pickable::default(), // ← Enables picking for the cell, allowing it to receive pointer events like clicks and hovers
                                    // BackgroundColor(Color::srgba(0.15, 0.15, 0.40, 0.99)),
                                    BackgroundColor(BACKGROUND_NORMAL),
                                    BorderColor::all(if num != 0 {
                                        BORDER_NORM_NUMS
                                    } else {
                                        BORDER_NORM_TMPS
                                    }),
                                    Outline {
                                        width: Val::Px(1.5),
                                        offset: Val::Px(2.5),
                                        color: Color::srgba(1.00, 1.00, 1.00, 1.00), // CELL_OUTLINE_COLOR,
                                    },
                                    Ndx(i), // index component for identifying the cell in queries, instead of using row/col
                                    // AllTempsRes::new(temps.clone()),  // Add an AllTempsRes component to each Cell, initialized with the temps vector for that cell
                                    Cell::new(i, num, temps.clone()), // Clone the temps vector to ensure each Cell has its own copy of the data, instead of referencing the same vector
                                    // commands.insert_resource(Cell{ ndx: i, num: num, temps: temps.clone() }),
                                    Name::new(format!("cell_{i}")), // Spawn each cell with its index, num, temps and locked status as a Cell component, for use in queries and updates
                                                                    // eprintln!("Cell name {} for index {}", cell.name, i),
                                                                    // Pickable::default(),
                                                                    // Observer
                                ));
                                cell.observe(|mut event: On<Pointer<Click>>| {
                                    // Read the underlying pointer event data
                                    println!("Pointer {:?} was just clicked!", event.pointer_id);
                                    // Stop the event from bubbling up the entity hierarchy
                                    event.propagate(false);
                                });
                                // eprintln!("Cell Name: {}, Cell index {}", format!("cell_{i}"), i);
                                if num != 0 {
                                    cell.with_children(|c| {
                                        c.spawn((
                                            Text::new(num.to_string()),
                                            TextFont {
                                                font: font_bold.clone(),
                                                font_size: 42.0,
                                                ..default()
                                            },
                                            TextColor(NUMS_PERM),
                                            Pickable::IGNORE,
                                            // Pickable{ should_block_lower: true, is_hoverable: true },
                                            ZIndex(5),
                                        ));
                                    });
                                } else if !temps.is_empty() {
                                    cell.with_children(|c| {
                                        c.spawn((
                                            Node {
                                                display: Display::Grid,
                                                // position_type: PositionType::Relative,
                                                top: Val::Px(-2.0),
                                                left: Val::Px(1.0),
                                                grid_auto_flow: GridAutoFlow::Column,
                                                // flex_wrap: FlexWrap::Wrap,
                                                column_gap: Val::Px(4.0),
                                                row_gap: Val::Px(4.0),
                                                width: Val::Percent(88.0),
                                                height: Val::Percent(88.0), // 92.0 to allow for padding and gaps between temps
                                                justify_content: JustifyContent::Start, // FlexStart
                                                align_items: AlignItems::Start, // FlexStart
                                                grid_template_columns: vec![
                                                    GridTrack::percent(70.0),
                                                    GridTrack::percent(30.0),
                                                    GridTrack::flex(1.0), // extra row to hold temp > 6, if needed but not visible & without breaking the 3x3 grid layout of temps & in leu of .clanp() or .limit()
                                                ],
                                                grid_template_rows: vec![
                                                    GridTrack::flex(1.0),
                                                    GridTrack::flex(1.0),
                                                    GridTrack::flex(1.0),
                                                ],
                                                ..default()
                                            },
                                            Pickable::IGNORE,
                                            Name::new("temps_container"),
                                        ))
                                        .with_children(
                                            |t_parent| {
                                                for &t in temps.iter().filter(|&&t| t != 0) {
                                                    t_parent.spawn((
                                                        Text::new(t.to_string()),
                                                        TextFont {
                                                            font: font_mono.clone(),
                                                            font_size: 14.0,
                                                            ..default()
                                                        },
                                                        TextColor(NUMS_TEMP),
                                                        Pickable::IGNORE,
                                                        ZIndex(4),
                                                    ));
                                                }
                                            },
                                        );
                                    });
                                }
                            }
                        });
                }); // Third Child of Three, the Lower Text Messages Container
            parent
                .spawn((
                    Node {
                        box_sizing: BoxSizing::BorderBox,
                        width: Val::Percent(100.0), // 726.0
                        height: Val::Percent(7.5),  // (75.0 - 16.0  -8.0)  9.615 %
                        padding: UiRect::all(Val::Px(4.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect {
                            top: Val::Px(0.0),
                            bottom: Val::Px(3.0),
                            left: Val::Px(3.0),
                            right: Val::Px(3.0),
                        },
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderColor::all(BOARD_LINES_BLU_GRY_3),
                    Name::new("bottom_panel"),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            box_sizing: BoxSizing::BorderBox,
                            width: Val::Percent(100.0), // 736.0
                            height: Val::Percent(100.), // 65.0
                            border: UiRect::all(Val::Px(1.0)),
                            padding: UiRect::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            justify_self: JustifySelf::Center,
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        BorderColor::all(BOARD_LINES_BLU_GRY_3),
                        Name::new("bottom_panel_border"),
                    ));
                });
        });
}

// Ignore all code below this line !

// Ignore all code below

/*
    .observe(|click: On<Pointer<Click>>, mut commands: Commands| {
    commands.entity(click.entity)
        .insert(Interaction::Pressed)
}),
    .observe(|hover: On<Pointer<Hover>>, mut commands: Commands| {
        commands.entity(hover.entity).insert(Interaction::Hovered);
    })
*/

/*
    // ─── Important markers ───────────────────────────────────────
    Cell::default(), // your Cell component
                     // You can later add:
                     // Name::new(format!("cell_{}", i)),
                     // or Interaction { .. } if you want click handling
*/

//  UiButton::rectangle().background_color(palettes::css::GOLDENROD.into());

/*
pub fn on_cell_click(mut commands: Commands, click: On<Pointer<Click>>) {
    println!("Cell clicked: {:?}", click.entity);
    // You can also add logic here to update the cell's state, e.g. toggle a temp number, or select the cell for input
}
*/

/*
   .observe(|click: On<Pointer<Click>>, mut commands: Commands| {
        commands.entity(click.entity).insert(Interaction::Clicked);
    })
    .observe(|hover: On<Pointer<Click>>, mut commands: Commands| {
        commands.entity(hover.entity).insert(Interaction::Hovered);
    }),
    .observe(|mut event: On<Pointer<Click>>| {
            // Read the underlying pointer event data
        println!("Pointer {:?} was just clicked!", event.pointer_id);
            // Stop the event from bubbling up the entity hierarchy
        event.propagate(false);
    }),

    .observe(|mut event: On<Pointer<Click>>| {
        // Read the underlying pointer event data
        println!("Pointer {:?} was just clicked!", event.pointer_id);
        // Stop the event from bubbling up the entity hierarchy
        event.propagate(false);
    });
*/
