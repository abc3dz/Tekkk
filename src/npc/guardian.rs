use bevy::prelude::*;
use bevy::gltf::GltfAssetLabel;
use avian3d::prelude::*;
use bevy::animation::graph::AnimationGraph;
use bevy::animation::AnimationPlayer;
use bevy::input::gamepad::{Gamepad, GamepadButton};
use crate::components::*;
use crate::npc::{
    advanced_practice::AdvancedPracticePlugin,
    basic_practice::BasicPracticePlugin,
    basic_practice::spawn_basic_practice_gun,
    advanced_practice::spawn_advanced_minion,
    practice_common::PracticeCommonPlugin,
};
use crate::cel_shader::*;
use crate::pause_menu::GameMode;

pub struct GuardianPlugin;

impl Plugin for GuardianPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                BasicPracticePlugin,
                AdvancedPracticePlugin,
                PracticeCommonPlugin,
            ))
            .init_resource::<BasicPracticeActive>()
            .init_resource::<AdvancedPracticeActive>()
            .init_resource::<GuardianMenuSelection>()
            .init_resource::<GuardianDialogOpen>()
            .init_resource::<GuardianDialogEscConsumed>()
            .init_resource::<GuardianAllocSelection>()
            .init_resource::<GuardianDialogFocus>()
            .init_resource::<GuardianAllocPointSelection>()
            .init_resource::<GuardianAllocIncreaseRepeat>()
            .init_resource::<GuardianAllocDecreaseRepeat>()
            .insert_resource(BasicGunRespawnTimer(Timer::from_seconds(1.0, TimerMode::Once)))
            .insert_resource(AdvancedMinionRespawnTimer(Timer::from_seconds(1.0, TimerMode::Once)))
            .add_systems(Startup, setup_guardian_animation_graph)
            .add_systems(First, reset_guardian_esc_consumed)
            .add_systems(OnEnter(GameScene::Hub), spawn_guardian_npc)
            .add_systems(Update, setup_guardian_animation_player.run_if(in_state(GameScene::Hub)))
            .add_systems(Update, (
                check_guardian_interaction_area,
                check_guardian_interaction_area_exit,
                show_guardian_dialog,
                guardian_dialog_esc_input.after(show_guardian_dialog),
                update_guardian_focus_visuals.after(guardian_dialog_esc_input),
                guardian_menu_keyboard.after(update_guardian_focus_visuals),
                guardian_menu_confirm_keyboard.after(guardian_menu_keyboard),
                guardian_atk_selection_keyboard.after(guardian_dialog_esc_input),
                guardian_alloc_point_keyboard.after(guardian_atk_selection_keyboard),
                update_guardian_alloc_ui
                    .after(update_guardian_focus_visuals)
                    .after(guardian_alloc_point_keyboard)
                    .after(guardian_atk_selection_keyboard),
                update_guardian_controls_language_and_focus.after(update_guardian_focus_visuals),
                cleanup_guardian_ui_when_player_leave,
            ).run_if(in_state(GameScene::Hub).and(in_state(GameMode::Playing))),)
            .add_systems(OnExit(GameScene::Hub), despawn_hub_only_entities);
    }
}

pub fn reset_guardian_esc_consumed(mut consumed: ResMut<GuardianDialogEscConsumed>) {
    consumed.0 = false;
}

pub fn spawn_guardian_npc(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((
            HubOnly,
            Npc,
            GuardianNpc,
            Transform {
                translation: Vec3::new(-8.0, 1.25, -6.0),
                ..default()
            },
            RigidBody::Static,
            Collider::capsule(0.45, 1.6),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("npc/Guardian.glb"))),
                Transform::from_xyz(0.0, -1.25, 0.0),
                ApplyToonMaterial
            ));
            parent.spawn((
                GuardianInteractArea,
                Sensor,
                CollisionEventsEnabled,
                Collider::cuboid(1.4, 2.0, 1.4),
                Transform::from_xyz(0.0, 0.0, 1.5),
            ));
        });
}

pub fn setup_guardian_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();
    let idle = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(2).from_asset("npc/Guardian.glb")),
        1.0,
        graph.root,
    );
    let welcome = graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(3).from_asset("npc/Guardian.glb")),
        1.0,
        graph.root,
    );
    let graph_handle = graphs.add(graph);
    commands.insert_resource(GuardianAnimationGraph {
        graph: graph_handle,
        idle,
        welcome,
    });
}

pub fn setup_guardian_animation_player(
    mut commands: Commands,
    anim_graph: Res<GuardianAnimationGraph>,
    mut query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parent_query: Query<&ChildOf>,
    guardian_query: Query<(), With<GuardianNpc>>,
) {
    for (entity, mut player) in &mut query {
        if !is_child_of_guardian(entity, &parent_query, &guardian_query) {
            continue;
        }
        println!("Guardian AnimationPlayer found");
        commands.entity(entity).insert((
            AnimationGraphHandle(anim_graph.graph.clone()),
            GuardianAnimationTarget,
            GuardianAnimState::Idle,
        ));
        player.stop_all();
        player.play(anim_graph.idle).repeat();
    }
}

fn is_child_of_guardian(
    mut entity: Entity,
    parent_query: &Query<&ChildOf>,
    guardian_query: &Query<(), With<GuardianNpc>>,
) -> bool {
    loop {
        if guardian_query.get(entity).is_ok() {
            return true;
        }
        let Ok(parent) = parent_query.get(entity) else {
            return false;
        };
        entity = parent.0;
    }
}

pub fn check_guardian_interaction_area(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionStart>,
    guardian_area_query: Query<Entity, With<GuardianInteractArea>>,
    player_query: Query<Entity, With<Player>>,
    anim_graph: Res<GuardianAnimationGraph>,
    mut guardian_anim_query: Query<&mut AnimationPlayer, With<GuardianAnimationTarget>>,
) {
    for event in collision_events.read() {
        let collider1 = event.collider1;
        let collider2 = event.collider2;
        let body1 = event.body1.unwrap_or(collider1);
        let body2 = event.body2.unwrap_or(collider2);
        let hit_guardian_area =
            guardian_area_query.get(collider1).is_ok() || guardian_area_query.get(collider2).is_ok();
        
        if !hit_guardian_area {
            continue;
        }
        
        let player_entity =
            if player_query.get(body1).is_ok() {
                Some(body1)
            } else if player_query.get(body2).is_ok() {
                Some(body2)
            } else if player_query.get(collider1).is_ok() {
                Some(collider1)
            } else if player_query.get(collider2).is_ok() {
                Some(collider2)
            } else {
                None
            };
            
        if let Some(player_entity) = player_entity {
            println!("Player entered Guardian area");
            commands.entity(player_entity).insert(PlayerInGuardianArea);
            for mut anim_player in &mut guardian_anim_query {
                anim_player.stop_all();
                anim_player.play(anim_graph.welcome);
            }
        }
    }
}

pub fn check_guardian_interaction_area_exit(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEnd>,
    guardian_area_query: Query<Entity, With<GuardianInteractArea>>,
    player_query: Query<Entity, With<Player>>,
    anim_graph: Res<GuardianAnimationGraph>,
    mut guardian_anim_query: Query<&mut AnimationPlayer, With<GuardianAnimationTarget>>,
) {
    for event in collision_events.read() {
        let collider1 = event.collider1;
        let collider2 = event.collider2;
        let body1 = event.body1.unwrap_or(collider1);
        let body2 = event.body2.unwrap_or(collider2);
        let hit_guardian_area =
            guardian_area_query.get(collider1).is_ok() || guardian_area_query.get(collider2).is_ok();
            
        if !hit_guardian_area {
            continue;
        }
        
        let player_entity =
            if player_query.get(body1).is_ok() {
                Some(body1)
            } else if player_query.get(body2).is_ok() {
                Some(body2)
            } else if player_query.get(collider1).is_ok() {
                Some(collider1)
            } else if player_query.get(collider2).is_ok() {
                Some(collider2)
            } else {
                None
            };
            
        if let Some(player_entity) = player_entity {
            commands.entity(player_entity).remove::<PlayerInGuardianArea>();
            for mut anim_player in &mut guardian_anim_query {
                anim_player.stop_all();
                anim_player.play(anim_graph.idle).repeat();
            }
        }
    }
}

pub fn show_guardian_dialog(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    in_area_query: Query<(), With<PlayerInGuardianArea>>,
    player_alloc_query: Query<&AtkAndDefElement, With<Player>>,
    dialog_query: Query<Entity, With<GuardianDialogUI>>,
    mut selection: ResMut<GuardianMenuSelection>,
    mut atk_selection: ResMut<GuardianAllocSelection>,
    mut point_selection: ResMut<GuardianAllocPointSelection>,
    mut focus: ResMut<GuardianDialogFocus>,
    fonts: Res<GameFonts>,
    loc: Res<Localization>,
    mut dialog_open: ResMut<GuardianDialogOpen>,
) {
    if in_area_query.is_empty() {
        return;
    }
    
    if !dialog_query.is_empty() {
        dialog_open.0 = true;
        return;
    }
    
    selection.index = 0;
    *focus = GuardianDialogFocus::Menu;
    
    let current_index = match player_alloc_query.single() {
        Ok(atk_element) => ALL_ELEMENTS
            .iter()
            .position(|element| *element == atk_element.0)
            .unwrap_or(0),
        Err(_) => 0,
    };
    atk_selection.index = current_index;
    point_selection.index = current_index;
    
    commands
        .spawn((
            GuardianDialogUI,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.60)),
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Percent(85.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                margin: UiRect::top(Val::Auto), // 👈 ดูดพื้นที่ว่างด้านบนทั้งหมด = ดันทั้งก้อนลงชิดล่าง
                ..default()
            })
            .with_children(|outer| {
                // แผงหลัก: รูป + เมนู + Allocation
                outer
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(300.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(24.0),
                            padding: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.78)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            ImageNode::new(asset_server.load("npc/GuardianWelcome.png")),
                            Node {
                                width: Val::Px(150.0),
                                height: Val::Px(150.0),
                                ..default()
                            },
                        ));

                        parent
                            .spawn((
                                GuardianMenuPanel,
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(10.0),
                                    flex_grow: 1.0,
                                    padding: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.16, 0.16, 0.22, 0.9)),
                            ))
                            .with_children(|menu| {
                                spawn_guardian_button(menu, loc.get("basic_practice"), GuardianMenuAction::BasicPractice, &fonts);
                                spawn_guardian_button(menu, loc.get("advanced_practice"), GuardianMenuAction::AdvancedPractice, &fonts);
                                spawn_guardian_button(menu, loc.get("full_hp_mana"), GuardianMenuAction::FullHpMana, &fonts);
                                spawn_guardian_button(menu, loc.get("element_allocation"), GuardianMenuAction::ElementAllocation, &fonts);
                            });
                    });

                // แผงบอกปุ่มควบคุม
                spawn_guardian_controls_panel(outer, &fonts, &loc);
            });
        });

    dialog_open.0 = true;
}

pub fn guardian_dialog_esc_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut commands: Commands,
    dialog_query: Query<Entity, With<GuardianDialogUI>>,
    mut dialog_open: ResMut<GuardianDialogOpen>,
    mut esc_consumed: ResMut<GuardianDialogEscConsumed>,
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<Entity, With<GuardianAllocWindow>>, 
    mut focus: ResMut<GuardianDialogFocus>
) {
    if !dialog_open.0 {
        return;
    }
    let close_pressed = keyboard.just_pressed(KeyCode::Escape)
        || any_gp_just_pressed(&gamepads, GamepadButton::Start);
    if close_pressed {
        // จังหวะแรก: ถ้าหน้าต่างจัดสรรเปิดอยู่ ให้ปิดมันก่อน ยังไม่ปิด dialog
        if !window_query.is_empty() {
            for entity in &window_query {
                commands.entity(entity).despawn();
            }
            *focus = GuardianDialogFocus::Menu;
            esc_consumed.0 = true;
            return;
        }

        // จังหวะสอง: ปิด dialog ทั้งอัน (โค้ดเดิม)
        for entity in &dialog_query {
            commands.entity(entity).despawn();
        }
        if let Ok(mut transform) = player_query.single_mut() {
            transform.translation.z += 2.5;
        }
        dialog_open.0 = false;
        esc_consumed.0 = true;
    }
}

fn spawn_guardian_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: GuardianMenuAction,
    fonts: &GameFonts
) {
    parent
        .spawn((
            GuardianMenuButton,
            action,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(text),
                TextFont { font: fonts.abc3dz.clone(), font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn guardian_menu_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    dialog_open: Res<GuardianDialogOpen>,
    focus: Res<GuardianDialogFocus>,
    mut selection: ResMut<GuardianMenuSelection>,
    basic_active: Res<BasicPracticeActive>,
    advanced_active: Res<AdvancedPracticeActive>,
    window_query: Query<(), With<GuardianAllocWindow>>, // เพิ่ม
    mut button_query: Query<(&GuardianMenuAction, &mut BackgroundColor), With<GuardianMenuButton>>,
) {
    if !dialog_open.0 {
        return;
    }
    const MENU_COUNT: usize = 4;
    if selection.index >= MENU_COUNT {
        selection.index = 0;
    }
    if *focus == GuardianDialogFocus::Menu {
        let up_pressed = keyboard.just_pressed(KeyCode::ArrowUp)
            || keyboard.just_pressed(KeyCode::KeyW)
            || any_gp_just_pressed(&gamepads, GamepadButton::DPadUp);
        let down_pressed = keyboard.just_pressed(KeyCode::ArrowDown)
            || keyboard.just_pressed(KeyCode::KeyS)
            || any_gp_just_pressed(&gamepads, GamepadButton::DPadDown);
        if up_pressed {
            if selection.index == 0 {
                selection.index = MENU_COUNT - 1;
            } else {
                selection.index -= 1;
            }
        }
        if down_pressed {
            selection.index = (selection.index + 1) % MENU_COUNT;
        }
    }
    let menu_focused = *focus == GuardianDialogFocus::Menu;
    let window_open = !window_query.is_empty();
    for (action, mut background) in &mut button_query {
        let index = match action {
            GuardianMenuAction::BasicPractice => 0,
            GuardianMenuAction::AdvancedPractice => 1,
            GuardianMenuAction::FullHpMana => 2,
            GuardianMenuAction::ElementAllocation => 3,
        };
        let active = match action {
            GuardianMenuAction::BasicPractice => basic_active.0,
            GuardianMenuAction::AdvancedPractice => advanced_active.0,
            GuardianMenuAction::FullHpMana => false,
            GuardianMenuAction::ElementAllocation => window_open, // เปิดค้าง = สี active
        };
        let new_color = if !menu_focused {
            Color::srgb(0.04, 0.04, 0.05)
        } else if active {
            if selected(index, selection.index) { Color::srgb(0.07, 0.12, 0.07) } else { Color::srgb(0.02, 0.05, 0.02) }
        } else if index == selection.index {
            Color::srgb(0.35, 0.35, 0.35)
        } else {
            Color::srgb(0.15, 0.15, 0.15)
        };
        if background.0 != new_color {
            *background = BackgroundColor(new_color);
        }
    }
}

fn selected(a: usize, b: usize) -> bool { a == b }

pub fn guardian_menu_confirm_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    dialog_open: Res<GuardianDialogOpen>,
    selection: Res<GuardianMenuSelection>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    practice_query: Query<Entity, With<PracticeEntity>>,
    mut player_query: Query<(&mut Health, &mut Mana, &mut Transform), With<Player>>,
    mut focus: ResMut<GuardianDialogFocus>,
    fonts: Res<GameFonts>,
    loc: Res<Localization>,
    dialog_query: Query<Entity, With<GuardianDialogUI>>,
    window_query: Query<Entity, With<GuardianAllocWindow>>,
    mut practice: PracticeSettings, // 👈 รวม 4 ตัวมาอยู่ตรงนี้
) {
    if !dialog_open.0 || *focus != GuardianDialogFocus::Menu {
        return;
    }
    
    let gamepad_confirm = any_gp_just_pressed(&gamepads, GamepadButton::South);
    let confirm_pressed = keyboard.just_pressed(KeyCode::Space) || gamepad_confirm;
    if !confirm_pressed {
        return;
    }

    match selection.index {
        0 => {
            // เปลี่ยนจาก basic_practice_active เป็น practice.basic_active
            practice.basic_active.0 = !practice.basic_active.0; 
            if !practice.basic_active.0 {
                for entity in &practice_query {
                    commands.entity(entity).despawn();
                }
                // เปลี่ยนจาก respawn_timer เป็น practice.basic_timer
                practice.basic_timer.0 = Timer::from_seconds(0.1, TimerMode::Once); 
            }
        }
        1 => {
            // เปลี่ยนจาก advanced_practice_active เป็น practice.advanced_active
            practice.advanced_active.0 = !practice.advanced_active.0;
            if !practice.advanced_active.0 {
                for entity in &practice_query {
                    commands.entity(entity).despawn();
                }
                // เปลี่ยนจาก advanced_respawn_timer เป็น practice.advanced_timer
                practice.advanced_timer.0 = Timer::from_seconds(0.1, TimerMode::Once);
            }
        }
        2 => {
            if let Ok((mut health, mut mana, _)) = player_query.single_mut() {
                health.current = health.max;
                mana.current = mana.max;
            }
        }
        3 => {
            if window_query.is_empty() {
                let Ok(dialog_entity) = dialog_query.single() else { return; };
                spawn_guardian_alloc_window(&mut commands, dialog_entity, &fonts, &loc);
                *focus = GuardianDialogFocus::Allocation; 
            } else {
                for entity in &window_query {
                    commands.entity(entity).despawn();
                }
                *focus = GuardianDialogFocus::Menu;
            }
        }
        _ => {}
    }
    
    commands.spawn(AudioPlayer::new(asset_server.load("sounds/ui_confirm.ogg")));
}

pub fn cleanup_guardian_ui_when_player_leave(
    mut commands: Commands,
    player_query: Query<(), With<PlayerInGuardianArea>>,
    dialog_query: Query<Entity, With<GuardianDialogUI>>,
    mut dialog_open: ResMut<GuardianDialogOpen>,
) {
    if !player_query.is_empty() {
        return;
    }
    if dialog_query.is_empty() {
        dialog_open.0 = false;
        return;
    }
    for entity in &dialog_query {
        commands.entity(entity).despawn();
    }
    dialog_open.0 = false;
}

pub fn despawn_hub_only_entities(
    mut commands: Commands,
    hub_query: Query<Entity, With<HubOnly>>,
) {
    for entity in &hub_query {
        commands.entity(entity).despawn();
    }
}

fn element_key(element: Element) -> &'static str {
    match element {
        Element::Water => "water",
        Element::Fire => "fire",
        Element::Wind => "wind",
        Element::Earth => "earth",
        Element::Inw => "inw",
        Element::Neutral => "neutral",
    }
}

fn spawn_alloc_panel(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    loc: &Localization,
) {
    parent
        .spawn((
            GuardianAllocPanel,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                width: Val::Px(320.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.9)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(loc.get("redistribute_elements")),
                TextFont { font: fonts.abc3dz.clone(), font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
            panel.spawn((
                AllocPoolText,
                Text::new(format!("{}: 0", loc.get("neutral_pool"))),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.2)),
            ));
            for element in ALL_ELEMENTS {
                spawn_alloc_row(panel, element, fonts, loc);
            }
            spawn_atk_element_row(panel, fonts, loc);
        });
}

fn spawn_alloc_row(
    parent: &mut ChildSpawnerCommands,
    element: Element,
    fonts: &GameFonts,
    loc: &Localization,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                AllocElementText(element),
                Text::new(format!("{}: 0 ", loc.get(element_key(element)))),
                TextFont { font: fonts.abc3dz.clone(), font_size: 18.0, ..default() },
                TextColor(Color::WHITE),
                Node { width: Val::Px(150.0), ..default() },
            )); 
            spawn_alloc_button(row, element, false, fonts); // [-]
            spawn_alloc_button(row, element, true, fonts);  // [+]
        });
}

fn spawn_alloc_button(
    parent: &mut ChildSpawnerCommands,
    element: Element,
    increase: bool,
    fonts: &GameFonts,
) {
    parent
        .spawn((
            AllocButton { element, increase },
            Node {
                width: Val::Px(36.0),
                height: Val::Px(28.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(if increase { "+" } else { "-" }),
                TextFont { font: fonts.abc3dz.clone(), font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn update_guardian_alloc_ui(
    dialog_query: Query<(), With<GuardianDialogUI>>,
    focus: Res<GuardianDialogFocus>,
    atk_selection: Res<GuardianAllocSelection>,
    point_selection: Res<GuardianAllocPointSelection>,
    loc: Res<Localization>, // เพิ่ม Localization เข้ามา
    player_query: Query<(&ElementMastery, &ElementPointPool, &AtkAndDefElement), With<Player>>,
    mut pool_text: Query<(&mut Text, &mut TextColor), (With<AllocPoolText>, Without<AllocElementText>, Without<AtkElementText>)>,
    mut element_texts: Query<(&AllocElementText, &mut Text, &mut TextColor), (With<AllocElementText>, Without<AllocPoolText>, Without<AtkElementText>)>,
    mut atk_text: Query<(&mut Text, &mut TextColor), (With<AtkElementText>, Without<AllocPoolText>, Without<AllocElementText>)>,
    mut alloc_buttons: Query<(&AllocButton, &mut BackgroundColor), (Without<AtkElementButton>, Without<GuardianMenuButton>)>,
    mut atk_buttons: Query<(&AtkElementButton, &mut BackgroundColor), (Without<AllocButton>, Without<GuardianMenuButton>)>,
) {
    if dialog_query.is_empty() {
        return;
    }
    let Ok((mastery, pool, atk_element)) = player_query.single() else {
        return;
    };
    if ALL_ELEMENTS.is_empty() {
        return;
    }
    
    let count = ALL_ELEMENTS.len();
    let point_element = ALL_ELEMENTS[point_selection.index % count];
    let atk_selected_element = ALL_ELEMENTS[atk_selection.index % count];
    let allocation_focused = *focus == GuardianDialogFocus::Allocation;
    let dim_text = Color::srgb(0.35, 0.35, 0.35);
    let dark_button = Color::srgb(0.04, 0.04, 0.05);
    
    // อัปเดต Neutral Pool
    for (mut text, mut text_color) in &mut pool_text {
        text.set_if_neq(Text::new(format!("{}: {} ", loc.get("neutral_pool"), pool.points)));
        let new_color = if allocation_focused {
            Color::srgb(1.0, 0.85, 0.2)
        } else {
            dim_text
        };
        if text_color.0 != new_color {
            *text_color = TextColor(new_color);
        }
    }
    
    // อัปเดตค่าธาตุแต่ละธาตุ + ไฮไลต์ชื่อธาตุ
    for (marker, mut text, mut text_color) in &mut element_texts {
        let exp = mastery.get(marker.0).map(|p| p.exp).unwrap_or(0);
        let element_name = loc.get(element_key(marker.0));
        text.set_if_neq(Text::new(format!("{}: {} ", element_name, exp)));
        
        let new_color = if !allocation_focused {
            dim_text
        } else if marker.0 == point_element {
            Color::srgb(1.0, 0.85, 0.2)
        } else if marker.0 == atk_selected_element {
            Color::srgb(0.4, 1.0, 0.9)
        } else {
            Color::WHITE
        };
        if text_color.0 != new_color {
            *text_color = TextColor(new_color);
        }
    }
    
    // อัปเดตธาตุโจมตีปัจจุบัน
    for (mut text, mut text_color) in &mut atk_text {
        let element_name = loc.get(element_key(atk_element.0));
        text.set_if_neq(Text::new(format!("{}: {} ", loc.get("attack_element"), element_name)));
        
        let new_color = if allocation_focused {
            Color::srgb(0.4, 1.0, 0.9)
        } else {
            dim_text
        };
        if text_color.0 != new_color {
            *text_color = TextColor(new_color);
        }
    }
    
    // ไฮไลต์ปุ่ม + / - ของธาตุที่เลือกในโหมดบวกลบ
    for (button, mut background) in &mut alloc_buttons {
        let new_color = if !allocation_focused {
            dark_button
        } else if button.element == point_element {
            Color::srgb(0.35, 0.35, 0.65)
        } else {
            Color::srgb(0.2, 0.2, 0.25)
        };
        if background.0 != new_color {
            *background = BackgroundColor(new_color);
        }
    }
    
    // ไฮไลต์ปุ่มเลือกธาตุโจมตี
    for (button, mut background) in &mut atk_buttons {
        let new_color = if !allocation_focused {
            dark_button
        } else {
            let is_selected = button.0 == atk_selected_element;
            let is_current = button.0 == atk_element.0;
            if is_current && is_selected {
                Color::srgb(0.2, 0.75, 0.45)
            } else if is_current {
                Color::srgb(0.15, 0.55, 0.35)
            } else if is_selected {
                Color::srgb(0.35, 0.35, 0.65)
            } else {
                Color::srgb(0.2, 0.2, 0.25)
            }
        };
        if background.0 != new_color {
            *background = BackgroundColor(new_color);
        }
    }
}

fn spawn_atk_element_row(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    loc: &Localization, // เพิ่ม Localization
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                AtkElementText,
                Text::new(format!("{}: -", loc.get("attack_element"))),
                TextFont { font: fonts.abc3dz.clone(), font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.4, 1.0, 0.9)),
            ));
            
            col.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|row| {
                for element in ALL_ELEMENTS {
                    row.spawn((
                        AtkElementButton(element),
                        Node {
                            width: Val::Px(62.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(loc.get(element_key(element))), // ดึงชื่อธาตุจาก lang.json
                            TextFont { font: fonts.abc3dz.clone(), font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });
        });
}

pub fn guardian_dialog_focus_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    dialog_open: Res<GuardianDialogOpen>,
    mut focus: ResMut<GuardianDialogFocus>,
) {
    if !dialog_open.0 {
        return;
    }
    let tab_pressed = keyboard.just_pressed(KeyCode::Tab)
        || any_gp_just_pressed(&gamepads, GamepadButton::Select);
    if tab_pressed {
        *focus = match *focus {
            GuardianDialogFocus::Menu => GuardianDialogFocus::Allocation,
            GuardianDialogFocus::Allocation => GuardianDialogFocus::Menu,
        };
    }
}

pub fn guardian_atk_selection_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    dialog_open: Res<GuardianDialogOpen>,
    focus: Res<GuardianDialogFocus>,
    mut atk_selection: ResMut<GuardianAllocSelection>,
    mut player_query: Query<&mut AtkAndDefElement, With<Player>>,
) {
    if !dialog_open.0 || *focus != GuardianDialogFocus::Allocation {
        return;
    }
    let count = ALL_ELEMENTS.len();
    if count == 0 {
        return;
    }
    let Ok(mut atk_element) = player_query.single_mut() else {
        return;
    };
    let left_pressed = keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::KeyA)
        || any_gp_just_pressed(&gamepads, GamepadButton::DPadLeft);
    let right_pressed = keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::KeyD)
        || any_gp_just_pressed(&gamepads, GamepadButton::DPadRight);
    if left_pressed {
        atk_selection.index = (atk_selection.index + count - 1) % count;
    }
    if right_pressed {
        atk_selection.index = (atk_selection.index + 1) % count;
    }
    if atk_selection.index >= count {
        atk_selection.index = 0;
    }
    let selected_element = ALL_ELEMENTS[atk_selection.index];
    let confirm_pressed = keyboard.just_pressed(KeyCode::Enter)
        || any_gp_just_pressed(&gamepads, GamepadButton::East);
    if confirm_pressed {
        atk_element.0 = selected_element;
    }
}

pub fn guardian_alloc_point_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    gamepads: Query<&Gamepad>,
    dialog_open: Res<GuardianDialogOpen>,
    focus: Res<GuardianDialogFocus>,
    mut point_selection: ResMut<GuardianAllocPointSelection>,
    mut increase_repeat: ResMut<GuardianAllocIncreaseRepeat>,
    mut decrease_repeat: ResMut<GuardianAllocDecreaseRepeat>,
    mut press_state: Local<GuardianAllocPressState>,
    mut player_query: Query<(&mut ElementMastery, &mut ElementPointPool), With<Player>>,
) {
    if !dialog_open.0 || *focus != GuardianDialogFocus::Allocation {
        press_state.increase = false;
        press_state.decrease = false;
        increase_repeat.0.reset();
        decrease_repeat.0.reset();
        return;
    }
    let count = ALL_ELEMENTS.len();
    if count == 0 {
        return;
    }
    let Ok((mut mastery, mut pool)) = player_query.single_mut() else {
        return;
    };
    let mastery = &mut *mastery;
    let pool = &mut *pool;
    
    let up_pressed = keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
        || any_gp_just_pressed(&gamepads, GamepadButton::DPadUp);
    let down_pressed = keyboard.just_pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::KeyS)
        || any_gp_just_pressed(&gamepads, GamepadButton::DPadDown);
    if up_pressed {
        point_selection.index = (point_selection.index + count - 1) % count;
    }
    if down_pressed {
        point_selection.index = (point_selection.index + 1) % count;
    }
    if point_selection.index >= count {
        point_selection.index = 0;
    }
    let selected_element = ALL_ELEMENTS[point_selection.index];
    
    let increase_pressed = keyboard.pressed(KeyCode::KeyU)
        || any_gp_trigger(&gamepads, GamepadButton::RightTrigger);
    let decrease_pressed = keyboard.pressed(KeyCode::KeyE)
        || any_gp_trigger(&gamepads, GamepadButton::LeftTrigger);
        
    if increase_pressed && !press_state.increase {
        increase_element_from_pool(mastery, pool, selected_element);
        increase_repeat.0.reset();
    } else if increase_pressed {
        increase_repeat.0.tick(time.delta());
        if increase_repeat.0.just_finished() {
            increase_element_from_pool(mastery, pool, selected_element);
        }
    } else {
        increase_repeat.0.reset();
    }
    
    if decrease_pressed && !press_state.decrease {
        decrease_element_to_pool(mastery, pool, selected_element);
        decrease_repeat.0.reset();
    } else if decrease_pressed {
        decrease_repeat.0.tick(time.delta());
        if decrease_repeat.0.just_finished() {
            decrease_element_to_pool(mastery, pool, selected_element);
        }
    } else {
        decrease_repeat.0.reset();
    }
    
    press_state.increase = increase_pressed;
    press_state.decrease = decrease_pressed;
}

pub fn update_guardian_focus_visuals(
    dialog_open: Res<GuardianDialogOpen>,
    focus: Res<GuardianDialogFocus>,
    mut menu_panel_query: Query<&mut BackgroundColor, (With<GuardianMenuPanel>, Without<GuardianAllocPanel>)>,
    mut alloc_panel_query: Query<&mut BackgroundColor, (With<GuardianAllocPanel>, Without<GuardianMenuPanel>)>,
) {
    if !dialog_open.0 {
        return;
    }
    let menu_focused = *focus == GuardianDialogFocus::Menu;
    let bright_menu = Color::srgba(0.16, 0.16, 0.22, 0.9);
    let bright_alloc = Color::srgba(0.10, 0.10, 0.16, 0.9);
    let dark_panel = Color::srgba(0.03, 0.03, 0.05, 0.9);
    for mut background in &mut menu_panel_query {
        let new_color = if menu_focused { bright_menu } else { dark_panel };
        if background.0 != new_color {
            *background = BackgroundColor(new_color);
        }
    }
    for mut background in &mut alloc_panel_query {
        let new_color = if menu_focused { dark_panel } else { bright_alloc };
        if background.0 != new_color {
            *background = BackgroundColor(new_color);
        }
    }
}
fn guardian_control_key(key: GuardianControlKey) -> &'static str {
    match key {
        GuardianControlKey::MenuTitle => "guardian_controls_menu_title",
        GuardianControlKey::MenuUp => "guardian_controls_menu_up",
        GuardianControlKey::MenuDown => "guardian_controls_menu_down",
        GuardianControlKey::MenuConfirm => "guardian_controls_menu_confirm",
        GuardianControlKey::MenuClose => "guardian_controls_menu_close",

        GuardianControlKey::AllocTitle => "guardian_controls_alloc_title",
        GuardianControlKey::AllocUp => "guardian_controls_alloc_up",
        GuardianControlKey::AllocDown => "guardian_controls_alloc_down",
        GuardianControlKey::AllocIncrease => "guardian_controls_alloc_increase",
        GuardianControlKey::AllocDecrease => "guardian_controls_alloc_decrease",
        GuardianControlKey::AllocAtkLeft => "guardian_controls_alloc_atk_left",
        GuardianControlKey::AllocAtkRight => "guardian_controls_alloc_atk_right",
        GuardianControlKey::AllocConfirm => "guardian_controls_alloc_confirm",
        GuardianControlKey::AllocClose => "guardian_controls_alloc_close",
    }
}
pub fn update_guardian_controls_language_and_focus(
    dialog_query: Query<(), With<GuardianDialogUI>>,
    focus: Res<GuardianDialogFocus>,
    loc: Res<Localization>,
    mut label_query: Query<(&GuardianControlLabel, &mut Text, &mut TextColor)>,
) {
    if dialog_query.is_empty() {
        return;
    }

    let menu_focused = *focus == GuardianDialogFocus::Menu;

    for (label, mut text, mut text_color) in &mut label_query {
        let key = guardian_control_key(label.0);

        // อัปเดตภาษาตาม Localization ปัจจุบัน
        text.set_if_neq(Text::new(loc.get(key)));

        // เช็คว่าข้อความนี้อยู่ฝั่ง Menu หรือ Allocation
        let is_menu_key = matches!(
            label.0,
            GuardianControlKey::MenuTitle
                | GuardianControlKey::MenuUp
                | GuardianControlKey::MenuDown
                | GuardianControlKey::MenuConfirm
                | GuardianControlKey::MenuClose
        );

        let is_title = matches!(
            label.0,
            GuardianControlKey::MenuTitle | GuardianControlKey::AllocTitle
        );

        let active = menu_focused == is_menu_key;

        let new_color = if active {
            if is_title {
                Color::srgb(1.0, 0.85, 0.2)
            } else {
                Color::WHITE
            }
        } else {
            Color::srgb(0.45, 0.45, 0.45)
        };

        if text_color.0 != new_color {
            *text_color = TextColor(new_color);
        }
    }
}
fn spawn_guardian_controls_panel(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    loc: &Localization,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.9)),
        ))
        .with_children(|col| {
            spawn_control_label(col, GuardianControlKey::MenuTitle, fonts, loc, 16.0, Color::srgb(1.0, 0.85, 0.2));
            spawn_control_label(col, GuardianControlKey::MenuUp, fonts, loc, 14.0, Color::WHITE);
            spawn_control_label(col, GuardianControlKey::MenuDown, fonts, loc, 14.0, Color::WHITE);
            spawn_control_label(col, GuardianControlKey::MenuConfirm, fonts, loc, 14.0, Color::WHITE);
            spawn_control_label(col, GuardianControlKey::MenuClose, fonts, loc, 14.0, Color::WHITE);
        });
}
fn spawn_alloc_controls_panel(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    loc: &Localization,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        })
        .with_children(|col| {
            spawn_control_label(col, GuardianControlKey::AllocTitle, fonts, loc, 16.0, Color::srgb(1.0, 0.85, 0.2));
            spawn_control_label(col, GuardianControlKey::AllocUp, fonts, loc, 14.0, Color::WHITE);
            spawn_control_label(col, GuardianControlKey::AllocDown, fonts, loc, 14.0, Color::WHITE);
            spawn_control_label(col, GuardianControlKey::AllocIncrease, fonts, loc, 14.0, Color::WHITE);
            spawn_control_label(col, GuardianControlKey::AllocDecrease, fonts, loc, 14.0, Color::WHITE);
            spawn_control_label(col, GuardianControlKey::AllocAtkLeft, fonts, loc, 14.0, Color::WHITE);
            spawn_control_label(col, GuardianControlKey::AllocAtkRight, fonts, loc, 14.0, Color::WHITE);
            spawn_control_label(col, GuardianControlKey::AllocConfirm, fonts, loc, 14.0, Color::WHITE);
            spawn_control_label(col, GuardianControlKey::AllocClose, fonts, loc, 14.0, Color::WHITE);
        });
}
fn spawn_guardian_alloc_window(
    commands: &mut Commands,
    dialog_entity: Entity,
    fonts: &GameFonts,
    loc: &Localization,
) {
    commands.entity(dialog_entity).with_children(|root| {
        root.spawn((
            GuardianAllocWindow,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|center| {
            center
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(14.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.04, 0.04, 0.08, 0.96)),
                ))
                .with_children(|window| {
                    window.spawn((
                        Text::new(loc.get("alloc_window_title")),
                        TextFont { font: fonts.abc3dz.clone(), font_size: 24.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    ));
                    window
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(24.0),
                            align_items: AlignItems::FlexStart,
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_alloc_panel(row, fonts, loc);          // แผงจัดสรร
                            spawn_element_bonus_table(row, fonts, loc);  // ตาราง Elemental/10
                            spawn_alloc_controls_panel(row, fonts, loc); // 👈 ปุ่มควบคุมฝั่งจัดสรร
                        });
                });
        });
    });
}

fn spawn_control_label(
    parent: &mut ChildSpawnerCommands,
    key: GuardianControlKey,
    fonts: &GameFonts,
    loc: &Localization,
    font_size: f32,
    color: Color,
) {
    parent.spawn((
        GuardianControlLabel(key),
        Text::new(loc.get(guardian_control_key(key))),
        TextFont {
            font: fonts.abc3dz.clone(),
            font_size,
            ..default()
        },
        TextColor(color),
    ));
}
//Gamepad
fn any_gp_just_pressed(gamepads: &Query<&Gamepad>, button: GamepadButton) -> bool {
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(button) {
            return true;
        }
    }
    false
}

fn any_gp_trigger(gamepads: &Query<&Gamepad>, button: GamepadButton) -> bool {
    for gamepad in gamepads.iter() {
        if gamepad.pressed(button) {
            return true;
        }
    }
    false
}

//alloc window
const ELEMENT_BONUS_TABLE: [(&str, [f32; 6]); 5] = [
    //           Atk    Def    CritRate CritDmg  HP     MP
    ("water",  [0.2,   0.0,   0.0,     0.0,     0.0,   10.0]),
    ("fire",   [0.8,   0.0,   0.0,     0.02,    0.0,   0.0]),
    ("wind",   [0.0,   0.0,   0.005,   0.01,    0.0,   0.0]),
    ("earth",  [0.0,   0.6,   0.0,     0.0,     8.0,   0.0]),
    ("inw",    [0.15,  0.15,  0.001,   0.005,   2.0,   2.0]),
];

fn format_element_bonus(column: usize, value: f32) -> String {
    if value == 0.0 {
        return "-".to_string();
    }
    match column {
        2 | 3 => format!("{:.1}%", value * 100.0), // Crit Rate / Crit Dmg แสดงเป็น %
        _ => {
            if value.fract() == 0.0 { format!("{:.0}", value) } else { format!("{}", value) }
        }
    }
}

fn spawn_element_bonus_table(
    parent: &mut ChildSpawnerCommands,
    fonts: &GameFonts,
    loc: &Localization,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.9)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(loc.get("element_bonus_title")),
                TextFont { font: fonts.abc3dz.clone(), font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
            panel
                .spawn(Node {
                    display: Display::Grid,
                    grid_template_columns: vec![
                        GridTrack::px(110.0), // Elemental/10
                        GridTrack::px(70.0),  // Atk
                        GridTrack::px(70.0),  // Def
                        GridTrack::px(90.0),  // Crit Rate
                        GridTrack::px(90.0),  // Crit Dmg
                        GridTrack::px(60.0),  // HP
                        GridTrack::px(60.0),  // MP
                    ],
                    row_gap: Val::Px(6.0),
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|grid| {
                    let headers = [
                        loc.get("elemental_per_10"),
                        loc.get("atk_label"),
                        loc.get("def_label"),
                        loc.get("crit_rate_label"),
                        loc.get("crit_dmg_label"),
                        loc.get("hp_label"),
                        loc.get("mp_label"),
                    ];
                    for header in headers {
                        grid.spawn((
                            Text::new(header),
                            TextFont { font: fonts.abc3dz.clone(), font_size: 16.0, ..default() },
                            TextColor(Color::srgb(1.0, 0.85, 0.2)),
                        ));
                    }
                    for (element_key, bonuses) in ELEMENT_BONUS_TABLE {
                        grid.spawn((
                            Text::new(loc.get(element_key)),
                            TextFont { font: fonts.abc3dz.clone(), font_size: 16.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                        for (column, value) in bonuses.iter().enumerate() {
                            let color = if *value > 0.0 {
                                Color::srgb(0.25, 1.0, 0.35)  // มีโบนัส = เขียว
                            } else {
                                Color::srgb(0.45, 0.45, 0.50) // ไม่มี = เทาจาง
                            };
                            grid.spawn((
                                Text::new(format_element_bonus(column, *value)),
                                TextFont { font: fonts.abc3dz.clone(), font_size: 16.0, ..default() },
                                TextColor(color),
                            ));
                        }
                    }
                });
        });
}