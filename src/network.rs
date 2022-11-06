use bevy::utils::HashMap;
use bevy::window::Window;
use bevy::{prelude::*, utils::HashSet};
use bevy_egui::{self, egui, EguiContext};
use bevy_inspector_egui::egui::output;
use bevy_inspector_egui::egui::{epaint::CircleShape, Color32, Pos2, Shape, Stroke, Style, Vec2};
use neat::{
    genome::{Gene, Genome},
    trainer::Trainer,
};
use rand::{thread_rng, Rng};

use core::ops::Deref;

pub struct NetworkState {
    pub genes: Vec<Gene>,
    pub sensor_nodes: usize,
    pub output_nodes: usize,
}

impl Default for NetworkState {
    fn default() -> Self {
        let new_gene = |node_in, node_out| Gene {
            node_in,
            node_out,
            weight: thread_rng().gen_range(-1f32..=1f32),
            enabled: true,
            innovation: 0,
        };

        Self {
            genes: vec![
                new_gene(0, 3),
                new_gene(1, 3),
                new_gene(3, 2),
                new_gene(0, 4),
                new_gene(1, 4),
                new_gene(4, 2),
                new_gene(0, 5),
                new_gene(1, 5),
                new_gene(5, 2),
            ],
            sensor_nodes: 2,
            output_nodes: 1,
        }
    }
}

#[derive(Default, Debug)]
pub struct Node {
    input: Vec<usize>,
    output: Vec<usize>,
    id: NodeType,
}

pub fn construct_network(state: &NetworkState) -> (Vec<Node>, HashMap<usize, usize>) {
    let mut found: HashMap<usize, usize> = default();
    let mut network: Vec<Node> = vec![];

    let enabled_genes = state.genes.iter().filter(|g| g.enabled);
    let mut i = 0;
    for gene in enabled_genes {
        let mut try_add_node = |id: usize| {
            let get = found.get(&id);
            if get.is_none() {
                found.insert(id, i);
                network.push(Node {
                    input: vec![],
                    output: vec![],
                    id: NodeType::get(state, id),
                });
                i += 1;
                return i - 1;
            } else {
                *get.unwrap()
            }
        };
        let input = try_add_node(gene.node_in);
        let output = try_add_node(gene.node_out);

        network[input].output.push(output);
        network[output].input.push(input);
    }
    dbg!(&network);
    (network, found)
}

/// for ergonomics while working with egui color
pub trait IntoColor32 {
    fn into_col32(&self) -> Color32;
}

impl IntoColor32 for Color {
    fn into_col32(&self) -> Color32 {
        use Color::*;
        match self {
            Rgba {
                red: r,
                green: g,
                blue: b,
                alpha: _,
            } => Color32::from_rgb((r * 256.) as u8, (g * 256.) as u8, (b * 256.) as u8),
            _ => self.as_rgba().into_col32(),
        }
    }
}

pub fn node_color(value: f32) -> Color32 {
    let start = [0.0, 0.9357, 0.5118];
    let end = [120.0, 0.9048, 0.5059];
    let mut out = [0.0, 0.0, 0.0];

    for (i, (a, b)) in start.iter().zip(end.iter()).enumerate() {
        out[i] = a + (b - a) * value
    }
    Color::hsl(out[0], out[1], out[2]).into_col32()
}
#[derive(Copy, Clone, Debug)]
enum NodeType {
    Sensor(usize),
    Output(usize),
    Hidden(usize),
}

impl Default for NodeType {
    fn default() -> Self {
        Self::Sensor(0)
    }
}

impl NodeType {
    pub fn get(state: &NetworkState, id: usize) -> Self {
        let mut cpy = id;
        if cpy < state.sensor_nodes {
            return Self::Sensor(cpy);
        }
        cpy -= state.sensor_nodes;
        if cpy < state.output_nodes {
            return Self::Output(cpy);
        }
        Self::Hidden(id)
    }

    pub fn index(self, state: &NetworkState) -> usize {
        match self {
            NodeType::Sensor(n) => n,
            NodeType::Output(n) => state.sensor_nodes + n,
            NodeType::Hidden(n) => n,
        }
    }
}

const overall_mult: f32 = 1.0;

pub fn spring_force(a: Vec2, b: Vec2) -> Vec2 {
    let resting_len = 50.0;
    let spring = 0.2;

    let len = (a - b).length();
    let diff = len - resting_len;
    (b - a).normalized() * diff * spring * overall_mult
}

pub fn repulsion(a: Vec2, b: Vec2) -> Vec2 {
    // let strength = 0.6;

    // let len = (a - b).length_sq();
    // let x = ((a - b) / len);
    // let out = (x * strength * overall_mult);
    // // dbg!(len);
    // // spring_force(a, b) * 0.5
    // out

    let resting_len = 150.0;
    let spring = 0.001;

    let len = (a - b).length();
    let diff = len - resting_len;
    (b - a).normalized() * diff * spring * overall_mult
}

pub fn network_ui(
    // duh, necessary for egui
    mut egui_context: ResMut<EguiContext>,
    // stores the state of the ui
    state: ResMut<NetworkState>,
    // previous size
    mut prev_size: Local<Vec2>,
    // previous window pos / anchor
    mut prev_pos: Local<Pos2>,
    // // stored shapes
    // mut shapes: Local<Vec<Shape>>,
    mut setup: Local<bool>,
    mut network: Local<Vec<Node>>,
    mut id_to_index: Local<HashMap<usize, usize>>,
    mut node_pos: Local<Vec<Vec2>>,
) {
    if !*setup {
        (*network, *id_to_index) = construct_network(&*state);
    }
    egui_context.ctx_mut().set_visuals(egui::Visuals::light());
    egui::Window::new("Network Visualizer")
        .collapsible(false)
        .resizable(false)
        .default_size([400.0, 400.0])
        .show(egui_context.ctx_mut(), |ui| {
            let size = ui.available_size();
            let pos = ui.clip_rect().min;
            ui.set_min_size(size);

            // // redraw the shapes
            // if size != *prev_size || *prev_pos != pos {
            //      = draw_network(size, pos, &state);
            //     *prev_size = size;
            //     *prev_pos = pos;
            // }

            let mut nodes: Vec<Shape> = vec![];

            let padding = 0.05;
            let unit = 30.0;

            let node_stroke = Stroke {
                width: unit / 20.0,
                color: Color32::from_gray(0),
            };

            let mut draw_nodes = |num, xpos, offset: f32| {
                let node_spacing = unit * 1.5;

                let mut ret: Vec<Vec2> = default();
                for i in 0..num {
                    let y = (offset + size.y - node_spacing * num as f32) / 2.0
                        + node_spacing * i as f32;
                    let center = [xpos, y + node_spacing / 2.0].into();
                    ret.push(center);
                    // nodes.push(Shape::Circle(CircleShape {
                    //     center: (pos + center).into(),
                    //     radius: unit / 2.0,
                    //     fill: node_color(thread_rng().gen::<f32>()),
                    //     stroke: node_stroke,
                    // }));
                }
                ret
            };

            let mut sensor_pos = draw_nodes(state.sensor_nodes, unit, 0.0);
            let mut output_pos = draw_nodes(state.output_nodes, size.x - unit, unit);

            let get_node_index = |node: &Node| *id_to_index.get(&node.id.index(&state)).unwrap();
            let random = || rand::thread_rng().gen_range(-10.0..=10.0);

            let get_pos_from_id = |x: usize, index| match network[x].id {
                NodeType::Sensor(x) => sensor_pos[x],
                NodeType::Output(x) => output_pos[x],
                NodeType::Hidden(x) => node_pos[index],
            };

            if !*setup {
                *node_pos = vec![default(); network.len()];
                for (i, node) in network.iter().enumerate() {
                    node_pos[i] = match node.id {
                        NodeType::Sensor(x) => sensor_pos[x],
                        NodeType::Output(x) => output_pos[x],
                        NodeType::Hidden(x) => {
                            Vec2::new(size.x / 2.0, size.y / 2.0) + Vec2::new(random(), random())
                        }
                    };
                }
            } else {
                let mut copy: Vec<Vec2> = default();
                for (i, center) in node_pos.iter().enumerate() {
                    if let NodeType::Hidden(_) = network[i].id {
                        let mut new = *center;
                        for id in network[i].input.iter().chain(network[i].output.iter()) {
                            let factor = 40.0;
                            let mut val = spring_force(*center, get_pos_from_id(*id, i)) / factor;
                            if let NodeType::Hidden(_) = network[*id].id {
                                val *= factor;
                            }
                            new = new + val
                        }
                        for &pos in node_pos.iter() {
                            if *center == pos {
                                continue;
                            }
                            new = new + repulsion(*center, pos);
                        }
                        copy.push(new);
                    } else {
                        copy.push(*center)
                    }
                }
                // dbg!(&copy);
                *node_pos = copy;
            }

            for (i, center) in node_pos.iter().enumerate() {
                nodes.push(Shape::Circle(CircleShape {
                    center: (pos + *center),
                    radius: unit / 2.0,
                    fill: node_color(i as f32 / node_pos.len() as f32),
                    stroke: node_stroke,
                }));
            }

            let mut connections: Vec<Shape> = vec![];

            let min_width = 4.0;
            let max_width = 7.0;

            for (i, gene) in state.genes.iter().enumerate() {
                let input = id_to_index.get(&gene.node_in).unwrap();
                let output = id_to_index.get(&gene.node_out).unwrap();

                connections.push(Shape::LineSegment {
                    points: [pos + node_pos[*input], pos + node_pos[*output]],
                    stroke: Stroke {
                        width: (max_width - min_width) * gene.weight.abs() + min_width,
                        color: node_color(i as f32 / state.genes.len() as f32),
                    },
                })
            }

            ui.painter().extend(connections);
            ui.painter().extend(nodes);
        });

    if !*setup {
        *setup = true;
    }
}

pub fn info_widget(
    // duh, necessary for egui
    mut egui_context: ResMut<EguiContext>,
    state: Res<NetworkState>,
) {
    egui_context.ctx_mut().set_visuals(egui::Visuals::light());
    egui::Window::new("Network Info")
        .collapsible(false)
        .resizable(false)
        .default_size([400.0, 400.0])
        .show(egui_context.ctx_mut(), |ui| {
            ui.label(format!("Inputs: {}", state.sensor_nodes));
            ui.label(format!("Outputs: {}", state.output_nodes));
            ui.label(format!("Connections: {}", state.genes.len()));
        });
}
