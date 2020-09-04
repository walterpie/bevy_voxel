#[cfg(feature = "savedata")]
use serde::{Deserialize, Serialize};

use bevy::prelude::*;

#[cfg(feature = "savedata")]
use crate::serialize::SerDePartialEq;

use crate::{
    collections::lod_tree::Voxel,
    render::entity::{Face, MeshPart, VoxelExt},
    world::{Chunk, Map},
};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shade {
    pub top: f32,
    pub bottom: f32,
    pub front: f32,
    pub back: f32,
    pub left: f32,
    pub right: f32,
}

impl Shade {
    pub fn zero() -> Self {
        Shade {
            top: 0.0,
            bottom: 0.0,
            front: 0.0,
            back: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }
}

impl Default for Shade {
    fn default() -> Self {
        Shade {
            top: 1.0,
            bottom: 1.0,
            front: 1.0,
            back: 1.0,
            left: 1.0,
            right: 1.0,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Block {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub shade: Shade,
    pub color: Color,
}

#[cfg(feature = "savedata")]
impl SerDePartialEq<Self> for Block {
    fn serde_eq(&self, other: &Self) -> bool {
        self.color == other.color
    }
}

impl Voxel for Block {
    fn average(data: &[Self]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        let mut color = Color::rgba(0.0, 0.0, 0.0, 0.0);
        let mut top = 0.0_f32;
        let mut bottom = 0.0_f32;
        let mut left = 0.0_f32;
        let mut right = 0.0_f32;
        let mut front = 0.0_f32;
        let mut back = 0.0_f32;

        for block in data {
            top = top.max(block.shade.top);
            bottom = bottom.max(block.shade.bottom);
            left = left.max(block.shade.left);
            right = right.max(block.shade.right);
            front = front.max(block.shade.front);
            back = back.max(block.shade.back);
            color += block.color;
        }

        color *= (data.len() as f32).recip();

        Some(Self {
            color,
            shade: Shade {
                top,
                bottom,
                left,
                right,
                front,
                back,
            },
        })
    }

    fn can_merge(&self) -> bool {
        true
    }
}

impl VoxelExt for Block {
    fn mesh(
        &self,
        coords: (i32, i32, i32),
        map: &Map<Self>,
        chunk: &Chunk<Self>,
        width: usize,
    ) -> MeshPart {
        let mut positions = Vec::new();
        let mut shades = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();

        let mut n = 0;
        if let Some((p, s, c)) = generate_top_side(
            self,
            map,
            chunk,
            coords,
            width,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) = generate_bottom_side(
            self,
            map,
            chunk,
            coords,
            width,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) = generate_front_side(
            self,
            map,
            chunk,
            coords,
            width,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) = generate_back_side(
            self,
            map,
            chunk,
            coords,
            width,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) = generate_left_side(
            self,
            map,
            chunk,
            coords,
            width,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) = generate_right_side(
            self,
            map,
            chunk,
            coords,
            width,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        MeshPart {
            positions,
            shades,
            colors,
            indices,
        }
    }

    fn set_shade(&mut self, face: Face, light: f32) {
        match face {
            Face::Top => self.shade.top = light,
            Face::Bottom => self.shade.bottom = light,
            Face::Front => self.shade.front = light,
            Face::Back => self.shade.back = light,
            Face::Left => self.shade.left = light,
            Face::Right => self.shade.right = light,
        }
    }

    fn shade(&mut self, face: Face) -> Option<f32> {
        match face {
            Face::Top => Some(self.shade.top),
            Face::Bottom => Some(self.shade.bottom),
            Face::Front => Some(self.shade.front),
            Face::Back => Some(self.shade.back),
            Face::Left => Some(self.shade.left),
            Face::Right => Some(self.shade.right),
        }
    }
}

fn generate_front_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dx in 0..width {
        for dy in 0..width {
            let render = if z + width >= cw {
                let (cx, cy, cz) = chunk.position();
                let cz = cz + cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((x + dx, y + dy, 0))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x + dx, y + dy, z + width))
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x, y, z + size],
                        [x + size, y, z + size],
                        [x + size, y + size, z + size],
                        [x, y + size, z + size],
                    ],
                    [
                        block.shade.front,
                        block.shade.front,
                        block.shade.front,
                        block.shade.front,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_back_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dx in 0..width {
        for dy in 0..width {
            let render = if z - 1 < 0 {
                let (cx, cy, cz) = chunk.position();
                let cz = cz - cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((x + dx, y + dy, cw - 1))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x + dx, y + dy, z - 1))
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x, y + size, z],
                        [x + size, y + size, z],
                        [x + size, y, z],
                        [x, y, z],
                    ],
                    [
                        block.shade.back,
                        block.shade.back,
                        block.shade.back,
                        block.shade.back,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_right_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dy in 0..width {
        for dz in 0..width {
            let render = if x - 1 < 0 {
                let (cx, cy, cz) = chunk.position();
                let cx = cx - cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((cw - 1, y + dy, z + dz))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x - 1, y + dy, z + dz))
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x, y, z],
                        [x, y, z + size],
                        [x, y + size, z + size],
                        [x, y + size, z],
                    ],
                    [
                        block.shade.right,
                        block.shade.right,
                        block.shade.right,
                        block.shade.right,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_left_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dy in 0..width {
        for dz in 0..width {
            let render = if x + width >= cw {
                let (cx, cy, cz) = chunk.position();
                let cx = cx + cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((0, y + dy, z + dz))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x + width, y + dy, z + dz))
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x + size, y, z],
                        [x + size, y + size, z],
                        [x + size, y + size, z + size],
                        [x + size, y, z + size],
                    ],
                    [
                        block.shade.left,
                        block.shade.left,
                        block.shade.left,
                        block.shade.left,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_top_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dx in 0..width {
        for dz in 0..width {
            let render = if y + width >= cw {
                let (cx, cy, cz) = chunk.position();
                let cy = cy + cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((x + dx, 0, z + dz))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x + dx, y + width, z + dz))
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x + size, y + size, z],
                        [x, y + size, z],
                        [x, y + size, z + size],
                        [x + size, y + size, z + size],
                    ],
                    [
                        block.shade.top,
                        block.shade.top,
                        block.shade.top,
                        block.shade.top,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_bottom_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dx in 0..width {
        for dz in 0..width {
            let render = if y - 1 < 0 {
                let (cx, cy, cz) = chunk.position();
                let cy = cy - cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((x + dx, cw - 1, z + dz))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x + dx, y - 1, z + dz))
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x + size, y, z + size],
                        [x, y, z + size],
                        [x, y, z],
                        [x + size, y, z],
                    ],
                    [
                        block.shade.bottom,
                        block.shade.bottom,
                        block.shade.bottom,
                        block.shade.bottom,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}
