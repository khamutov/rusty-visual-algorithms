mod animation;
mod geom;

use quicksilver::{
    geom::{Circle, Vector},
    graphics::{Color, Graphics},
    Input, Result, Settings, Timer, Window,
};

use crate::animation::{Linear, LinearConfig};
use lyon::{geom::LineSegment, math::point};
use rand::distributions::{Distribution, Uniform, WeightedIndex};
use rand::prelude::ThreadRng;
use std::vec::Vec;

#[derive(Debug)]
struct Node {
    x: f32,
    y: f32,
}

const HEIGHT: f32 = 1024.0;
const LENGTH: f32 = 768.0;

impl Node {
    pub fn new(mut rng: ThreadRng) -> Self {
        let x_dist = Uniform::from(0.0..HEIGHT);
        let y_dist = Uniform::from(0.0..LENGTH);
        Node {
            x: x_dist.sample(&mut rng),
            y: y_dist.sample(&mut rng),
        }
    }
}

struct Connection {
    from_node: u32, // start node
    to_node: u32,   // end node
    from_coord: Vector,
    to_coord: Vector,
    animation: Option<Linear<(Vector, Vector)>>,
}

impl Connection {
    fn draw(&mut self, gfx: &mut Graphics) {
        match self.animation.as_mut() {
            Some(animation) => {
                animation.draw(gfx);
                if animation.is_ended() {
                    self.animation = None
                }
            }
            None => {
                let line1 = geom::LineT::new(self.from_coord, self.to_coord).with_thickness(4.0);
                let path1 = line1.draw();
                gfx.fill_polygon(path1.as_slice(), Color::BLACK);
            }
        }
    }

    fn animate(&mut self) {
        let timing = Timer::time_per_second(30.); // TODO: make global timer

        let animation = LinearConfig {
            begin_state: (self.from_coord, self.to_coord),
            timing,
            draw: Box::new(|state, percent, gfx| {
                let vec1 = state.0;
                let vec2 = state.1;
                let middle_point = Vector::new(
                    vec1.x + (vec2.x - vec1.x) * percent,
                    vec1.y + (vec2.y - vec1.y) * percent,
                );

                let line1 = geom::LineT::new(vec1, middle_point).with_thickness(4.0);
                let path1 = line1.draw();
                gfx.fill_polygon(path1.as_slice(), Color::RED);

                let line2 = geom::LineT::new(vec2, middle_point).with_thickness(4.0);
                let path2 = line2.draw();
                gfx.fill_polygon(path2.as_slice(), Color::BLACK);

                Ok(())
            }),
            frame_count: 150,
        }
        .start();

        self.animation = Some(animation);
    }
}

struct World {
    nodes: Vec<Node>,
    connections: Vec<Connection>,
}

impl World {
    pub fn new() -> World {
        let node_count = 20;

        let mut rng = rand::thread_rng();
        let nodes: Vec<Node> = (0..node_count).map(|_| Node::new(rng)).collect();

        let connections_count = 40;
        let mut connections: Vec<(u32, u32)> = Vec::new();
        for _ in 0..connections_count {
            let distrib = Uniform::from(0..nodes.len());
            let node_index = distrib.sample(&mut rng);
            let node = nodes.get(node_index).unwrap();
            let weights: Vec<f32> = nodes
                .iter()
                .map(|i| (euclidean_dist(node, i) / HEIGHT).exp())
                .collect();

            let max_weights: f32 = weights
                .iter()
                .fold(0.0f32, |val, next_val| next_val.max(val));
            let normalized_weights: Vec<f32> =
                weights.into_iter().map(|elem| max_weights - elem).collect();

            let sum_weights: f32 = normalized_weights.iter().sum();
            let mut scaled_weights: Vec<f32> = normalized_weights
                .into_iter()
                .map(|elem| elem / sum_weights)
                .collect();

            for (supposed_node_position, item) in scaled_weights.iter_mut().enumerate() {
                let supposed_node = nodes.get(supposed_node_position).unwrap();
                let supposed_line = LineSegment {
                    from: point(node.x, node.y),
                    to: point(supposed_node.x, supposed_node.y),
                };
                for (fst_connection, snd_connection) in connections.iter() {
                    let node1 = nodes.get(*fst_connection as usize).unwrap();
                    let node2 = nodes.get(*snd_connection as usize).unwrap();
                    let existed_line = LineSegment {
                        from: point(node1.x, node1.y),
                        to: point(node2.x, node2.y),
                    };
                    if supposed_line.intersects(&existed_line) {
                        *item = 0.0f32;
                        continue;
                    }
                }
            }

            let weighted_distrib = match WeightedIndex::new(&scaled_weights) {
                Ok(weighted_distrib) => weighted_distrib,
                Err(_) => continue,
            };
            let suggested_index = weighted_distrib.sample(&mut rng) as u32;
            let node_index = node_index as u32;
            if connections.contains(&(node_index, suggested_index))
                || connections.contains(&(suggested_index, node_index))
                || suggested_index == node_index
            {
                continue;
            }
            connections.push((node_index, suggested_index))
        }

        let conn = connections
            .into_iter()
            .map(|(a, b)| -> Connection {
                let node1 = &nodes[a as usize];
                let node2 = &nodes[b as usize];
                let node1_coord = Vector::new(node1.x, node1.y);
                let node2_coord = Vector::new(node2.x, node2.y);
                Connection {
                    from_node: a,
                    to_node: b,
                    from_coord: node1_coord,
                    to_coord: node2_coord,
                    animation: None,
                }
            })
            .collect();
        World {
            nodes,
            connections: conn,
        }
    }
}

#[inline]
fn euclidean_dist(n1: &Node, n2: &Node) -> f32 {
    ((n1.x - n2.x).powi(2) + (n1.y - n2.y).powi(2)).sqrt()
}

async fn app(window: Window, mut gfx: Graphics, mut input: Input) -> Result<()> {
    let mut world = World::new();

    for connection in world.connections.iter_mut() {
        connection.animate();
    }

    loop {
        while let Some(_) = input.next_event().await {}

        // Clear the screen to a blank, white color
        gfx.clear(Color::WHITE);
        for node in world.nodes.iter() {
            let node_view = Circle::new(Vector::new(node.x, node.y), 10.0);
            gfx.fill_circle(&node_view, Color::BLACK);
            gfx.stroke_circle(&node_view, Color::BLACK);
        }

        for connection in world.connections.iter_mut() {
            connection.draw(&mut gfx);
        }

        gfx.present(&window)?;
    }
}

fn main() {
    quicksilver::run(
        Settings {
            title: "Square Example",
            ..Settings::default()
        },
        app,
    );
}
