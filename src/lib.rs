use std::io::{self, SeekFrom};
use std::{error::Error, fs::File, io::prelude::*};

use flate2::write::ZlibEncoder;
use flate2::Compression;

const HEADER_MAGIC: &str = "EyedentityGames Packing File 0.1";

struct EtFile {
    file_location: String,
    file_data_comp: Vec<u8>,
    file_size: u32,
    file_size_comp: u32,
    pub file_offset: u32,
}

impl EtFile {
    fn new(file_name: Option<String>, file_location: String) -> Result<Self, Box<dyn Error>> {
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

    fn get_compressed_data(&self) -> &Vec<u8> {
        &self.file_data_comp
    }

    fn get_file_info(&mut self) -> Vec<u8> {
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
        //TODO: error handling if the file doesn't exist
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
