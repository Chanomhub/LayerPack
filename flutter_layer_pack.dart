import 'dart:ffi' as ffi;
import 'dart:io';
import 'package:ffi/ffi.dart';

// การตั้งค่าสำหรับการใช้งาน:
// 1. Compile Rust: `cargo build --release` (สำหรับ Desktop) หรือใช้ `cargo-ndk` (สำหรับ Android)
// 2. iOS: ต้อง link ไฟล์ .a หรือ .framework ใน Xcode และใช้ ffi.DynamicLibrary.executable()
// 3. Android: วาง liblayer_pack.so ไว้ที่ android/app/src/main/jniLibs/<arch>/

class LayerPack {
  // Singleton Pattern: เพื่อไม่ให้โหลด Library ซ้ำซ้อน
  static final LayerPack _instance = LayerPack._internal();
  factory LayerPack() => _instance;

  late ffi.DynamicLibrary _lib;
  late int Function(ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>) _unpackFiles;

  // รหัสลับ (แนะนำให้ทำ Obfuscation ในขั้นตอน build)
  static const String _secretKey = "LAYER_PACK_SECRET_2026";

  LayerPack._internal() {
    _lib = _loadLibrary();
    
    // Lookup ฟังก์ชันจาก Native
    _unpackFiles = _lib
        .lookup<ffi.NativeFunction<ffi.Int32 Function(
            ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>)>>('ffi_unpack_files')
        .asFunction();
  }

  ffi.DynamicLibrary _loadLibrary() {
    if (Platform.isAndroid) {
      return ffi.DynamicLibrary.open('liblayer_pack.so');
    }
    if (Platform.isIOS) {
      // iOS มักจะใช้วิธี Static Linking หรือโหลดผ่าน Symbol ในตัว Executable
      return ffi.DynamicLibrary.executable();
    }
    if (Platform.isMacOS) {
      // สำหรับ macOS Desktop
      return ffi.DynamicLibrary.open('liblayer_pack.dylib');
    }
    if (Platform.isWindows) {
      return ffi.DynamicLibrary.open('layer_pack.dll');
    }
    if (Platform.isLinux) {
      return ffi.DynamicLibrary.open('liblayer_pack.so');
    }
    throw UnsupportedError('Platform นี้ยังไม่รองรับ: ${Platform.operatingSystem}');
  }

  /// ฟังก์ชันแตกไฟล์หลัก
  /// [packPath]: ที่อยู่ไฟล์ .lpack
  /// [outputPath]: โฟลเดอร์ปลายทาง
  /// คืนค่า 0 หากสำเร็จ, -99 หาก Key ผิด, และค่าติดลบอื่นๆ ตาม Error ใน Rust
  int unpack(String packPath, String outputPath) {
    final pKey = _secretKey.toNativeUtf8();
    final pPack = packPath.toNativeUtf8();
    final pOut = outputPath.toNativeUtf8();
    
    try {
      return _unpackFiles(pKey, pPack, pOut);
    } catch (e) {
      // จัดการ Error กรณีเรียก Native ไม่ได้
      return -999;
    } finally {
      // สำคัญ: ต้องคืน Memory ทุกครั้งหลังใช้ FFI
      malloc.free(pKey);
      malloc.free(pPack);
      malloc.free(pOut);
    }
  }
}
