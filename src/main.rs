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

    World { nodes }
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
    for node in world.nodes {
        let node_view = Circle::new(Vector::new(node.x, node.y), 10.0);
        gfx.fill_circle(&node_view, Color::BLACK);
        gfx.stroke_circle(&node_view, Color::BLACK);
    }

    // Paint a blue square with a red outline in the center of our screen
    // It should have a top-left of (350, 100) and a size of (150, 100)
    // let rect = Rectangle::new(Vector::new(350.0, 100.0), Vector::new(100.0, 100.0));
    // gfx.fill_rect(&rect, Color::BLUE);
    // gfx.stroke_rect(&rect, Color::RED);
    // Send the data to be drawn
    gfx.present(&window)?;
    loop {
        while let Some(_) = input.next_event().await {}
    }
}
