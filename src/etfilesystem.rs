use crate::etfile::EtFile;
use crate::utils;

use glob::glob;
use std::io::{ErrorKind, SeekFrom};
use std::os::unix::prelude::FileExt;
use std::path::Path;
use std::{error::Error, fs::File, io::prelude::*};
use std::{fs, io};

const HEADER_MAGIC: &str = "EyedentityGames Packing File 0.1";

enum OpenMode {
    Read,
    Write,
}

pub struct EtFileSystem {
    mode: OpenMode,
    file: File,
    file_name: String,
    file_count: u32,
    offset: u32,
    files: Vec<EtFile>,
}

impl EtFileSystem {
    pub fn write(file_name: &str) -> Self {
        let mut pak = Self {
            mode: OpenMode::Write,
            file: File::create(&file_name).unwrap(),
            file_name: file_name.to_string(),
            file_count: 0,
            offset: 0,
            files: Vec::new(),
        };

        // write dummy header
        pak.write_header().expect("Cannot write file header");

        pak
    }

    pub fn read(file_name: &str) -> Self {
        let mut pak = Self {
            mode: OpenMode::Read,
            file: File::open(&file_name).unwrap(),
            file_name: file_name.to_string(),
            file_count: 0,
            offset: 0,
            files: Vec::new(),
        };

        let mut buf = [0u8; 4];

        // seek to skip magic version (256 bytes)
        // and version (4bits)
        pak.file.seek(SeekFrom::Start(260)).unwrap();
        pak.file.read_exact(&mut buf).unwrap();
        pak.file_count = u32::from_le_bytes(buf);

        // current position is 264
        pak.file.read_exact(&mut buf).unwrap();
        pak.offset = u32::from_le_bytes(buf);

        let mut current_offset = 0;
        for _ in 0..pak.file_count {
            pak.file
                .seek(SeekFrom::Start((pak.offset + current_offset) as u64))
                .unwrap();

            let mut location = [0u8; 256];
            pak.file.read_exact(&mut location).unwrap();

            // convert utf-8 to string
            // split when byte is 0
            let iter: Vec<_> = location.split(|byte| byte == &0).collect();
            let location = unsafe { String::from_utf8_unchecked(iter[0].to_vec()) };

            // temporary buf to store 4bytes of value
            let mut buf = [0; 4];

            // new etfile object
            let mut file = EtFile::new(None, &location)
                .unwrap_or_else(|_| panic!("Cannot create etfile with location: {}", location));

            // filesizecomp
            pak.file.read_exact(&mut buf).unwrap();
            file.comp_size = u32::from_le_bytes(buf);
            // filesize
            pak.file.read_exact(&mut buf).unwrap();
            file.file_size = u32::from_le_bytes(buf);
            // allocsize
            pak.file.read_exact(&mut buf).unwrap();
            file.alloc_size = u32::from_le_bytes(buf);
            // offset
            pak.file.read_exact(&mut buf).unwrap();
            file.data_offset = u32::from_le_bytes(buf);

            let mut filedatacomp = vec![];
            filedatacomp.resize(file.alloc_size as usize, 0);

            pak.file
                .read_exact_at(&mut filedatacomp, file.data_offset as u64)
                .unwrap();

            file.comp_data = filedatacomp;

            pak.files.push(file);

            current_offset += 316;
        }

        pak
    }

    pub fn unpack(&self, out_dir: Option<String>) -> Result<(), Box<dyn Error>> {
        // out directory
        // by default the pak name
        let out_dir: &str =
            &out_dir.unwrap_or_else(|| self.file_name[..self.file_name.len() - 4].to_string());

        for file in &self.files {
            let file_location = utils::to_normal_path(&file.path); // path of the file. Windows Path by default

            // absolute path of the file
            // out_dir/file_location
            let absolute_path = Path::new(out_dir).join(&file_location);

            // make the directory
            fs::create_dir_all(&absolute_path.parent().unwrap())?;

            fs::write(absolute_path, &file.get_decompressed_data())?;
        }

        Ok(())
    }

    pub fn add_file(
        &mut self,
        file_name: String,
        file_location: String,
    ) -> Result<(), Box<dyn Error>> {
        let mut file_location = file_location;

        if file_location.starts_with('\\') {
            file_location = format!("\\{}", file_location);
        };

        self.files
            .push(EtFile::new(Some(&file_name), &file_location)?);

        Ok(())
    }

    pub fn add_files(&mut self, directory: &str) -> Result<(), Box<dyn Error>> {
        //TODO: add files inside folder
        if !fs::metadata(&directory).unwrap().is_dir() {
            let directory_error =
                io::Error::new(ErrorKind::InvalidInput, String::from("Not a directory"));

            return Err(Box::new(directory_error));
        }

        for file in glob(&format!("{}/**/*.*", &directory)).expect("Failed to read glob pattern") {
            let relative_path = format!(
                "\\{}",
                file.as_ref()
                    .unwrap()
                    .strip_prefix(&directory)
                    .unwrap()
                    .display()
                    .to_string()
                    .replace("/", "\\")
            );

            let etfile = EtFile::new(file?.to_str(), &relative_path)?;
            self.files.push(etfile);
        }

        Ok(())
    }

    pub fn close_file_system(&mut self) {
        if let OpenMode::Write = self.mode {
            self.file.seek(SeekFrom::Start(1024)).unwrap();
            self.write_data().unwrap();
            self.write_footer().unwrap();
        }
    }

    fn write_header(&mut self) -> Result<(), Box<dyn Error>> {
        self.file.write_all(HEADER_MAGIC.as_bytes())?;
        self.file.write_all(&[0; 224])?;
        self.file.write_all(&11u32.to_le_bytes())?;
        self.file.write_all(&self.file_count.to_le_bytes())?;
        self.file.write_all(&self.offset.to_le_bytes())?;
        self.file.write_all(&0u32.to_le_bytes())?;
        self.file.write_all(&[0; 752])?;

        Ok(())
    }

    fn rewrite_header(&mut self) -> Result<(), Box<dyn Error>> {
        self.file_count = self.files.len() as u32;
        self.offset = self.file.seek(SeekFrom::Current(0))? as u32;

        self.file.seek(SeekFrom::Start(256 + 4))?;
        self.file.write_all(&self.file_count.to_le_bytes())?;
        self.file.write_all(&self.offset.to_le_bytes())?;

        self.file.seek(SeekFrom::Start(self.offset as u64))?;

        Ok(())
    }

    fn write_data(&mut self) -> Result<(), Box<dyn Error>> {
        for file in &mut self.files {
            file.data_offset = self.file.seek(SeekFrom::Current(0))? as u32;
            self.file.write_all(file.get_compressed_data())?;
        }

        Ok(())
    }

    fn write_footer(&mut self) -> Result<(), Box<dyn Error>> {
        self.rewrite_header()?;

        for file in &mut self.files {
            self.file.write_all(&file.get_file_info())?;
        }

        Ok(())
    }
}
