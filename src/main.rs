use avian2d::prelude::*;
use bevy::{
    color::palettes::css,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::WindowResolution,
};
use rand::Rng;

#[derive(Component)]
struct Particle {
    color_id: usize,
}

#[derive(Resource)]
struct ParticleSystem {
    colors: Vec<Color>,
    behavior_matrix: Vec<Vec<f32>>,
    speed: f32,
}

impl ParticleSystem {
    fn new() -> Self {
        let colors = [css::RED, css::YELLOW, css::BLUE];
        let mut rng = rand::rng();
        let n = colors.len();

        let behavior_matrix = (0..n)
            .map(|_| (0..n).map(|_| rng.random_range(-1.0..=1.0)).collect())
            .collect();

        // Or specify exact behaviors like this:
        // let behavior_matrix = vec![
        //     vec![ 1.0, -1.0,  0.5], // Red's behavior
        //     vec![ 1.0, -0.5, -0.5], // Yellow's behavior
        //     vec![ 0.5, -0.5, -0.5], // Green's behavior
        // ];

        ParticleSystem {
            colors: colors.map(|color| color.into()).into(),
            behavior_matrix,
            speed: BASE_SPEED,
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
}

const WINDOW_WIDTH: f32 = 384.0;
const WINDOW_HEIGHT: f32 = 216.0;
const PARTICLE_SIZE: f32 = 1.0;
const ATTRACTION_RADIUS: f32 = 20.0;
const NUM_PARTICLES: usize = 2000;
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
            PhysicsPlugins::default(),
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
            RigidBody::Dynamic,
            Collider::circle(PARTICLE_SIZE / 2.0),
            LockedAxes::ROTATION_LOCKED,
        ));
    }
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(0.0, WINDOW_HEIGHT),
        Transform::from_xyz(-WINDOW_WIDTH / 2.0, 0.0, 0.0),
    ));
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(0.0, WINDOW_HEIGHT),
        Transform::from_xyz(WINDOW_WIDTH / 2.0, 0.0, 0.0),
    ));
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(WINDOW_WIDTH, 0.0),
        Transform::from_xyz(0.0, -WINDOW_HEIGHT / 2.0, 0.0),
    ));
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(WINDOW_WIDTH, 0.0),
        Transform::from_xyz(0.0, WINDOW_HEIGHT / 2.0, 0.0),
    ));
}

fn update_particles(
    particle_system: Res<ParticleSystem>,
    mut particle_query: Query<(&Transform, &Particle, &mut LinearVelocity)>,
) {
    let particles: Vec<(Vec3, usize)> = particle_query
        .iter()
        .map(|(transform, particle, _)| (transform.translation, particle.color_id))
        .collect();

    for (transform, particle, mut velocity) in particle_query.iter_mut() {
        let mut force = Vec2::ZERO;
        let mut count = 0.0;

        for (other_pos, other_color_id) in &particles {
            if transform.translation == *other_pos {
                continue;
            }

            let distance = transform.translation.distance(*other_pos);
            let direction = (*other_pos - transform.translation).truncate().normalize();

            if distance < ATTRACTION_RADIUS {
                let behavior = particle_system.get_behavior(particle.color_id, *other_color_id);

                // Apply force based on distance (stronger when closer)

                force += direction * behavior;

                count += 1.0;
            }
        }

        if count > 0.0 {
            force /= count;
        }

        // The velocity is directly determined by the force (which includes the behavior multiplier)

        // Then in the update function:
        velocity.x = force.x * particle_system.speed;
        velocity.y = force.y * particle_system.speed;
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
            RigidBody::Dynamic,
            Collider::circle(PARTICLE_SIZE / 2.0),
            LockedAxes::ROTATION_LOCKED,
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
                RigidBody::Dynamic,
                Collider::circle(PARTICLE_SIZE / 2.0),
                LockedAxes::ROTATION_LOCKED,
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
    if keyboard.just_pressed(KeyCode::KeyR) {
        particle_system.regenerate_matrix();
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
