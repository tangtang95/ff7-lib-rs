use glam::{Vec3, Vec2};
use derive_more::Constructor;

#[derive(Debug)]
pub struct PModelHeader {
    pub version: i32,
    pub vertex_type: i32,
    pub num_vertices: i32, 
    pub num_normals: i32,
    pub num_unk1: i32,
    pub num_tex_coords: i32,
    pub num_vertex_colors: i32,
    pub num_edges: i32,
    pub num_polys: i32,
    pub num_unk2: i32,
    pub num_unk3: i32,
    pub num_hundreds: i32,
    pub num_groups: i32,
    pub num_bounding_boxes: i32,
    pub norm_index_table_flags: i32
}

#[derive(Debug, Clone, Constructor)]
pub struct BGRAColor {
    b: u8,
    g: u8,
    r: u8,
    a: u8
}

#[derive(Default, Debug, Constructor)]
pub struct PModel {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    unk1: Vec<Vec3>,
    tex_coords: Vec<Vec2>,
    vertex_colors: Vec<BGRAColor> 
}

impl From<u32> for BGRAColor {
    fn from(value: u32) -> Self {
        BGRAColor::new(
            ((value >> 24) & 0xFF) as u8,
            ((value >> 16) & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8,
        )
    }
}
