use std::{
    convert::TryInto,
    fs::OpenOptions,
    io::{
        Cursor,
        Seek,
        Write,
    },
    path::Path,
};

use byteorder::{
    WriteBytesExt,
    LE,
};
use thiserror::Error;

use crate::{
    chunk::{
        chunk_writer,
        ChunkId,
        ChunkWriter,
    },
    vox::{
        ColorIndex,
        Version,
        VoxData,
    },
    Color,
    Palette,
    Vector,
    Voxel,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Integer overflow")]
    Overflow(#[from] std::num::TryFromIntError),

    #[error("Can't write VOX file with no models.")]
    NoModels,

    /// This is a work-around,since sometimes we want to read VOX files in a chunk-writer closure.
    #[error("Reader error")]
    Reader(#[from] crate::reader::Error),
}

impl Version {
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_u32::<LE>(self.0)?;
        Ok(())
    }
}

impl Vector {
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_i8(self.x)?;
        writer.write_i8(self.y)?;
        writer.write_i8(self.z)?;
        Ok(())
    }
}

impl Voxel {
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        self.point.write(&mut writer)?;
        self.color_index.write(&mut writer)?;
        Ok(())
    }
}

impl Palette {
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        for color in &self.colors[1..] {
            color.write(&mut writer)?;
        }

        Ok(())
    }
}

impl Color {
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        // FIXME: I think color is stored in ABGR format.
        writer.write_u8(self.r)?;
        writer.write_u8(self.g)?;
        writer.write_u8(self.b)?;
        writer.write_u8(self.a)?;
        Ok(())
    }
}

impl ColorIndex {
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_u8(self.0)?;
        Ok(())
    }
}

pub fn write_file_header<W: Write>(mut writer: W, version: Version) -> Result<(), Error> {
    writer.write_all(b"VOX ")?;
    version.write(writer)?;
    Ok(())
}

pub fn main_chunk_writer<W: Write + Seek, F: FnMut(&mut ChunkWriter<W>) -> Result<(), Error>>(
    mut writer: W,
    version: Version,
    f: F,
) -> Result<(), Error> {
    write_file_header(&mut writer, version)?;

    chunk_writer(writer, f, ChunkId::Main)
}

pub fn to_writer<W: Write + Seek>(writer: W, vox: &VoxData) -> Result<(), Error> {
    main_chunk_writer(writer, Version::default(), |chunk_writer| {
        // Write PACK, if there is more than 1 model
        match vox.models.len() {
            0 => return Err(Error::NoModels),
            1 => {}
            n => {
                chunk_writer.child_content_writer(ChunkId::Pack, |writer| {
                    writer.write_u32::<LE>(n.try_into()?)?;
                    Ok(())
                })?;
            }
        }

        // Write models
        for model in &vox.models {
            // Write SIZE chunk
            chunk_writer.child_content_writer(ChunkId::Size, |writer| {
                model.size.write(writer)?;
                Ok(())
            })?;

            // Write XYZI chunk
            chunk_writer.child_content_writer(ChunkId::Xyzi, |mut writer| {
                writer.write_u32::<LE>(model.voxels.len().try_into()?)?;
                for voxel in &model.voxels {
                    voxel.write(&mut writer)?;
                }
                Ok(())
            })?;
        }

        // Write palette
        if !vox.palette.is_default() {
            chunk_writer.child_content_writer(ChunkId::Rgba, |writer| {
                vox.palette.write(writer)?;
                Ok(())
            })?;
        }

        Ok(())
    })
}

pub fn to_vec(vox: &VoxData) -> Result<Vec<u8>, Error> {
    //let mut buf = Vec::with_capacity(vox.size_hint());
    let mut buf = Vec::with_capacity(1024);
    to_writer(Cursor::new(&mut buf), vox)?;
    buf.shrink_to_fit();
    Ok(buf)
}

pub fn to_file<P: AsRef<Path>>(path: P, vox: &VoxData) -> Result<(), Error> {
    to_writer(OpenOptions::new().create(true).write(true).open(path)?, vox)
}

#[cfg(test)]
mod tests {
    use crate::{
        writer::to_file,
        Model,
        Vector,
        VoxData,
        Voxel,
    };

    #[test]
    fn write_vox_from_data() {
        let mut vox = VoxData::default();
        vox.models.push(Model {
            size: Vector::new(3, 3, 1),
            voxels: vec![
                Voxel::new([1, 0, 0], 1),
                Voxel::new([2, 1, 0], 1),
                Voxel::new([0, 2, 0], 1),
                Voxel::new([1, 2, 0], 1),
                Voxel::new([2, 2, 0], 1),
            ],
        });

        to_file("../test_files/glider.vox", &vox).unwrap();
    }
}
