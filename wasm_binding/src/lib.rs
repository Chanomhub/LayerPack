use wasm_bindgen::prelude::*;
use layer_pack::resolver::LoadedPack;

#[wasm_bindgen]
pub struct WasmLayerPack {
    inner: LoadedPack,
}

#[wasm_bindgen]
impl WasmLayerPack {
    // Constructor: รับข้อมูลเป็น Uint8Array (หรือ Vec<u8> ใน Rust)
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>) -> Result<WasmLayerPack, JsValue> {
        // ใช้ load_from_memory ที่เราเพิ่งเพิ่มใน Core
        let pack = LoadedPack::load_from_memory(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to load pack: {}", e)))?;
        
        Ok(WasmLayerPack { inner: pack })
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.manifest.name.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn author(&self) -> Option<String> {
        self.inner.manifest.author.clone()
    }

    #[wasm_bindgen]
    pub fn get_file_list(&self) -> Vec<String> {
        self.inner.file_list()
    }

    #[wasm_bindgen]
    pub fn read_file(&mut self, path: String) -> Result<Vec<u8>, JsValue> {
        self.inner.read_file(&path)
            .map_err(|e| JsValue::from_str(&format!("Failed to read file: {}", e)))
    }
}
