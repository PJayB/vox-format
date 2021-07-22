//! Basic types

use std::{
    collections::HashMap,
    convert::{
        TryFrom,
        TryInto,
    },
    fmt,
    io::{
        Read,
        Write,
    },
    ops::Index,
};

use byteorder::{
    ReadBytesExt,
    WriteBytesExt,
    LE,
};
use thiserror::Error;

use crate::{
    default_palette::DEFAULT_PALETTE,
    reader::Error as ReadError,
    writer::Error as WriteError,
};

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

impl Version {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, ReadError> {
        Ok(Self(reader.read_u32::<LE>()?))
    }

    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), WriteError> {
        writer.write_u32::<LE>(self.0)?;
        Ok(())
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

    pub fn read<R: Read>(mut reader: R) -> Result<Self, ReadError> {
        Ok(Self {
            point: Vector::read(&mut reader)?,
            color_index: ColorIndex::read(&mut reader)?,
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), WriteError> {
        self.point.write(&mut writer)?;
        self.color_index.write(&mut writer)?;
        Ok(())
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
    pub fn read<R: Read>(mut reader: R) -> Result<Self, ReadError> {
        Ok(Self {
            x: reader.read_i8()?,
            y: reader.read_i8()?,
            z: reader.read_i8()?,
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), WriteError> {
        writer.write_i8(self.x)?;
        writer.write_i8(self.y)?;
        writer.write_i8(self.z)?;
        Ok(())
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

    pub fn get(&self, color_index: ColorIndex) -> Color {
        self.colors[color_index.0 as usize]
    }

    // TODO: Return a struct here
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (ColorIndex, Color)> + 'a {
        self.colors
            .iter()
            .enumerate()
            .map(|(i, color)| (ColorIndex(i as u8), *color))
    }

    pub fn read<R: Read>(mut reader: R) -> Result<Self, ReadError> {
        let mut palette = Palette::default();

        for i in 0..255 {
            palette.colors[i + 1] = Color::read(&mut reader)?;
        }

        Ok(palette)
    }

    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), WriteError> {
        for color in &self.colors[1..] {
            color.write(&mut writer)?;
        }

        Ok(())
    }
}

impl Index<ColorIndex> for Palette {
    type Output = Color;

    fn index(&self, color_index: ColorIndex) -> &Self::Output {
        &self.colors[color_index.0 as usize]
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

impl Color {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, ReadError> {
        // FIXME: I think color is stored in ABGR format.
        Ok(Self {
            r: reader.read_u8()?,
            g: reader.read_u8()?,
            b: reader.read_u8()?,
            a: reader.read_u8()?,
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), WriteError> {
        // FIXME: I think color is stored in ABGR format.
        writer.write_u8(self.r)?;
        writer.write_u8(self.g)?;
        writer.write_u8(self.b)?;
        writer.write_u8(self.a)?;
        Ok(())
    }
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

impl ColorIndex {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, ReadError> {
        Ok(Self(reader.read_u8()?))
    }

    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), WriteError> {
        writer.write_u8(self.0)?;
        Ok(())
    }
}

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

impl Material {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, ReadError> {
        let ty = MaterialType::read(&mut reader)?;
        let weight = reader.read_f32::<LE>()?;
        let flags = reader.read_u32::<LE>()?;

        let plastic = (flags & 1 != 0)
            .then(|| reader.read_f32::<LE>())
            .transpose()?;
        let roughness = (flags & 2 != 0)
            .then(|| reader.read_f32::<LE>())
            .transpose()?;
        let specular = (flags & 4 != 0)
            .then(|| reader.read_f32::<LE>())
            .transpose()?;
        let ior = (flags & 8 != 0)
            .then(|| reader.read_f32::<LE>())
            .transpose()?;
        let attenuation = (flags & 16 != 0)
            .then(|| reader.read_f32::<LE>())
            .transpose()?;
        let power = (flags & 32 != 0)
            .then(|| reader.read_f32::<LE>())
            .transpose()?;
        let glow = (flags & 64 != 0)
            .then(|| reader.read_f32::<LE>())
            .transpose()?;

        Ok(Material {
            ty,
            weight,
            plastic,
            roughness,
            specular,
            ior,
            attenuation,
            power,
            glow,
            is_total_power: (flags & 128 != 0),
        })
    }
}

#[derive(Debug, Error)]
#[error("Invalid material type: {0}")]
pub struct MaterialTryFromError(pub u8);

impl TryFrom<u8> for MaterialType {
    type Error = MaterialTryFromError;

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        match x {
            0 => Ok(MaterialType::Diffuse),
            1 => Ok(MaterialType::Metal),
            2 => Ok(MaterialType::Glass),
            3 => Ok(MaterialType::Emissive),
            x => Err(MaterialTryFromError(x)),
        }
    }
}

impl From<MaterialType> for u8 {
    fn from(ty: MaterialType) -> Self {
        match ty {
            MaterialType::Diffuse => 0,
            MaterialType::Metal => 1,
            MaterialType::Glass => 2,
            MaterialType::Emissive => 3,
        }
    }
}

impl MaterialType {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, ReadError> {
        reader
            .read_u8()?
            .try_into()
            .map_err(|e: MaterialTryFromError| ReadError::InvalidMaterial(e.0))
    }

    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), WriteError> {
        writer.write_u8((*self).into())?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialType {
    Diffuse,
    Metal,
    Glass,
    Emissive,
}
