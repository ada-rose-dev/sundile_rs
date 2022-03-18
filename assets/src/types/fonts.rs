use std::collections::HashMap;
use std::path::*;
use std::fs::*;
use std::io::Read;
use crate::prelude::*;

pub type FontData = Vec<u8>;

impl RawAsset<Vec<u8>> for FontData {
    /// Loads in the font file as raw bytes.
    fn from_disk(path: &PathBuf) -> Self {
        let mut buffer = Vec::<u8>::new();
        let mut file = File::open(path).unwrap();
        file.read_to_end(&mut buffer).unwrap();
        buffer
    }

    /// Simply returns the byte vector.
    fn to_asset(self, _: &AssetBuilder) -> Vec<u8> {
        self
    }
}

pub struct Mapper {
    map: HashMap<String, FontData>
}
impl Mapper {
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }
}

impl RawAssetMapper for Mapper {
    fn load(&mut self, asset_dir: &PathBuf) {
        crate::util::generic_load::<FontData, FontData>(&mut self.map, asset_dir, "fonts", "ttf");
    }
    fn to_asset_map<'a>(self: Box<Self>, builder: &AssetBuilder) -> AssetMap {
        crate::util::generic_to_asset_map::<FontData, FontData>(self.map, builder)
    }
    fn load_bin_map(&mut self, bin_map: BincodeAssetMap) {
        crate::util::generic_load_bin_map::<FontData, FontData>(&mut self.map, bin_map);
    }
    fn to_bin_map(self: Box<Self>) -> BincodeAssetMap {
        crate::util::generic_to_bin_map::<FontData, FontData>(self.map)
    }
}