use crate::structs::Vertex;

use std::path::Path;

extern crate tobj;
use cgmath::{Vector2, Vector3};

pub fn load() -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = vec![];
    let mut indices = vec![];

    let (models, _) = tobj::load_obj(&Path::new("src/resources/models/viking_room.obj"), true)
        .expect("Failed to load model!");

    for model in models.iter() {
        let tobj::Model { mesh, name: _name } = model;

        for i in 0..(mesh.positions.len() / 3) {
            let vertex = Vertex {
                position: Vector3 {
                    x: mesh.positions[i * 3],
                    y: mesh.positions[i * 3 + 1],
                    z: mesh.positions[i * 3 + 2],
                },
                color: Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                },
                uv: Vector2 {
                    x: mesh.texcoords[i * 2],
                    y: 1.0 - mesh.texcoords[i * 2 + 1],
                },
            };
            vertices.push(vertex);
        }

        indices = mesh.indices.clone();
    }

    (vertices, indices)
}
