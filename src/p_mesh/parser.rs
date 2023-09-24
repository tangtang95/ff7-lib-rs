use anyhow::Result;
use std::{io::Read, mem::size_of};

use glam::{Vec2, Vec3};
use nom::{
    bytes::complete::take,
    combinator::{map, verify},
    multi::count,
    number::complete::{le_f32, le_i32, le_u32},
    sequence::Tuple,
    IResult,
};

use super::{
    data::{BGRAColor, PMesh, PMeshHeader},
    P_FILE_HEADER_SIZE,
};

/**
 * Parse P mesh file header of 128 bytes
 */
fn p_mesh_header(input: &[u8]) -> IResult<&[u8], PMeshHeader> {
    let (input, version) = le_i32(input)?;
    let (input, _) = take(4usize)(input)?;
    let (input, vertex_type) = le_i32(input)?;

    let (input, (num_vertices, num_normals, num_unk1)) =
        (le_i32_to_usize, le_i32_to_usize, le_i32_to_usize).parse(input)?;
    let (input, (num_tex_coords, num_vertex_colors)) =
        (le_i32_to_usize, le_i32_to_usize).parse(input)?;
    let (input, (num_edges, num_polys)) = (le_i32_to_usize, le_i32_to_usize).parse(input)?;

    let (input, (num_unk2, num_unk3)) = (le_i32_to_usize, le_i32_to_usize).parse(input)?;
    let (input, (num_hundreds, num_groups, num_bounding_boxes)) =
        (le_i32_to_usize, le_i32_to_usize, le_i32_to_usize).parse(input)?;
    let (input, norm_index_table_flags) = le_i32(input)?;

    let (input, _) = take(64usize)(input)?;

    Ok((
        input,
        PMeshHeader {
            version,
            vertex_type,
            num_vertices,
            num_normals,
            num_unk1,
            num_tex_coords,
            num_vertex_colors,
            num_edges,
            num_polys,
            num_unk2,
            num_unk3,
            num_hundreds,
            num_groups,
            num_bounding_boxes,
            norm_index_table_flags,
        },
    ))
}

fn p_mesh<'a>(input: &'a [u8], p_mesh_header: &PMeshHeader) -> IResult<&'a [u8], PMesh> {
    let (input, vertices) = count(vec3, p_mesh_header.num_vertices)(&input)?;
    let (input, normals) = count(vec3, p_mesh_header.num_normals)(&input)?;
    let (input, unk1_array) = count(vec3, p_mesh_header.num_unk1)(&input)?;
    let (input, tex_coords) = count(vec2, p_mesh_header.num_tex_coords)(&input)?;
    let (input, vertex_colors) = count(bgra_color, p_mesh_header.num_vertex_colors)(&input)?;

    Ok((
        input,
        PMesh::new(vertices, normals, unk1_array, tex_coords, vertex_colors),
    ))
}

fn vec3(input: &[u8]) -> IResult<&[u8], Vec3> {
    let (input, (x, y, z)) = (le_f32, le_f32, le_f32).parse(input)?;
    Ok((input, Vec3::new(x, y, z)))
}

fn vec2(input: &[u8]) -> IResult<&[u8], Vec2> {
    let (input, (x, y)) = (le_f32, le_f32).parse(input)?;
    Ok((input, Vec2::new(x, y)))
}

fn bgra_color(input: &[u8]) -> IResult<&[u8], BGRAColor> {
    let (input, color) = le_u32(input)?;
    Ok((input, BGRAColor::from(color)))
}

fn le_i32_to_usize(input: &[u8]) -> IResult<&[u8], usize> {
    let i32_positive_parser = verify(le_i32, |&num| num >= 0);
    map(i32_positive_parser, |num| num as usize)(input)
}

pub fn parse_p_mesh<T>(mut reader: T) -> Result<(PMeshHeader, PMesh)>
where
    T: Read,
{
    let mut input = [0u8; P_FILE_HEADER_SIZE];
    reader.read_exact(&mut input)?;
    let (_, p_mesh_header) = p_mesh_header(&input).map_err(|e| e.to_owned())?;

    let p_mesh_byte_size = (p_mesh_header.num_vertices * size_of::<f32>() * 3)
        + (p_mesh_header.num_normals * size_of::<f32>() * 3)
        + (p_mesh_header.num_unk1 * size_of::<f32>() * 3)
        + (p_mesh_header.num_tex_coords * size_of::<f32>() * 2)
        + (p_mesh_header.num_vertex_colors * size_of::<u32>());

    let mut input: Vec<u8> = vec![];
    reader
        .by_ref()
        .take(p_mesh_byte_size.try_into()?)
        .read_to_end(&mut input)?;

    let (_, p_mesh) = p_mesh(&input, &p_mesh_header).map_err(|e| e.to_owned())?;
    Ok((p_mesh_header, p_mesh))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs::File, io::BufReader};

    #[test]
    fn test() {
        let file = File::open("data/aaac.p").unwrap();
        let buf_reader = BufReader::new(file);
        let (p_mesh_header, p_mesh) = parse_p_mesh(buf_reader).unwrap();
        
        println!("{:?}\n\n{:?}", p_mesh_header, p_mesh);
    }
}
