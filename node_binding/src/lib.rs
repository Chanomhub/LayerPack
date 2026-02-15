use napi_derive::napi;
use napi::{Result, Error, Status};
use napi::bindgen_prelude::Buffer;
use layer_pack::resolver::LoadedPack;

// ประกาศ struct ให้ JS เรียกใช้ได้
#[napi]
pub struct JsLayerPack {
    // เก็บ inner struct ของ Rust ไว้ข้างใน (ไม่ต้อง expose ให้ JS เห็นตรงๆ)
    inner: LoadedPack,
}

// ดึงรหัสผ่านมาจาก Environment ตอน build
const SECURITY_KEY: &str = env!("LPACK_SECURITY_KEY");

#[napi]
impl JsLayerPack {
    // Constructor: โหลดไฟล์ pack
    #[napi(constructor)]
    pub fn new(path: String, key: String) -> Result<Self> {
        // [SECURITY] ตรวจสอบรหัสลับป้องกันการนำไปใช้ผิด
        if key != SECURITY_KEY {
             return Err(Error::new(Status::InvalidArg, "Invalid security key".to_string()));
        }

        let pack = LoadedPack::load(&path)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to load pack: {}", e)))?;
        
        Ok(JsLayerPack { inner: pack })
    }

    // Method: อ่านชื่อ pack (Get Name)
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.manifest.name.clone()
    }

    // Method: อ่านผู้แต่ง (Get Author)
    #[napi(getter)]
    pub fn author(&self) -> Option<String> {
        self.inner.manifest.author.clone()
    }

    // Method: ดึงรายชื่อไฟล์ทั้งหมด
    #[napi]
    pub fn get_file_list(&self) -> Vec<String> {
        self.inner.file_list()
    }

    // Method: อ่านไฟล์ออกมาเป็น Buffer
    #[napi]
    pub fn read_file(&mut self, path: String) -> Result<Buffer> {
        let data = self.inner.read_file(&path)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to read file: {}", e)))?;
        Ok(data.into())
    }
}
