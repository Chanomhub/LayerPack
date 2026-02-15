use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use crate::resolver::LoadedPack;

// ดึงรหัสผ่านมาจาก Environment ตอน build
const SECURITY_KEY: &str = env!("LPACK_SECURITY_KEY");

#[no_mangle]
pub unsafe extern "C" fn ffi_unpack_files(
    key: *const c_char, // [SECURITY] รหัสลับต้องตรงกัน
    pack_path: *const c_char, 
    output_path: *const c_char
) -> c_int {
    if key.is_null() || pack_path.is_null() || output_path.is_null() {
        return -1;
    }

    // ตรวจสอบ Key
    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return -10, // Invalid key format
    };

    if key_str != SECURITY_KEY {
        return -99; // Wrong security key!
    }

    let pack_path_str = match CStr::from_ptr(pack_path).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let output_path_str = match CStr::from_ptr(output_path).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let pack_path = Path::new(pack_path_str);
    let output_path = Path::new(output_path_str);

    match LoadedPack::load(pack_path) {
        Ok(mut loaded) => {
            let files = loaded.file_list();
            for path in files {
                match loaded.read_file(&path) {
                    Ok(content) => {
                         let out_file_path = output_path.join(&path);
                         if let Some(parent) = out_file_path.parent() {
                             if std::fs::create_dir_all(parent).is_err() {
                                 return -3;
                             }
                         }
                         if std::fs::write(&out_file_path, content).is_err() {
                             return -4;
                         }
                    },
                    Err(_) => return -5,
                }
            }
            0
        },
        Err(_) => -6,
    }
}
