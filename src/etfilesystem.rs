use crate::etfile::EtFile;

use std::io::SeekFrom;
use std::{error::Error, fs::File, io::prelude::*};

const HEADER_MAGIC: &str = "EyedentityGames Packing File 0.1";

pub struct EtFileSystem {
    file: File,
    file_count: u32,
    file_offset: u32,
    files: Vec<EtFile>,
}

impl EtFileSystem {
    pub fn new(file_name: String) -> Self {
        let mut pak = Self {
            file: File::create(&file_name).unwrap(),
            file_count: 0,
            file_offset: 0,
            files: vec![],
        };

        pak.write_header().unwrap();

        pak
    }

    pub fn add_file(
        &mut self,
        file_name: String,
        file_location: String,
    ) -> Result<(), Box<dyn Error>> {
        //TODO: error handling when the file doesn't exist
        self.files
            .push(EtFile::new(Some(file_name), file_location)?);

        Ok(())
    }

    pub fn close_file_system(&mut self) {
        self.file.seek(SeekFrom::Start(1024)).unwrap();
        self.write_data().unwrap();
        self.write_footer().unwrap();
        drop(&self.file);
    }

    fn write_header(&mut self) -> Result<(), Box<dyn Error>> {
        self.file.write(HEADER_MAGIC.as_bytes())?;
        self.file.write(&[0; 224])?;
        self.file.write(&11u32.to_le_bytes())?;
        self.file.write(&self.file_count.to_le_bytes())?;
        self.file.write(&self.file_offset.to_le_bytes())?;
        self.file.write(&0u32.to_le_bytes())?;
        self.file.write(&[0; 752])?;

        Ok(())
    }

    fn rewrite_header(&mut self) -> Result<(), Box<dyn Error>> {
        self.file_count = self.files.len() as u32;
        self.file_offset = self.file.seek(SeekFrom::Current(0))? as u32;

        self.file.seek(SeekFrom::Start(256 + 4))?;
        self.file.write(&self.file_count.to_le_bytes())?;
        self.file.write(&self.file_offset.to_le_bytes())?;

        self.file.seek(SeekFrom::Start(self.file_offset as u64))?;

        Ok(())
    }

    fn write_data(&mut self) -> Result<(), Box<dyn Error>> {
        for file in &mut self.files {
            file.file_offset = self.file.seek(SeekFrom::Current(0))? as u32;
            self.file.write(file.get_compressed_data())?;
        }

        Ok(())
    }

    fn write_footer(&mut self) -> Result<(), Box<dyn Error>> {
        self.rewrite_header()?;

        for file in &mut self.files {
            self.file.write(&file.get_file_info()[..])?;
        }

        Ok(())
    }
}
