const { JsLayerPack } = require('./index.js');
const fs = require('fs');
const path = require('path');

// 1. จำลองการสร้างไฟล์ lpack (เราใช้ของที่สร้างจาก cargo run ปกติ)
// แต่ในที่นี้สมมติว่ามีไฟล์ test.lpack อยู่แล้ว หรือเราจะข้ามไป test เลย

// path ไปยังไฟล์ lpack (สมมติว่า user เอามาวางไว้)
const lpackPath = path.resolve('../test_node.lpack');

console.log(`Checking for ${lpackPath}...`);

try {
    const pack = new JsLayerPack(lpackPath);
    
    console.log("=== Load Success ===");
    console.log(`Name: ${pack.name}`);
    console.log(`Author: ${pack.author || "Unknown"}`);
    
    const files = pack.getFileList();
    console.log(`Files found (${files.length}):`, files);

    if (files.length > 0) {
        const firstFile = files[0];
        console.log(`Reading content of ${firstFile}...`);
        const content = pack.readFile(firstFile);
        console.log(`Content length: ${content.length} bytes`);
        console.log(`Content (text): ${content.toString('utf8')}`);
    }

} catch (error) {
    console.error("Error:", error);
}
