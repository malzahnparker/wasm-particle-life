use bevy::{
    color::palettes::css,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::mouse::MouseWheel,
    prelude::*,
    window::WindowResolution,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use rand::{seq::SliceRandom, Rng};
use std::collections::HashMap;

#[derive(Resource)]
struct ColorCount {
    count: usize,
}

#[derive(Component)]
struct Particle {
    color_id: usize,
}

#[derive(Resource)]
struct ParticleCount {
    count: usize,
}

#[derive(Resource)]
struct ParticleSystem {
    colors: Vec<Color>,
    behavior_matrix: Vec<Vec<f32>>,
    speed: f32,
    beta: f32,
    gamma: f32,
    attraction_radius: f32,
}

impl ParticleSystem {
    fn new() -> Self {
        let all_colors = vec![
            // Reds
            css::RED,
            css::CRIMSON,
            css::DARK_RED,
            css::FIRE_BRICK,
            css::INDIAN_RED,
            css::LIGHT_CORAL,
            css::SALMON,
            css::DARK_SALMON,
            css::LIGHT_SALMON,
            css::ORANGE_RED,
            // Oranges
            css::ORANGE_RED,
            css::TOMATO,
            css::DARK_ORANGE,
            css::ORANGE,
            css::GOLD,
            css::DARK_GOLDENROD,
            css::GOLDENROD,
            css::PALE_GOLDENROD,
            css::PEACHPUFF,
            css::NAVAJO_WHITE,
            // Yellows
            css::YELLOW,
            css::LIGHT_YELLOW,
            css::LEMON_CHIFFON,
            css::LIGHT_GOLDENROD_YELLOW,
            css::PAPAYA_WHIP,
            css::MOCCASIN,
            css::KHAKI,
            css::DARK_KHAKI,
            css::YELLOW_GREEN,
            css::OLIVE,
            // Greens
            css::LIME,
            css::LIMEGREEN,
            css::LAWN_GREEN,
            css::CHARTREUSE,
            css::GREEN_YELLOW,
            css::SPRING_GREEN,
            css::MEDIUM_SPRING_GREEN,
            css::LIGHT_GREEN,
            css::PALE_GREEN,
            css::DARK_SEA_GREEN,
            css::MEDIUM_SEA_GREEN,
            css::SEA_GREEN,
            css::FOREST_GREEN,
            css::GREEN,
            css::DARK_GREEN,
            // Cyans
            css::MEDIUM_AQUAMARINE,
            css::AQUA,
            css::DARK_CYAN,
            css::LIGHT_CYAN,
            css::PALE_TURQUOISE,
            css::AQUAMARINE,
            css::TURQUOISE,
            css::MEDIUM_TURQUOISE,
            css::DARK_TURQUOISE,
            css::LIGHT_SEA_GREEN,
            // Blues
            css::DEEP_SKY_BLUE,
            css::LIGHT_BLUE,
            css::SKY_BLUE,
            css::LIGHT_SKY_BLUE,
            css::STEEL_BLUE,
            css::ALICE_BLUE,
            css::DODGER_BLUE,
            css::ROYAL_BLUE,
            css::BLUE,
            css::MEDIUM_BLUE,
            css::DARK_BLUE,
            css::NAVY,
            css::MIDNIGHT_BLUE,
            css::CORNFLOWER_BLUE,
            css::SLATE_BLUE,
            // Purples
            css::MEDIUM_SLATE_BLUE,
            css::DARK_SLATE_BLUE,
            css::LAVENDER,
            css::THISTLE,
            css::PLUM,
            css::VIOLET,
            css::ORCHID,
            css::MAGENTA,
            css::MEDIUM_ORCHID,
            css::MEDIUM_PURPLE,
            css::BLUE_VIOLET,
            css::DARK_VIOLET,
            css::DARK_ORCHID,
            css::DARK_MAGENTA,
            css::PURPLE,
            // Pinks
            css::INDIGO,
            css::MEDIUM_VIOLET_RED,
            css::PALE_VIOLETRED,
            css::DEEP_PINK,
            css::HOT_PINK,
            css::LIGHT_PINK,
            css::PINK,
            css::ANTIQUE_WHITE,
            css::BEIGE,
            css::BISQUE,
            // Browns
            css::SADDLE_BROWN,
            css::SIENNA,
            css::CHOCOLATE,
            css::PERU,
            css::SANDY_BROWN,
            css::BURLYWOOD,
            css::TAN,
            css::ROSY_BROWN,
            css::MOCCASIN,
            css::NAVAJO_WHITE,
            // Grays and others
            css::MAROON,
            css::BROWN,
            css::DARK_OLIVEGREEN,
            css::OLIVE_DRAB,
            css::DARK_CYAN,
            css::TEAL,
            css::DARK_SLATE_GRAY,
            css::SLATE_GRAY,
            css::LIGHT_SLATE_GRAY,
            css::DIM_GRAY,
        ];

        let mut rng = rand::rng();
        let num_colors = rng.random_range(20..=100);
        let mut colors_indices: Vec<usize> = (0..all_colors.len()).collect();
        colors_indices.shuffle(&mut rng);
        let colors: Vec<Color> = colors_indices[0..num_colors]
            .iter()
            .map(|&i| Color::from(all_colors[i]))
            .collect();

        let n = colors.len();

        let behavior_matrix = (0..n)
            .map(|_| (0..n).map(|_| rng.random_range(-1.0..=1.0)).collect())
            .collect();

        let beta = rng.random_range(0.1..=0.4);
        let gamma = rng.random_range(0.6..=0.9);
        let attraction_radius = 100.0;

        ParticleSystem {
            colors,
            behavior_matrix,
            speed: BASE_SPEED,
            beta,
            gamma,
            attraction_radius,
        }
    }

    fn get_behavior(&self, from_color: usize, to_color: usize) -> f32 {
        self.behavior_matrix[from_color][to_color]
    }
    fn regenerate_matrix(&mut self) {
        let mut rng = rand::rng();
        let n = self.colors.len();
        self.behavior_matrix = (0..n)
            .map(|_| (0..n).map(|_| rng.random_range(-1.0..=1.0)).collect())
            .collect();
    }
    fn regenerate_constants(&mut self) {
        let mut rng = rand::rng();
        self.beta = rng.random_range(0.1..=0.4);
        self.gamma = rng.random_range(0.6..=0.9);
        self.attraction_radius = 100.0;
    }
}

const WINDOW_WIDTH: f32 = 1920.0;
const WINDOW_HEIGHT: f32 = 1080.0;
const PARTICLE_SIZE: f32 = 5.0;
const NUM_PARTICLES: usize = 5000;
const BASE_SPEED: f32 = 1600.0;
const CAMERA_SPEED: f32 = 500.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Particle Life".to_string(),
                    resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            FrameTimeDiagnosticsPlugin,
            EguiPlugin,
            LogDiagnosticsPlugin::default(),
        ))
        .insert_resource(ParticleSystem::new())
        .insert_resource(ParticleCount { count: 5000 }) // Initial particle count
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_particles,
                move_camera,
                handle_matrix_regeneration,
                adjust_speed,
                ui_system,
            ),
        )
        .run();
}
fn setup(
    mut commands: Commands,
    particle_system: Res<ParticleSystem>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d::default());

    dbg!(&particle_system.behavior_matrix);

    let mut rng = rand::rng();

    for _ in 0..NUM_PARTICLES {
        let x = rng.random_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
        let y = rng.random_range(-WINDOW_HEIGHT / 2.0..WINDOW_HEIGHT / 2.0);

        let color_id = rng.random_range(0..particle_system.colors.len());

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(PARTICLE_SIZE / 2.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from(particle_system.colors[color_id]))),
            Transform::from_xyz(x, y, 0.0),
            Particle { color_id },
        ));
    }
}

fn update_particles(
    particle_system: Res<ParticleSystem>,
    time: Res<Time>,
    mut particle_query: Query<(&mut Transform, &Particle)>,
) {
    dbg!(particle_query.iter().count());
    let dt = time.delta_secs() * particle_system.speed;
    let beta = particle_system.beta;
    let gamma = particle_system.gamma;
    let gamma_beta_diff = gamma - beta;
    let one_minus_gamma = 1.0 - gamma;
    let attraction_radius = particle_system.attraction_radius;

    // Create a spatial grid for faster neighbor lookups
    let cell_size = attraction_radius;
    let mut grid: HashMap<(i32, i32), Vec<(Vec3, usize)>> = HashMap::new();

    // Populate the grid
    for (transform, particle) in particle_query.iter() {
        let pos = transform.translation;
        let cell_x = (pos.x / cell_size).floor() as i32;
        let cell_y = (pos.y / cell_size).floor() as i32;
        grid.entry((cell_x, cell_y))
            .or_default()
            .push((pos, particle.color_id));
    }

    // Update particles
    for (mut transform, particle) in &mut particle_query {
        let pos = transform.translation;
        let cell_x = (pos.x / cell_size).floor() as i32;
        let cell_y = (pos.y / cell_size).floor() as i32;

        let mut force = Vec2::ZERO;
        let mut count = 0.0;

        // Check neighboring cells
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(cell_particles) = grid.get(&(cell_x + dx, cell_y + dy)) {
                    for &(other_pos, other_color_id) in cell_particles {
                        if pos == other_pos {
                            continue;
                        }

                        let to_other = other_pos - pos;
                        let distance = to_other.length() / attraction_radius;

                        if distance < 1.0 {
                            let direction = to_other.truncate().normalize();
                            let behavior =
                                particle_system.get_behavior(particle.color_id, other_color_id);

                            let force_magnitude = if distance < beta {
                                -1.0 + (distance / beta)
                            } else if distance < gamma {
                                behavior * ((distance - beta) / gamma_beta_diff)
                            } else {
                                behavior * ((1.0 - distance) / one_minus_gamma)
                            };

                            force += direction * force_magnitude;
                            count += 1.0;
                        }
                    }
                }
            }
        }

        if count > 0.0 {
            force /= count;
        }

        transform.translation += force.extend(0.0) * dt;
    }
}

fn move_camera(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    particle_system: Res<ParticleSystem>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    let mut camera_transform = query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        camera_transform.scale /= 1.1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        camera_transform.scale *= 1.1;
    }

    if direction != Vec3::ZERO {
        direction = direction.normalize();
        let scale = camera_transform.scale;
        camera_transform.translation += direction * CAMERA_SPEED * time.delta_secs() * scale;
    }
}

fn handle_matrix_regeneration(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut particle_system: ResMut<ParticleSystem>,
    particles: Query<Entity, With<Particle>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        // Clear all existing particles
        for entity in &particles {
            commands.entity(entity).despawn();
        }

        // Generate new colors and matrix
        let mut rng = rand::rng();
        let all_colors = vec![
            // Reds
            css::RED,
            css::CRIMSON,
            css::DARK_RED,
            css::FIRE_BRICK,
            css::INDIAN_RED,
            css::LIGHT_CORAL,
            css::SALMON,
            css::DARK_SALMON,
            css::LIGHT_SALMON,
            css::ORANGE_RED,
            // Oranges
            css::ORANGE_RED,
            css::TOMATO,
            css::DARK_ORANGE,
            css::ORANGE,
            css::GOLD,
            css::DARK_GOLDENROD,
            css::GOLDENROD,
            css::PALE_GOLDENROD,
            css::PEACHPUFF,
            css::NAVAJO_WHITE,
            // Yellows
            css::YELLOW,
            css::LIGHT_YELLOW,
            css::LEMON_CHIFFON,
            css::LIGHT_GOLDENROD_YELLOW,
            css::PAPAYA_WHIP,
            css::MOCCASIN,
            css::KHAKI,
            css::DARK_KHAKI,
            css::YELLOW_GREEN,
            css::OLIVE,
            // Greens
            css::LIME,
            css::LIMEGREEN,
            css::LAWN_GREEN,
            css::CHARTREUSE,
            css::GREEN_YELLOW,
            css::SPRING_GREEN,
            css::MEDIUM_SPRING_GREEN,
            css::LIGHT_GREEN,
            css::PALE_GREEN,
            css::DARK_SEA_GREEN,
            css::MEDIUM_SEA_GREEN,
            css::SEA_GREEN,
            css::FOREST_GREEN,
            css::GREEN,
            css::DARK_GREEN,
            // Cyans
            css::MEDIUM_AQUAMARINE,
            css::AQUA,
            css::DARK_CYAN,
            css::LIGHT_CYAN,
            css::PALE_TURQUOISE,
            css::AQUAMARINE,
            css::TURQUOISE,
            css::MEDIUM_TURQUOISE,
            css::DARK_TURQUOISE,
            css::LIGHT_SEA_GREEN,
            // Blues
            css::DEEP_SKY_BLUE,
            css::LIGHT_BLUE,
            css::SKY_BLUE,
            css::LIGHT_SKY_BLUE,
            css::STEEL_BLUE,
            css::ALICE_BLUE,
            css::DODGER_BLUE,
            css::ROYAL_BLUE,
            css::BLUE,
            css::MEDIUM_BLUE,
            css::DARK_BLUE,
            css::NAVY,
            css::MIDNIGHT_BLUE,
            css::CORNFLOWER_BLUE,
            css::SLATE_BLUE,
            // Purples
            css::MEDIUM_SLATE_BLUE,
            css::DARK_SLATE_BLUE,
            css::LAVENDER,
            css::THISTLE,
            css::PLUM,
            css::VIOLET,
            css::ORCHID,
            css::MAGENTA,
            css::MEDIUM_ORCHID,
            css::MEDIUM_PURPLE,
            css::BLUE_VIOLET,
            css::DARK_VIOLET,
            css::DARK_ORCHID,
            css::DARK_MAGENTA,
            css::PURPLE,
            // Pinks
            css::INDIGO,
            css::MEDIUM_VIOLET_RED,
            css::PALE_VIOLETRED,
            css::DEEP_PINK,
            css::HOT_PINK,
            css::LIGHT_PINK,
            css::PINK,
            css::ANTIQUE_WHITE,
            css::BEIGE,
            css::BISQUE,
            // Browns
            css::SADDLE_BROWN,
            css::SIENNA,
            css::CHOCOLATE,
            css::PERU,
            css::SANDY_BROWN,
            css::BURLYWOOD,
            css::TAN,
            css::ROSY_BROWN,
            css::MOCCASIN,
            css::NAVAJO_WHITE,
            // Grays and others
            css::MAROON,
            css::BROWN,
            css::DARK_OLIVEGREEN,
            css::OLIVE_DRAB,
            css::DARK_CYAN,
            css::TEAL,
            css::DARK_SLATE_GRAY,
            css::SLATE_GRAY,
            css::LIGHT_SLATE_GRAY,
            css::DIM_GRAY,
        ];

        let num_colors = rng.random_range(20..=100);
        let mut colors_indices: Vec<usize> = (0..all_colors.len()).collect();
        colors_indices.shuffle(&mut rng);
        let colors: Vec<Color> = colors_indices[0..num_colors]
            .iter()
            .map(|&i| Color::from(all_colors[i]))
            .collect();

        // Update ParticleSystem
        particle_system.colors = colors;
        particle_system.regenerate_matrix();
        particle_system.regenerate_constants();

        // Spawn new particles
        for _ in 0..NUM_PARTICLES {
            let x = rng.random_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
            let y = rng.random_range(-WINDOW_HEIGHT / 2.0..WINDOW_HEIGHT / 2.0);
            let color_id = rng.random_range(0..particle_system.colors.len());

            commands.spawn((
                Mesh2d(meshes.add(Circle::new(PARTICLE_SIZE / 2.0))),
                MeshMaterial2d(
                    materials.add(ColorMaterial::from(particle_system.colors[color_id])),
                ),
                Transform::from_xyz(x, y, 0.0),
                Particle { color_id },
            ));
        }
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        particle_system.regenerate_matrix();
    }
    if keyboard.just_pressed(KeyCode::KeyT) {
        particle_system.regenerate_constants();
    }
}

fn adjust_speed(keyboard: Res<ButtonInput<KeyCode>>, mut particle_system: ResMut<ParticleSystem>) {
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        particle_system.speed *= 2.0;
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
        particle_system.speed /= 2.0;
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut particle_system: ResMut<ParticleSystem>,
    mut particle_count: ResMut<ParticleCount>,
    diagnostics: Res<DiagnosticsStore>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    particles: Query<Entity, With<Particle>>,
) {
    egui::Window::new("Particle Life Controls")
        .default_pos([10.0, 10.0])
        .show(contexts.ctx_mut(), |ui| {
            // FPS Display
            if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(fps_value) = fps.smoothed() {
                    ui.label(format!("FPS: {:.1}", fps_value));
                }
            }

            ui.add_space(10.0);
            ui.heading("Simulation Parameters");

            // Particle count control
            let mut count = particle_count.count as i32;
            ui.horizontal(|ui| {
                ui.label("Particle Count:");
                if ui
                    .add(egui::Slider::new(&mut count, 100..=20000).text("count"))
                    .changed()
                {
                    particle_count.count = count as usize;
                    // Clear existing particles
                    for entity in &particles {
                        commands.entity(entity).despawn();
                    }
                    // Spawn new particles
                    let mut rng = rand::thread_rng();
                    for _ in 0..particle_count.count {
                        let x = rng.gen_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
                        let y = rng.gen_range(-WINDOW_HEIGHT / 2.0..WINDOW_HEIGHT / 2.0);
                        let color_id = rng.gen_range(0..particle_system.colors.len());
                        commands.spawn((
                            Mesh2d(meshes.add(Circle::new(PARTICLE_SIZE / 2.0))),
                            MeshMaterial2d(
                                materials
                                    .add(ColorMaterial::from(particle_system.colors[color_id])),
                            ),
                            Transform::from_xyz(x, y, 0.0),
                            Particle { color_id },
                        ));
                    }
                }
            });

            // Speed control
            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(egui::Slider::new(&mut particle_system.speed, 0.0..=3200.0));
            });

            // Beta control
            ui.horizontal(|ui| {
                ui.label("Beta:");
                ui.add(egui::Slider::new(&mut particle_system.beta, 0.0..=1.0));
            });

            // Gamma control
            ui.horizontal(|ui| {
                ui.label("Gamma:");
                ui.add(egui::Slider::new(&mut particle_system.gamma, 0.0..=1.0));
            });

            // Attraction radius control
            ui.horizontal(|ui| {
                ui.label("Attraction Radius:");
                ui.add(egui::Slider::new(
                    &mut particle_system.attraction_radius,
                    10.0..=200.0,
                ));
            });

            // Color count control
            let mut color_count = particle_system.colors.len() as i32;
            ui.horizontal(|ui| {
                ui.label("Color Count:");
                if ui
                    .add(egui::Slider::new(&mut color_count, 1..=100).text("colors"))
                    .changed()
                {
                    // Clear existing particles
                    for entity in &particles {
                        commands.entity(entity).despawn();
                    }
                    // Update ParticleSystem with new color count
                    let mut rng = rand::thread_rng();
                    let all_colors = vec![
                        // Reds
                        css::RED,
                        css::CRIMSON,
                        css::DARK_RED,
                        css::FIRE_BRICK,
                        css::INDIAN_RED,
                        css::LIGHT_CORAL,
                        css::SALMON,
                        css::DARK_SALMON,
                        css::LIGHT_SALMON,
                        css::ORANGE_RED,
                        // Oranges
                        css::ORANGE_RED,
                        css::TOMATO,
                        css::DARK_ORANGE,
                        css::ORANGE,
                        css::GOLD,
                        css::DARK_GOLDENROD,
                        css::GOLDENROD,
                        css::PALE_GOLDENROD,
                        css::PEACHPUFF,
                        css::NAVAJO_WHITE,
                        // Yellows
                        css::YELLOW,
                        css::LIGHT_YELLOW,
                        css::LEMON_CHIFFON,
                        css::LIGHT_GOLDENROD_YELLOW,
                        css::PAPAYA_WHIP,
                        css::MOCCASIN,
                        css::KHAKI,
                        css::DARK_KHAKI,
                        css::YELLOW_GREEN,
                        css::OLIVE,
                        // Greens
                        css::LIME,
                        css::LIMEGREEN,
                        css::LAWN_GREEN,
                        css::CHARTREUSE,
                        css::GREEN_YELLOW,
                        css::SPRING_GREEN,
                        css::MEDIUM_SPRING_GREEN,
                        css::LIGHT_GREEN,
                        css::PALE_GREEN,
                        css::DARK_SEA_GREEN,
                        css::MEDIUM_SEA_GREEN,
                        css::SEA_GREEN,
                        css::FOREST_GREEN,
                        css::GREEN,
                        css::DARK_GREEN,
                        // Cyans
                        css::MEDIUM_AQUAMARINE,
                        css::AQUA,
                        css::DARK_CYAN,
                        css::LIGHT_CYAN,
                        css::PALE_TURQUOISE,
                        css::AQUAMARINE,
                        css::TURQUOISE,
                        css::MEDIUM_TURQUOISE,
                        css::DARK_TURQUOISE,
                        css::LIGHT_SEA_GREEN,
                        // Blues
                        css::DEEP_SKY_BLUE,
                        css::LIGHT_BLUE,
                        css::SKY_BLUE,
                        css::LIGHT_SKY_BLUE,
                        css::STEEL_BLUE,
                        css::ALICE_BLUE,
                        css::DODGER_BLUE,
                        css::ROYAL_BLUE,
                        css::BLUE,
                        css::MEDIUM_BLUE,
                        css::DARK_BLUE,
                        css::NAVY,
                        css::MIDNIGHT_BLUE,
                        css::CORNFLOWER_BLUE,
                        css::SLATE_BLUE,
                        // Purples
                        css::MEDIUM_SLATE_BLUE,
                        css::DARK_SLATE_BLUE,
                        css::LAVENDER,
                        css::THISTLE,
                        css::PLUM,
                        css::VIOLET,
                        css::ORCHID,
                        css::MAGENTA,
                        css::MEDIUM_ORCHID,
                        css::MEDIUM_PURPLE,
                        css::BLUE_VIOLET,
                        css::DARK_VIOLET,
                        css::DARK_ORCHID,
                        css::DARK_MAGENTA,
                        css::PURPLE,
                        // Pinks
                        css::INDIGO,
                        css::MEDIUM_VIOLET_RED,
                        css::PALE_VIOLETRED,
                        css::DEEP_PINK,
                        css::HOT_PINK,
                        css::LIGHT_PINK,
                        css::PINK,
                        css::ANTIQUE_WHITE,
                        css::BEIGE,
                        css::BISQUE,
                        // Browns
                        css::SADDLE_BROWN,
                        css::SIENNA,
                        css::CHOCOLATE,
                        css::PERU,
                        css::SANDY_BROWN,
                        css::BURLYWOOD,
                        css::TAN,
                        css::ROSY_BROWN,
                        css::MOCCASIN,
                        css::NAVAJO_WHITE,
                        // Grays and others
                        css::MAROON,
                        css::BROWN,
                        css::DARK_OLIVEGREEN,
                        css::OLIVE_DRAB,
                        css::DARK_CYAN,
                        css::TEAL,
                        css::DARK_SLATE_GRAY,
                        css::SLATE_GRAY,
                        css::LIGHT_SLATE_GRAY,
                        css::DIM_GRAY,
                    ];

                    let mut colors_indices: Vec<usize> = (0..all_colors.len()).collect();
                    colors_indices.shuffle(&mut rng);
                    let colors: Vec<Color> = colors_indices[0..color_count as usize]
                        .iter()
                        .map(|&i| Color::from(all_colors[i]))
                        .collect();

                    particle_system.colors = colors;
                    particle_system.regenerate_matrix();
                    particle_system.regenerate_constants();

                    // Spawn new particles
                    for _ in 0..particle_count.count {
                        let x = rng.gen_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
                        let y = rng.gen_range(-WINDOW_HEIGHT / 2.0..WINDOW_HEIGHT / 2.0);
                        let color_id = rng.gen_range(0..particle_system.colors.len());
                        commands.spawn((
                            Mesh2d(meshes.add(Circle::new(PARTICLE_SIZE / 2.0))),
                            MeshMaterial2d(
                                materials
                                    .add(ColorMaterial::from(particle_system.colors[color_id])),
                            ),
                            Transform::from_xyz(x, y, 0.0),
                            Particle { color_id },
                        ));
                    }
                }
            });

            // Matrix regeneration controls
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("Regenerate Matrix").clicked() {
                    particle_system.regenerate_matrix();
                }
                if ui.button("Regenerate Constants").clicked() {
                    particle_system.regenerate_constants();
                }
                if ui.button("Reset Simulation").clicked() {
                    // Clear existing particles
                    for entity in &particles {
                        commands.entity(entity).despawn();
                    }
                    // Generate new colors and matrix
                    *particle_system = ParticleSystem::new();
                    // Spawn new particles
                    let mut rng = rand::thread_rng();
                    for _ in 0..particle_count.count {
                        let x = rng.gen_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
                        let y = rng.gen_range(-WINDOW_HEIGHT / 2.0..WINDOW_HEIGHT / 2.0);
                        let color_id = rng.gen_range(0..particle_system.colors.len());
                        commands.spawn((
                            Mesh2d(meshes.add(Circle::new(PARTICLE_SIZE / 2.0))),
                            MeshMaterial2d(
                                materials
                                    .add(ColorMaterial::from(particle_system.colors[color_id])),
                            ),
                            Transform::from_xyz(x, y, 0.0),
                            Particle { color_id },
                        ));
                    }
                }
            });
        });

    // Matrix visualization and editing window
    egui::Window::new("Behavior Matrix")
        .default_pos([WINDOW_WIDTH - 300.0, 10.0])
        .default_size([280.0, 300.0])
        .show(contexts.ctx_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let size = particle_system.colors.len();
                egui::Grid::new("behavior_matrix_grid")
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        for i in 0..size {
                            for j in 0..size {
                                let value = &mut particle_system.behavior_matrix[i][j];
                                ui.add(egui::Slider::new(value, -1.0..=1.0));
                            }
                            ui.end_row();
                        }
                    });
            });
        });
}
