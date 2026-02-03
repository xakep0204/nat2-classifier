// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn read_file_start(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .map(|content| {
            let first_two = content.chars().take(2).collect::<String>();
            first_two
        })
}

#[tauri::command]
fn analyze_nat2(path: &str) -> Result<String, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Split content into words and join from index 1 onwards
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.len() <= 1 {
        return Err("File does not contain enough data".to_string());
    }

    let sequence: String = words[1..].iter().map(|s| s.to_string()).collect::<Vec<String>>().join("");

    let sequence: String = if let Some(pos) = sequence.find("mRNA") {
        sequence[pos + 4..].to_string()
    } else {
        sequence
    };

    println!("Debug - sequence: {}", sequence);

    if sequence.len() < 857 {
        return Err(format!("Sequence too short: {} characters, need at least 857", sequence.len()));
    }

    let mut slow = 0;

    // Check each position (0-based indexing)
    if sequence.chars().nth(190) == Some('A') {
        slow += 1;
    }
    if sequence.chars().nth(340) == Some('C') {
        slow += 1;
    }
    if sequence.chars().nth(589) == Some('A') {
        slow += 2;
    }
    if sequence.chars().nth(802) == Some('G') {
        slow += 0; // This doesn't add to slow count
    }
    if sequence.chars().nth(856) == Some('A') {
        slow += 2;
    }

    let result = match slow {
        0 => "FAST ACETYLATOR: Normal NAT2 activity. Standard INH dosing acceptable.",
        1 => "INTERMEDIATE ACETYLATOR: Moderately reduced NAT2 activity. Monitor liver function during INH therapy. Consider dose adjustment if needed.",
        _ => "SLOW ACETYLATOR: High risk of INH-induced liver damage. Recommend: (1) Dose reduction, (2) Weekly liver function monitoring, (3) Consider alternative TB treatment if possible."
    };

    Ok(result.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![greet, read_file_start, analyze_nat2])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
