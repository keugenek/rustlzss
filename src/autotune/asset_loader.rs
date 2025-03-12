use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Represents different types of game assets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Texture,
    Model,
    LevelData,
    Audio,
    Animation,
    Unknown,
}

/// Holds information about a game asset
#[derive(Debug)]
pub struct AssetInfo {
    /// The file path of the asset
    pub path: PathBuf,
    /// The type of the asset
    pub asset_type: AssetType,
    /// The size of the asset in bytes
    pub size: usize,
    /// Cached data of the asset (if loaded)
    data: Option<Vec<u8>>,
}

impl AssetInfo {
    /// Create a new AssetInfo from a file path
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let metadata = fs::metadata(&path)?;
        let size = metadata.len() as usize;
        let asset_type = identify_asset_type(&path);
        
        Ok(AssetInfo {
            path,
            asset_type,
            size,
            data: None,
        })
    }
    
    /// Get the data of the asset, loading it if needed
    pub fn data(&mut self) -> io::Result<&[u8]> {
        if self.data.is_none() {
            let mut file = File::open(&self.path)?;
            let mut buffer = Vec::with_capacity(self.size);
            file.read_to_end(&mut buffer)?;
            self.data = Some(buffer);
        }
        
        Ok(self.data.as_ref().unwrap())
    }
    
    /// Get the file extension of the asset
    pub fn extension(&self) -> Option<String> {
        self.path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }
    
    /// Get the filename of the asset without path
    pub fn filename(&self) -> String {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}

/// Identifies the type of a game asset based on its extension
fn identify_asset_type(path: &Path) -> AssetType {
    let extension = match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => ext.to_lowercase(),
        None => return AssetType::Unknown,
    };
    
    match extension.as_str() {
        // Texture formats
        "png" | "jpg" | "jpeg" | "tga" | "dds" | "ktx" | "bmp" | "hdr" | "exr" | "psd" => AssetType::Texture,
        
        // Model formats
        "fbx" | "obj" | "gltf" | "glb" | "dae" | "blend" | "3ds" | "stl" | "ply" => AssetType::Model,
        
        // Level data formats (usually custom formats, but some common ones)
        "map" | "level" | "umap" | "unity" | "scene" => AssetType::LevelData,
        
        // Audio formats
        "wav" | "mp3" | "ogg" | "flac" | "m4a" | "aiff" => AssetType::Audio,
        
        // Animation formats
        "anim" | "animation" | "anm" | "smd" => AssetType::Animation,
        
        // Unknown
        _ => AssetType::Unknown,
    }
}

/// Recursively scans a directory for game assets
pub fn scan_directory<P: AsRef<Path>>(dir: P, max_files: Option<usize>) -> io::Result<Vec<AssetInfo>> {
    let mut assets = Vec::new();
    let mut dirs_to_visit = vec![dir.as_ref().to_path_buf()];
    
    while let Some(current_dir) = dirs_to_visit.pop() {
        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                dirs_to_visit.push(path);
            } else if path.is_file() {
                if let Ok(asset) = AssetInfo::new(&path) {
                    // Only add recognized asset types or files larger than 1KB
                    if asset.asset_type != AssetType::Unknown || asset.size > 1024 {
                        assets.push(asset);
                        
                        // Check if we've reached the maximum number of files
                        if let Some(max) = max_files {
                            if assets.len() >= max {
                                return Ok(assets);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(assets)
}