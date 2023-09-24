use glam::{Vec3, Vec2};
use derive_more::Constructor;

#[derive(Debug)]
pub struct PMeshHeader {
    pub version: i32,
    pub vertex_type: i32,
    pub num_vertices: usize, 
    pub num_normals: usize,
    pub num_unk1: usize,
    pub num_tex_coords: usize,
    pub num_vertex_colors: usize,
    pub num_edges: usize,
    pub num_polys: usize,
    pub num_unk2: usize,
    pub num_unk3: usize,
    pub num_hundreds: usize,
    pub num_groups: usize,
    pub num_bounding_boxes: usize,
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
pub struct PMesh {
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
