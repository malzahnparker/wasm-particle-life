use avian2d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
use rand::Rng;

// Component to identify particles
#[derive(Component)]
struct Particle {
    color: ParticleColor,
}

#[derive(PartialEq, Clone)]
enum ParticleColor {
    Red,
    Green,
}

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const PARTICLE_SIZE: f32 = 5.0;
const PARTICLE_SPEED: f32 = 100.0;
const ATTRACTION_RADIUS: f32 = 100.0;
const NUM_PARTICLES: usize = 500;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, update_particles)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d::default());

    let mut rng = rand::rng();

    // Spawn particles
    for _ in 0..NUM_PARTICLES {
        let x = rng.random_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
        let y = rng.random_range(-WINDOW_HEIGHT / 2.0..WINDOW_HEIGHT / 2.0);

        // Randomly choose color
        let color = if rng.random_bool(0.5) {
            ParticleColor::Red
        } else {
            ParticleColor::Green
        };

        commands.spawn((
            Sprite::from_color(
                match color {
                    ParticleColor::Red => css::RED,
                    ParticleColor::Green => css::GREEN,
                },
                Vec2::new(PARTICLE_SIZE, PARTICLE_SIZE),
            ),
            Transform::from_xyz(x, y, 0.0),
            Particle { color },
            RigidBody::Dynamic,
            Collider::circle(PARTICLE_SIZE / 2.0),
        ));
    }
}

fn update_particles(
    mut particle_query: Query<(&mut Transform, &mut Particle, &mut LinearVelocity)>,
) {
    let particles: Vec<(Vec3, ParticleColor, LinearVelocity)> = particle_query
        .iter()
        .map(|(transform, particle, velocity)| {
            (
                transform.translation,
                particle.color.clone(),
                velocity.clone(),
            )
        })
        .collect();

    for (transform, particle, mut velocity) in particle_query.iter_mut() {
        let mut direction = Vec2::ZERO;
        let mut count = 0.0;

        for (other_pos, other_color, _) in &particles {
            if transform.translation == *other_pos {
                continue;
            }

            let distance = transform.translation.distance(*other_pos);
            if distance < ATTRACTION_RADIUS {
                let force_direction = (*other_pos - transform.translation).normalize();

                match (particle.color.clone(), other_color) {
                    (ParticleColor::Red, ParticleColor::Red) => {
                        // Red attracts red
                        direction += force_direction.truncate();
                    }
                    (ParticleColor::Green, ParticleColor::Green) => {
                        // Green attracts green
                        direction += force_direction.truncate();
                    }
                    (ParticleColor::Red, ParticleColor::Green) => {
                        // Red follows green
                        direction += force_direction.truncate();
                    }
                    (ParticleColor::Green, ParticleColor::Red) => {
                        // Green runs from red
                        direction -= force_direction.truncate();
                    }
                }
                count += 1.0;
            }
        }

        if count > 0.0 {
            direction = (direction / count).normalize();
        }

        velocity.x = direction.x * PARTICLE_SPEED;
        velocity.y = direction.y * PARTICLE_SPEED;
    }
}
