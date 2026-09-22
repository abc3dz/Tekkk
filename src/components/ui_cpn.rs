// ui_cpn.rs
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs; // ใช้ตัวอ่านไฟล์ปกติ

#[derive(Resource)]
pub struct GameFonts {
    pub abc3dz: Handle<Font>,
}

pub fn load_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameFonts {
        abc3dz: asset_server.load("fonts/abc3dz.ttf"),
    });
}

// --- ระบบ JSON Localization (แบบอ่านไฟล์ตรงๆ ไม่ต้องง้อ AssetLoader) ---

#[derive(Deserialize, Debug, Clone)]
pub struct LangData {
    pub en: HashMap<String, String>,
    pub th: HashMap<String, String>,
    pub nth: HashMap<String, String>,
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Language {
    English,
    #[default]
    Thai,
    NorthernThai,
}

#[derive(Resource)]
pub struct Localization {
    pub current: Language,
    pub data: LangData, // <-- ตัด Option< > และ Handle ทิ้งให้หมด
}

impl Localization {
    // แก้ Syntax <'a> และ => ให้ถูกต้อง (ห้ามมีช่องว่างใน =>)
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        // ตัด if let Some(data) ออก เพราะ self.data เป็น LangData ตรงๆ แล้ว ไม่ใช่ Option
        let map = match self.current {
            Language::English => &self.data.en,
            Language::Thai => &self.data.th,
            Language::NorthernThai => &self.data.nth,
        };
        
        // ดึงค่าจาก map โดยตรง
        if let Some(text) = map.get(key) {
            return text.as_str();
        }
        
        // ถ้าไม่เจอ key นี้ใน JSON ให้คืนค่า key กลับไปเลย
        key
    }
}

// ระบบ setup เริ่มต้น (อ่านไฟล์ทีเดียวจบ ไม่ต้องมีระบบเช็คสถานะให้เมื่อย)
pub fn setup_localization(mut commands: Commands) {
    let content = fs::read_to_string("assets/lang.json")
        .expect("Failed to read assets/lang.json. Make sure the file exists in the assets folder.");
    
    let data: LangData = serde_json::from_str(&content)
        .expect("Failed to parse lang.json. Check your JSON syntax.");
    
    commands.insert_resource(Localization {
        current: Language::English,
        data,
    });
}

// Resource สำหรับเก็บภาษาก่อนหน้า เพื่อเช็คว่ามีการเปลี่ยนภาษาหรือไม่
#[derive(Resource, Default)]
pub struct PreviousLanguage(pub Language);