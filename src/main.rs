use quicksilver::{
    geom::{Circle, Vector},
    graphics::{Color, Graphics},
    Input, Result, Settings, Window,
};

use rand::distributions::{Distribution, Uniform, WeightedIndex};
use std::vec::Vec;
use rand::prelude::ThreadRng;
use lyon::{geom::LineSegment, math::point};

#[derive(Debug)]
struct Node {
    x: f32,
    y: f32,
}

const HEIGHT: f32 = 1024.0;
const LENGTH: f32 = 768.0;

impl Node {
    pub fn gen(mut rng: ThreadRng) -> Self {
        let x_dist = Uniform::from(0.0..HEIGHT);
        let y_dist = Uniform::from(0.0..LENGTH);
        Node {
            x: x_dist.sample(&mut rng),
            y: y_dist.sample(&mut rng),
        }
    }
}

struct World {
    nodes: Vec<Node>,
    connections: Vec<(u32, u32)>,
}

impl World {
    pub fn gen() -> World {
        let node_count = 20;

        let mut rng = rand::thread_rng();
        let nodes: Vec<Node> = (0..node_count)
            .map(|_| Node::gen(rng))
            .collect();

        let connections_count = 40;
        let mut connections: Vec<(u32, u32)>  = Vec::new();
        for i in 0..connections_count {
            let distrib = Uniform::from(0..nodes.len());
            let node_index = distrib.sample(&mut rng);
            let node = nodes.get(node_index).unwrap();
            let weights: Vec<f32> = nodes.iter()
                .map(|i| (euclidean_dist(node, i) / 1000.0).exp())
                .collect();

            let max_weights: f32 = weights.iter().fold(0.0f32, |val, next_val| next_val.max(val));
            let normalized_weights: Vec<f32> = weights.into_iter()
                .map(|elem| max_weights - elem)
                .collect();

            let sum_weights: f32 = normalized_weights.iter().sum();
            let mut scaled_weights: Vec<f32> = normalized_weights.into_iter()
                .map(|elem| elem / sum_weights)
                .collect();

            #[cfg(debug_assertions)]
            if i == 0 {
                println!("{:?}", &scaled_weights);
            };


            for (fst_connection, snd_connection) in connections.iter() {
                let node1 = nodes.get(*fst_connection as usize).unwrap();
                let node2 = nodes.get(*snd_connection as usize).unwrap();
                let existed_line = LineSegment {
                    from: point(node1.x, node1.y),
                    to: point(node2.x, node2.y)
                };
                for (supposed_node_position, item) in scaled_weights.iter_mut().enumerate() {
                    let supposed_node = nodes.get(supposed_node_position).unwrap();
                    let supposed_line = LineSegment {
                        from: point(node.x, node.y),
                        to: point(supposed_node.x, supposed_node.y)
                    };
                    if supposed_line.intersects(&existed_line) {
                        *item = 0.0f32;
                    }
                }
            }

            #[cfg(debug_assertions)]
            if i == 39 {
                println!("{:?}", &scaled_weights);
            };

            let weighted_distrib = match WeightedIndex::new(&scaled_weights) {
                Ok(weighted_distrib) => weighted_distrib,
                Err(_) => continue
            };
            let suggested_index = weighted_distrib.sample(&mut rng) as u32;
            let node_index = node_index as u32;
            if connections.contains(&(node_index, suggested_index)) ||
                connections.contains(&(suggested_index, node_index)) ||
                suggested_index == node_index {
                continue
            }
            connections.push((node_index, suggested_index))
        };



        World { nodes, connections }
    }
}

#[inline]
fn euclidean_dist(n1: &Node, n2: &Node) -> f32 {
    ((n1.x - n2.x).powi(2) + (n1.y - n2.y).powi(2)).sqrt()
}

async fn app(window: Window, mut gfx: Graphics, mut input: Input) -> Result<()> {
    let world = World::gen();

    // Clear the screen to a blank, white color
    gfx.clear(Color::WHITE);
    for node in world.nodes.iter() {
        let node_view = Circle::new(Vector::new(node.x, node.y), 10.0);
        gfx.fill_circle(&node_view, Color::BLACK);
        gfx.stroke_circle(&node_view, Color::BLACK);
    }

    for connection in world.connections {
        let node1 = &world.nodes[connection.0 as usize];
        let node2 = &world.nodes[connection.1 as usize];
        gfx.stroke_path(
            &[Vector::new(node1.x, node1.y), Vector::new(node2.x, node2.y)],
            Color::BLACK,
        );
    }

    gfx.present(&window)?;
    loop {
        while input.next_event().await.is_some() {}
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