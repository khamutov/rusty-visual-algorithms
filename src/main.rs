use quicksilver::{
    geom::{Circle, Vector},
    graphics::{Color, Graphics},
    Input, Result, Settings, Window,
};

use rand::distributions::{Distribution, Uniform};
use std::vec::Vec;

struct Node {
    x: f32,
    y: f32,
}

struct World {
    nodes: Vec<Node>,
    connections: Vec<(u32, u32)>,
}

fn gen_world() -> World {
    let node_count = 20;

    let mut nodes = Vec::new();

    let mut rng = rand::thread_rng();
    let x_dist = Uniform::from(0.0..1024.0);
    let y_dist = Uniform::from(0.0..768.0);
    for _ in 0..node_count {
        let new_node = Node {
            x: x_dist.sample(&mut rng),
            y: y_dist.sample(&mut rng),
        };

        nodes.push(new_node);
    }

    let connections_count = 40;
    let mut connections = Vec::new();
    let conn_dist = Uniform::from(0..node_count);
    for _ in 0..connections_count {
        connections.push((conn_dist.sample(&mut rng), conn_dist.sample(&mut rng)))
    }

    World { nodes, connections }
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

async fn app(window: Window, mut gfx: Graphics, mut input: Input) -> Result<()> {
    let world = gen_world();

    // Clear the screen to a blank, white color
    gfx.clear(Color::WHITE);
    for node in &world.nodes {
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
        while let Some(_) = input.next_event().await {}
    }
}
