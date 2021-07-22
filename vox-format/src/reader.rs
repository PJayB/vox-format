///! Provides functions to read VOX files.
use std::{
    fs::File,
    io::{
        Cursor,
        Read,
        Seek,
    },
    path::Path,
    usize,
};

use byteorder::{
    ReadBytesExt,
    LE,
};
use thiserror::Error;

use crate::{
    chunk::{
        read_main_chunk,
        Chunk,
        ChunkId,
    },
    data::{
        VoxBuffer,
        VoxData,
    },
    types::{
        Palette,
        Vector,
        Voxel,
    },
};

/// Error type returned when reading a VOX file fails.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Expected file header to start with b'VOX ', but got: {got:?}.")]
    InvalidMagic { got: [u8; 4] },

    #[error("Expected MAIN chunk, but read chunk with ID: {0:?}", got.id())]
    ExpectedMainChunk { got: Chunk },

    #[error("Found multiple PACK chunks (at {} and {}).", .chunks[0].offset(), chunks[1].offset())]
    MultiplePackChunks { chunks: [Chunk; 2] },

    #[error("Found {} SIZE chunks, {} XYZI chunks, and PACK said there are {} models.", .size_chunks.len(), .xyzi_chunks.len(), num_models)]
    InvalidNumberOfSizeAndXyziChunks {
        size_chunks: Vec<Chunk>,
        xyzi_chunks: Vec<Chunk>,
        num_models: usize,
    },

    #[error("Found multiple RGBA chunks (at {} and {}).", .chunks[0].offset(), chunks[1].offset())]
    MultipleRgbaChunks { chunks: [Chunk; 2] },

    #[error("Invalid material type: {0}")]
    InvalidMaterial(u8),

    #[error("IO error")]
    Io(#[from] std::io::Error),
}

/// Reads a VOX file from the reader into the [`VoxBuffer`].
pub fn read_vox_into<R: Read + Seek, B: VoxBuffer>(
    mut reader: R,
    buffer: &mut B,
) -> Result<(), Error> {
    let (main_chunk, version) = read_main_chunk(&mut reader)?;

    buffer.set_version(version);

    //print_chunk(&main_chunk, &mut self.reader, 0)?;
    log::trace!("main chunk: {:#?}", main_chunk);

    let mut pack_chunk = None;
    let mut size_chunks = vec![];
    let mut xyzi_chunks = vec![];
    let mut rgba_chunk = None;

    for r in main_chunk.children(&mut reader) {
        let chunk = r?;
        match chunk.id() {
            ChunkId::Pack => {
                if pack_chunk.is_some() {
                    return Err(Error::MultiplePackChunks {
                        chunks: [pack_chunk.take().unwrap(), chunk],
                    });
                }
                pack_chunk = Some(chunk);
            }
            ChunkId::Size => size_chunks.push(chunk),
            ChunkId::Xyzi => xyzi_chunks.push(chunk),
            ChunkId::Rgba => {
                if rgba_chunk.is_some() {
                    return Err(Error::MultipleRgbaChunks {
                        chunks: [rgba_chunk.take().unwrap(), chunk],
                    });
                }
                rgba_chunk = Some(chunk);
            }
            id => log::trace!("Skipping unsupported chunk: {:?}", id),
        }
    }

    // Call `set_palette` first, so the trait impl has the palette data already when
    // reading the voxels.
    if let Some(rgba_chunk) = rgba_chunk {
        log::debug!("read RGBA chunk");
        let palette = Palette::read(&mut rgba_chunk.content(&mut reader)?)?;
        buffer.set_palette(palette);
    }
    else {
        log::debug!("no RGBA chunk found");
    }

    let num_models = pack_chunk
        .map(|pack| Ok::<_, Error>(pack.content(&mut reader)?.read_u32::<LE>()? as usize))
        .transpose()?
        .unwrap_or(1);
    log::trace!("num_models = {}", num_models);

    if num_models != size_chunks.len() || num_models != xyzi_chunks.len() {
        return Err(Error::InvalidNumberOfSizeAndXyziChunks {
            size_chunks,
            xyzi_chunks,
            num_models,
        });
    }
    buffer.set_num_models(num_models);

    for (size_chunk, xyzi_chunk) in size_chunks.into_iter().zip(xyzi_chunks) {
        let model_size = Vector::read(&mut size_chunk.content(&mut reader)?)?;
        log::trace!("model_size = {:?}", model_size);
        buffer.set_model_size(model_size);

        let mut reader = xyzi_chunk.content(&mut reader)?;

        let num_voxels = reader.read_u32::<LE>()?;
        log::trace!("num_voxels = {}", num_voxels);

        for _ in 0..num_voxels {
            let voxel = Voxel::read(&mut reader)?;
            log::trace!("voxel = {:?}", voxel);
            buffer.set_voxel(voxel);
        }
    }

    Ok(())
}

/// Reads a VOX file from a reader into `VoxData`.
pub fn from_reader<R: Read + Seek>(reader: R) -> Result<VoxData, Error> {
    let mut buffer = VoxData::default();
    read_vox_into(reader, &mut buffer)?;
    Ok(buffer)
}

/// Reads a VOX file from a slice into `VoxData`.
pub fn from_slice(slice: &[u8]) -> Result<VoxData, Error> {
    from_reader(Cursor::new(slice))
}

/// Reads a VOX file from the specified path into `VoxData`.
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<VoxData, Error> {
    from_reader(File::open(path)?)
}

#[cfg(test)]
mod tests {
    // TODO: Write some proper test with some better test files.

    use std::io::Cursor;

    use super::read_vox_into;
    use crate::data::VoxData;

    #[test]
    fn it_works_perfectly_as_intended_lol() {
        dotenv::dotenv().ok();
        pretty_env_logger::init();

        let raw = include_bytes!("../../test_files/ore_small.vox");

        let mut buffer = VoxData::default();

        match read_vox_into(Cursor::new(raw), &mut buffer) {
            Ok(()) => {
                println!("{:#?}", buffer);
            }
            Err(e) => panic!("Error: {}", e),
        }
    }
}
