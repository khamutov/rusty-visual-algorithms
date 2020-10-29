mod animation;
mod geom;

use quicksilver::{
    geom::{Circle, Vector},
    graphics::{Color, Graphics},
    input::{Event, Key},
    Input, Result, Settings, Timer, Window,
};

use crate::animation::{Linear, LinearConfig};
use lyon::{geom::LineSegment, math::point};
use rand::distributions::{Distribution, Uniform, WeightedIndex};
use rand::prelude::ThreadRng;
use rand::Rng;
use std::vec::Vec;

#[derive(Debug)]
enum NodeState {
    New,
    Visited,
    Start,
    Target,
}

#[derive(Debug)]
struct Node {
    x: f32,
    y: f32,
    state: NodeState,
}

impl Node {
    fn draw(&self, gfx: &mut Graphics) {
        let node_view = Circle::new(Vector::new(self.x, self.y), 10.0);
        let color = match self.state {
            NodeState::New => Color::BLACK,
            NodeState::Visited => Color::RED,
            NodeState::Start => Color::BLUE,
            NodeState::Target => Color::GREEN,
        };
        gfx.fill_circle(&node_view, color);
        gfx.stroke_circle(&node_view, color);
    }
}

const HEIGHT: f32 = 1024.0;
const LENGTH: f32 = 768.0;
const VELOCITY: f32 = 768.0 / 5.0; // 1 screen for 5 sec

impl Node {
    pub fn new(mut rng: ThreadRng) -> Self {
        let x_dist = Uniform::from(0.0..HEIGHT);
        let y_dist = Uniform::from(0.0..LENGTH);
        Node {
            x: x_dist.sample(&mut rng),
            y: y_dist.sample(&mut rng),
            state: NodeState::New,
        }
    }
}

enum ConnectionState {
    Unexplored,
    Exploring,
    Explored,
    BestPath,
}

struct Connection {
    from_node: u32, // start node
    to_node: u32,   // end node
    from_coord: Vector,
    to_coord: Vector,
    animation: Option<Linear<(Vector, Vector)>>,
    state: ConnectionState,
}

impl Connection {
    fn draw(&mut self, gfx: &mut Graphics) {
        match self.animation.as_mut() {
            Some(animation) => {
                animation.draw(gfx);
                if animation.is_ended() {
                    self.animation = None;
                    self.state = ConnectionState::Explored;
                }
            }
            None => {
                let line1 = geom::LineT::new(self.from_coord, self.to_coord).with_thickness(4.0);
                let path1 = line1.draw();
                match self.state {
                    ConnectionState::Unexplored => {
                        gfx.fill_polygon(path1.as_slice(), Color::BLACK);
                    }
                    ConnectionState::Exploring => {
                        gfx.fill_polygon(path1.as_slice(), Color::RED);
                    }
                    ConnectionState::Explored => {
                        gfx.fill_polygon(path1.as_slice(), Color::RED);
                    }
                    ConnectionState::BestPath => {
                        gfx.fill_polygon(path1.as_slice(), Color::BLUE);
                    }
                }
            }
        }
    }

    fn explore(&mut self, from_node: u32) {
        match self.state {
            ConnectionState::Unexplored => {
                self.state = ConnectionState::Exploring;
                self.animate(from_node)
            }
            ConnectionState::Exploring => {} // do nothing, already in state
            ConnectionState::Explored => {}  // do nothing, already explored
            ConnectionState::BestPath => {}  // do nothing
        }
    }

    fn animate(&mut self, from_node: u32) {
        let ticks_per_second: f32 = 30.;
        let timing = Timer::time_per_second(ticks_per_second); // TODO: make global timer
        let dist = self.from_coord.distance(self.to_coord);
        let animation_duration = (ticks_per_second * dist / VELOCITY) as usize;

        let animation = LinearConfig {
            begin_state: if from_node == self.from_node {
                (self.from_coord, self.to_coord)
            } else {
                (self.to_coord, self.from_coord) // reversed animation
            },
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
            frame_count: animation_duration,
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
                    state: ConnectionState::Unexplored,
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

struct BFSAlgorithm {
    target: u32,
    frontier: std::collections::VecDeque<(u32, u32)>,
    visited: std::collections::HashSet<u32>,
    ended: bool,
}

impl BFSAlgorithm {
    pub fn new(start: u32, target: u32, world: &mut World) -> Self {
        let mut frontier = std::collections::VecDeque::new();
        BFSAlgorithm::add_all_siblings(start, world, &mut frontier);

        let mut visited = std::collections::HashSet::new();
        visited.insert(start);

        println!("frontier {:?}", frontier);
        println!("visited {:?}", visited);

        BFSAlgorithm {
            target,
            frontier,
            visited,
            ended: false,
        }
    }

    fn add_all_siblings(
        from_node: u32,
        world: &World,
        frontier: &mut std::collections::VecDeque<(u32, u32)>,
    ) {
        let initial_conns: Vec<&Connection> = world
            .connections
            .iter()
            .filter(|conn| (conn.from_node == from_node) || (conn.to_node == from_node))
            .collect();
        for conn in initial_conns {
            if conn.from_node == from_node {
                frontier.push_back((conn.from_node, conn.to_node))
            } else {
                frontier.push_back((conn.to_node, conn.from_node))
            }
        }
    }

    fn step(&mut self, world: &mut World) -> bool {
        if self.ended {
            return false;
        }

        println!("------");
        println!("frontier {:?}", self.frontier);
        println!("visited {:?}", self.visited);

        while let Some((from_node, to_node)) = self.frontier.pop_front() {
            if !self.visited.contains(&to_node) {
                // add connection as exploring
                let conn = world
                    .connections
                    .iter_mut()
                    .filter(|conn| {
                        (conn.from_node == from_node && conn.to_node == to_node)
                            || (conn.to_node == from_node && conn.from_node == to_node)
                    })
                    .next()
                    .map(|conn| conn.explore(from_node));

                // check for target
                if to_node == self.target {
                    self.ended = true;
                    println!("target reached!");
                    return false;
                }

                // add connection to that node to queue
                self.visited.insert(to_node);

                // insert all conns from new node
                BFSAlgorithm::add_all_siblings(to_node, world, &mut self.frontier);

                return true;
            }
        }
        self.ended = true;
        false
    }
}

async fn app(window: Window, mut gfx: Graphics, mut input: Input) -> Result<()> {
    let mut world = World::new();

    let mut rng = rand::thread_rng();
    let start_node = rng.gen_range(0, world.nodes.len());
    let target_node = {
        let pre_gen = rng.gen_range(0, world.nodes.len());
        if pre_gen == start_node {
            (pre_gen + 1) % world.nodes.len()
        } else {
            pre_gen
        }
    };
    world.nodes[start_node].state = NodeState::Start;
    world.nodes[target_node].state = NodeState::Target;

    let mut alg = BFSAlgorithm::new(start_node as u32, target_node as u32, &mut world);

    let mut running = true;
    while running {
        while let Some(event) = input.next_event().await {
            match event {
                Event::KeyboardInput(key) if key.is_down() => {
                    if key.key() == Key::Escape {
                        // If the user strikes escape, end the program
                        running = false;
                    }
                }
                _ => (),
            }
        }

        // if all animations ended
        if !world
            .connections
            .iter_mut()
            .any(|conn| conn.animation.is_some())
        {
            alg.step(&mut world);
        }

        // Clear the screen to a blank, white color
        gfx.clear(Color::WHITE);
        for node in world.nodes.iter() {
            node.draw(&mut gfx);
        }

        for connection in world.connections.iter_mut() {
            connection.draw(&mut gfx);
        }

        gfx.present(&window)?;
    }
    Ok(())
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
