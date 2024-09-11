# rust-basic-ransomw



# 🔐 Dynamic AES File Encryptor/Decryptor

Welcome to the **Dynamic AES File Encryptor/Decryptor** repository! This project provides a Rust-based application that demonstrates advanced techniques in file encryption and decryption using AES-256. The application includes dynamic library loading for AES functions, basic obfuscation techniques, and timing evasion strategies.

## 📦 Features

- **Dynamic AES Loading**: Load AES functions at runtime to evade static analysis.
- **File Encryption/Decryption**: Encrypt and decrypt files using AES-256.
- **Obfuscation**: Basic XOR-based obfuscation for strings and file operations.
- **Timing Evasion**: Randomized delays to avoid detection.

## 🚀 Getting Started

### Prerequisites

- **Rust**: Make sure you have Rust installed. You can install it from [rust-lang.org](https://www.rust-lang.org/tools/install).
- **Windows API**: This project uses Windows API calls, so it is intended for Windows environments.

### Installation

1. **Clone the repository**:

    ```bash
    git clone https://github.com/yourusername/dynamic-aes-encryptor.git
    cd dynamic-aes-encryptor
    ```

2. **Build the project**:

    ```bash
    cargo build --release
    ```

3. **Prepare the AES DLL**: Ensure you have an `aes.dll` with the functions `AES_encrypt` and `AES_decrypt` available in your working directory or system path.

### Usage

1. **Generate and Save Key**:

    ```bash
    cargo run
    ```

    This will generate a key and IV, save them to a file, and proceed with either encryption or decryption based on a random action.

2. **Encrypt/Decrypt Files**:

    Modify the `main` function or pass parameters to encrypt or decrypt files in your specified directory.

3. **Check Ransom Note**:

    After execution, a ransom note will be created in the specified directory.

## ⚙️ Configuration

- **File Paths**: Modify the `folder_path` and `ransom_note` paths in the `main` function to suit your environment.
- **XOR Key**: Adjust the XOR key used for obfuscation if needed.

## 🛠️ Development

- **Testing**: Ensure to test in a controlled environment to avoid any accidental data loss.
- **Contributions**: Feel free to fork the repository, submit issues, or propose changes.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👥 Author

- **Your Name** - [yourusername](https://github.com/yourusername)

## 📧 Contact

For any questions or concerns, please reach out to [your.email@example.com](mailto:your.email@example.com).

---

*This project is for educational and research purposes only. Use responsibly and ethically.*
