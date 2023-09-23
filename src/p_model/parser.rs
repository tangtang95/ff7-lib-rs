use anyhow::Result;
use std::{io::Read, mem::size_of};

use glam::{Vec2, Vec3};
use nom::{
    bytes::complete::take,
    multi::count,
    number::complete::{le_f32, le_i32, le_u32},
    sequence::Tuple,
    IResult,
};

use super::{
    data::{BGRAColor, PModel, PModelHeader},
    P_FILE_HEADER_SIZE,
};

/**
 * Parse P model file header of 128 bytes
 */
fn p_model_header(input: &[u8]) -> IResult<&[u8], PModelHeader> {
    let (input, version) = le_i32(input)?;
    let (input, _) = take(4usize)(input)?;
    let (input, vertex_type) = le_i32(input)?;

    let (input, (num_vertices, num_normals, num_unk1)) = (le_i32, le_i32, le_i32).parse(input)?;
    let (input, (num_tex_coords, num_vertex_colors)) = (le_i32, le_i32).parse(input)?;
    let (input, (num_edges, num_polys)) = (le_i32, le_i32).parse(input)?;

    let (input, (num_unk2, num_unk3)) = (le_i32, le_i32).parse(input)?;
    let (input, (num_hundreds, num_groups, num_bounding_boxes)) =
        (le_i32, le_i32, le_i32).parse(input)?;
    let (input, norm_index_table_flags) = le_i32(input)?;

    let (input, _) = take(64usize)(input)?;

    Ok((
        input,
        PModelHeader {
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

pub fn parse_p_model<T>(mut reader: T) -> Result<PModel>
where
    T: Read,
{
    let mut input = [0u8; P_FILE_HEADER_SIZE];
    reader.read_exact(&mut input)?;
    let (_, p_file_header) = p_model_header(&input).map_err(|e| e.to_owned())?;

    let mut input: Vec<u8> = vec![];
    reader
        .by_ref()
        .take(p_file_header.num_vertices as u64 * size_of::<f32>() as u64 * 3)
        .read_to_end(&mut input)?;
    let (_, vertices) =
        count(vec3, p_file_header.num_vertices as usize)(&input).map_err(|e| e.to_owned())?;

    let mut input: Vec<u8> = vec![];
    reader
        .by_ref()
        .take(p_file_header.num_normals as u64 * size_of::<f32>() as u64 * 3)
        .read_to_end(&mut input)?;
    let (_, normals) =
        count(vec3, p_file_header.num_normals as usize)(&input).map_err(|e| e.to_owned())?;

    let mut input: Vec<u8> = vec![];
    reader
        .by_ref()
        .take(p_file_header.num_unk1 as u64 * size_of::<f32>() as u64 * 3)
        .read_to_end(&mut input)?;
    let (_, unk1_array) =
        count(vec3, p_file_header.num_unk1 as usize)(&input).map_err(|e| e.to_owned())?;

    let mut input: Vec<u8> = vec![];
    reader
        .by_ref()
        .take(p_file_header.num_tex_coords as u64 * size_of::<f32>() as u64 * 2)
        .read_to_end(&mut input)?;
    let (_, tex_coords) =
        count(vec2, p_file_header.num_tex_coords as usize)(&input).map_err(|e| e.to_owned())?;

    let mut input: Vec<u8> = vec![];
    reader
        .by_ref()
        .take(p_file_header.num_vertex_colors as u64 * size_of::<u32>() as u64)
        .read_to_end(&mut input)?;
    let (_, vertex_colors) = count(bgra_color, p_file_header.num_vertex_colors as usize)(&input)
        .map_err(|e| e.to_owned())?;

    Ok(PModel::new(
        vertices,
        normals,
        unk1_array,
        tex_coords,
        vertex_colors,
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs::File, io::BufReader};

    #[test]
    fn test() {
        let file = File::open("data/aaac.p").unwrap();
        let buf_reader = BufReader::new(file);
        let polygon_data = parse_p_model(buf_reader).unwrap();

        println!("{:?}", polygon_data);
    }
}
