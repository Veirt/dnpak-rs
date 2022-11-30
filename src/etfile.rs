use std::path::Path;
use std::{error::Error, fs::File, io::prelude::*};
use std::{fmt, fs};

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::utils;

#[derive(Debug)]
pub struct EtFile {
    pub(crate) path: String,
    pub(crate) comp_data: Vec<u8>,
    pub(crate) file_size: u32,
    pub(crate) comp_size: u32,
    pub(crate) data_offset: u32,
    pub(crate) alloc_size: u32,
}

impl fmt::Display for EtFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl EtFile {
    pub(crate) fn new(file_name: Option<&str>, path: &str) -> Result<Self, Box<dyn Error>> {
        if let Some(file_name) = file_name {
            let mut file = File::open(file_name)?;

            let mut file_data: Vec<u8> = Vec::new();
            file.read_to_end(&mut file_data)?;

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(1));
            encoder.write_all(&file_data)?;

            let comp_data = encoder.finish()?;
            let file_size = file.metadata()?.len() as u32;
            let comp_size = comp_data.len() as u32;

            return Ok(Self {
                path: path.to_string(),
                comp_data,
                data_offset: 0,
                file_size,
                comp_size,
                alloc_size: 0,
            });
        }

        Ok(Self {
            path: path.to_string(),
            comp_data: Vec::new(),
            data_offset: 0,
            file_size: 0,
            comp_size: 0,
            alloc_size: 0,
        })
    }

    pub fn unpack(&self, out_dir: &str) -> Result<String, Box<dyn Error>> {
        let file_location = utils::to_normal_path(&self.path); // path of the file. Windows Path by default

        // absolute path of the file
        // out_dir/file_location
        let absolute_path = Path::new(out_dir).join(&file_location);

        // make the directory
        fs::create_dir_all(&*absolute_path.parent().unwrap())?;

        fs::write(absolute_path, &self.get_decompressed_data())?;

        Ok(file_location)
    }

    pub(crate) fn get_compressed_data(&self) -> &Vec<u8> {
        &self.comp_data
    }

    pub(crate) fn get_decompressed_data(&self) -> Vec<u8> {
        let mut decoder = ZlibDecoder::new(&*self.comp_data);

        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();
        buf
    }

    pub(crate) fn get_file_info(&mut self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        // fill with null
        // FBSTR[256]
        let mut file_location = self.path.clone().into_bytes();
        file_location.resize(256, 0);

        data.extend(file_location);
        data.extend(self.comp_size.to_le_bytes());
        data.extend(self.file_size.to_le_bytes());
        data.extend(self.comp_size.to_le_bytes());
        data.extend(self.data_offset.to_le_bytes());
        data.extend(0u32.to_le_bytes());
        data.extend(0u32.to_le_bytes());
        data.extend([0; 36]);

        data
    }
}

#[cfg(test)]
mod tests {
    use super::EtFile;
    use std::{error::Error, fs};

    const FILE: &str = "./tests/data/version.cfg";

    #[test]
    #[should_panic]
    fn test_invalid_path() {
        let _etfile =
            EtFile::new(Some("./invalid-peko.cfg"), "/invalid.cfg").expect("Cannot find the path");
    }

    #[test]
    fn test_get_decompressed_data() -> Result<(), Box<dyn Error>> {
        let etfile = EtFile::new(Some(FILE), "/version.cfg").expect("Cannot create a new EtFile");

        let decompressed = String::from_utf8(etfile.get_decompressed_data())?;

        assert_eq!("Version 7", decompressed);

        Ok(())
    }

    #[test]
    fn test_unpack() -> Result<(), Box<dyn Error>> {
        let etfile = EtFile::new(Some(FILE), "/version.cfg").expect("Cannot create a new EtFile");

        etfile
            .unpack("./tests/temp/unpack")
            .expect("Cannot unpack EtFile");

        let data_before: String = fs::read_to_string(FILE)?.parse()?;
        let data_after: String = fs::read_to_string("./tests/temp/unpack/version.cfg")?.parse()?;

        assert_eq!(data_before, data_after);

        // cleanup
        fs::remove_dir_all("./tests/temp/unpack").unwrap();
        Ok(())
    }
}
