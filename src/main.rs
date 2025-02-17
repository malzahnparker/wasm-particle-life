use bevy::{
    color::palettes::css,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::WindowResolution,
};
use rand::{seq::SliceRandom, Rng};

#[derive(Component)]
struct Particle {
    color_id: usize,
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
            css::RED,
            css::CRIMSON,
            css::CORAL,
            css::ORANGE,
            css::GOLD,
            css::GREEN_YELLOW,
            css::YELLOW_GREEN,
            css::GREEN,
            css::SEA_GREEN,
            css::DARK_CYAN,
            css::DEEP_SKY_BLUE,
            css::DODGER_BLUE,
            css::BLUE,
            css::MEDIUM_BLUE,
            css::INDIGO,
            css::BLUE_VIOLET,
        ];

        let mut rng = rand::rng();
        let num_colors = rng.random_range(2..=16);
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
        let attraction_radius = rng.random_range(50.0..=200.0);

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
        self.attraction_radius = rng.random_range(50.0..=200.0);
    }
}

const WINDOW_WIDTH: f32 = 1920.0;
const WINDOW_HEIGHT: f32 = 1080.0;
const PARTICLE_SIZE: f32 = 5.0;
const NUM_PARTICLES: usize = 5000;
const BASE_SPEED: f32 = 100.0;
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
            LogDiagnosticsPlugin::default(),
        ))
        .insert_resource(ParticleSystem::new())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_particles,
                move_camera,
                handle_matrix_regeneration,
                adjust_speed,
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
    let dt = time.delta_secs() * particle_system.speed;
    let beta = particle_system.beta;
    let gamma = particle_system.gamma;
    let gamma_beta_diff = gamma - beta;
    let one_minus_gamma = 1.0 - gamma;

    let particles: Vec<(Vec3, usize)> = particle_query
        .iter()
        .map(|(transform, particle)| (transform.translation, particle.color_id))
        .collect();

    for (mut transform, particle) in particle_query.iter_mut() {
        let mut force = Vec2::ZERO;
        let mut count = 0.0;

        for (other_pos, other_color_id) in particles.iter() {
            if transform.translation == *other_pos {
                continue;
            }

            let to_other = *other_pos - transform.translation;
            let distance = to_other.length() / particle_system.attraction_radius;

            if distance < 1.0 {
                let direction = to_other.truncate().normalize();
                let behavior = particle_system.get_behavior(particle.color_id, *other_color_id);

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

        if count > 0.0 {
            force /= count;
        }

        transform.translation += force.extend(0.0) * dt;
    }
}

fn move_camera(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
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
    if mouse.pressed(MouseButton::Left) {
        let mut rng = rand::rng();
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
    if mouse.pressed(MouseButton::Right) {
        let mut rng = rand::rng();
        for _ in 0..100 {
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

    if direction != Vec3::ZERO {
        direction = direction.normalize();
        let scale = camera_transform.scale;
        camera_transform.translation += direction * CAMERA_SPEED * time.delta_secs() * scale;
    }
}

fn handle_matrix_regeneration(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut particle_system: ResMut<ParticleSystem>,
) {
    if keyboard.just_pressed(KeyCode::KeyQ) {
        particle_system.regenerate_matrix();
        println!("Matrix regenerated: {:?}", particle_system.behavior_matrix);
    }
    if keyboard.just_pressed(KeyCode::KeyT) {
        particle_system.regenerate_constants();
        println!("Matrix regenerated: {:?}", particle_system.behavior_matrix);
    }
}

fn adjust_speed(keyboard: Res<ButtonInput<KeyCode>>, mut particle_system: ResMut<ParticleSystem>) {
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        particle_system.speed *= 2.0;
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
        particle_system.speed /= 2.0;
    }
}
