use std::io;
use std::{error::Error, fs::File, io::prelude::*};

use flate2::write::ZlibEncoder;
use flate2::Compression;

pub(crate) struct EtFile {
    file_location: String,
    file_data_comp: Vec<u8>,
    file_size: u32,
    file_size_comp: u32,
    pub file_offset: u32,
}

impl EtFile {
    pub(crate) fn new(
        file_name: Option<String>,
        file_location: String,
    ) -> Result<Self, Box<dyn Error>> {
        if let Some(file_name) = file_name {
            let mut file = File::open(&file_name)?;

            let mut file_data: Vec<u8> = Vec::new();
            file.read_to_end(&mut file_data)?;

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(1));
            encoder.write_all(&file_data)?;

            let file_data_comp = encoder.finish()?;
            let file_size = file.metadata()?.len() as u32;
            let file_size_comp = file_data_comp.len() as u32;

            return Ok(Self {
                file_location: file_location.to_string(),
                file_data_comp,
                file_offset: 0,
                file_size,
                file_size_comp,
            });
        }

        Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "idk")))
    }

    pub(crate) fn get_compressed_data(&self) -> &Vec<u8> {
        &self.file_data_comp
    }

    pub(crate) fn get_file_info(&mut self) -> Vec<u8> {
        let mut data: Vec<u8> = vec![];

        // fill with null
        // FBSTR[256]
        let mut file_location = self.file_location.clone().into_bytes();
        file_location.resize(256, 0);

        data.extend(file_location);
        data.extend(self.file_size_comp.to_le_bytes());
        data.extend(self.file_size.to_le_bytes());
        data.extend(self.file_size_comp.to_le_bytes());
        data.extend(self.file_offset.to_le_bytes());
        data.extend(0u32.to_le_bytes());
        data.extend(0u32.to_le_bytes());
        data.extend([0; 36]);

        data
    }
}
