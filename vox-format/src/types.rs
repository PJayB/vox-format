//! Basic types

use std::{
    collections::HashMap,
    fmt,
    ops::Index,
};

use crate::default_palette::DEFAULT_PALETTE;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(pub u32);

impl Default for Version {
    fn default() -> Self {
        Self(150)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    pub size: Vector,
    pub voxels: Vec<Voxel>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Voxel {
    pub point: Vector,
    pub color_index: ColorIndex,
}

impl Voxel {
    pub fn new(point: impl Into<Vector>, color_index: impl Into<ColorIndex>) -> Self {
        Self {
            point: point.into(),
            color_index: color_index.into(),
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vector {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

impl Vector {
    pub fn new(x: i8, y: i8, z: i8) -> Self {
        Self { x, y, z }
    }
}

impl From<[i8; 3]> for Vector {
    fn from(v: [i8; 3]) -> Self {
        Self::new(v[0], v[1], v[2])
    }
}

impl From<Vector> for [i8; 3] {
    fn from(v: Vector) -> Self {
        [v.x, v.y, v.z]
    }
}

impl fmt::Debug for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[derive(Clone, Debug)]
pub struct Palette {
    pub colors: [Color; 256],
}

impl Default for Palette {
    fn default() -> Self {
        DEFAULT_PALETTE.clone()
    }
}

impl Palette {
    pub fn is_default(&self) -> bool {
        self.colors == DEFAULT_PALETTE.colors
    }

    // TODO: Return a struct here
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (ColorIndex, Color)> + 'a {
        self.colors
            .iter()
            .enumerate()
            .map(|(i, color)| (ColorIndex(i as u8), *color))
    }
}

impl Index<ColorIndex> for Palette {
    type Output = Color;

    fn index(&self, index: ColorIndex) -> &Self::Output {
        &self.colors[index.0 as usize]
    }
}

#[derive(Clone, Debug, Default)]
pub struct MaterialPalette {
    /// TODO: Does the material ID correspond to a ColorIndex?
    materials: HashMap<ColorIndex, Material>,
}

impl MaterialPalette {
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }

    pub fn get(&self, material_id: ColorIndex) -> Option<&Material> {
        self.materials.get(&material_id)
    }

    // TODO: Return a struct here
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (ColorIndex, &'a Material)> + 'a {
        self.materials
            .iter()
            .map(|(color_index, material)| (*color_index, material))
    }

    pub fn insert(&mut self, material_id: ColorIndex, material: Material) {
        self.materials.insert(material_id, material);
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<Color> for [u8; 4] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

impl From<[u8; 4]> for Color {
    fn from(color: [u8; 4]) -> Self {
        Self {
            r: color[0],
            g: color[0],
            b: color[0],
            a: color[0],
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorIndex(pub u8);

impl From<u8> for ColorIndex {
    fn from(x: u8) -> Self {
        // I don't think this is invalid
        /*if x == 255 {
            panic!("Invalid color index: 255");
        }*/
        Self(x)
    }
}

impl From<ColorIndex> for u8 {
    fn from(x: ColorIndex) -> Self {
        x.0
    }
}

impl fmt::Display for ColorIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct Material {
    pub ty: MaterialType,
    pub weight: f32,
    pub plastic: Option<f32>,
    pub roughness: Option<f32>,
    pub specular: Option<f32>,
    pub ior: Option<f32>,
    pub attenuation: Option<f32>,
    pub power: Option<f32>,
    pub glow: Option<f32>,
    pub is_total_power: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialType {
    Diffuse,
    Metal,
    Glass,
    Emissive,
}
