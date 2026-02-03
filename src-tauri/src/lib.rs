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

#[derive(serde::Serialize, serde::Deserialize)]
struct AnalysisResult {
    id: String,
    sequence: String,
    classification: String,
}

fn analyze_sequence(sequence: &str) -> Result<String, String> {
    let sequence: String = if let Some(pos) = sequence.find("mRNA") {
        sequence[pos + 4..].to_string()
    } else {
        sequence.to_string()
    };

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

#[tauri::command]
fn download_csv_results(results_json: &str) -> Result<String, String> {
    let results: Vec<AnalysisResult> = serde_json::from_str(results_json)
        .map_err(|e| format!("Failed to parse results: {}", e))?;
    
    let mut wtr = csv::Writer::from_writer(vec![]);
    
    // Write header
    wtr.write_record(&["id", "sequence", "classification"])
        .map_err(|e| format!("Failed to write CSV header: {}", e))?;
    
    // Write data
    for result in results {
        wtr.write_record(&[result.id, result.sequence, result.classification])
            .map_err(|e| format!("Failed to write CSV record: {}", e))?;
    }
    
    wtr.flush()
        .map_err(|e| format!("Failed to flush CSV writer: {}", e))?;
    
    let csv_data = String::from_utf8(wtr.into_inner()
        .map_err(|e| format!("Failed to get CSV data: {}", e))?)
        .map_err(|e| format!("Failed to convert CSV to string: {}", e))?;
    
    Ok(csv_data)
}

#[tauri::command]
fn analyze_nat2(path: &str) -> Result<String, String> {
    let path_lower = path.to_lowercase();
    
    if path_lower.ends_with(".csv") {
        // Handle CSV file
        let mut rdr = csv::Reader::from_path(path)
            .map_err(|e| format!("Failed to read CSV file: {}", e))?;
        
        let mut results = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| format!("Failed to read CSV record: {}", e))?;
            
            if record.len() < 2 {
                return Err("CSV must have at least 2 columns: id, sequence".to_string());
            }
            
            let id = record[0].to_string();
            let sequence = record[1].to_string();
            
            let classification = analyze_sequence(&sequence)?;
            
            results.push(AnalysisResult {
                id,
                sequence,
                classification,
            });
        }
        
        // Return JSON for frontend processing
        serde_json::to_string(&results)
            .map_err(|e| format!("Failed to serialize results: {}", e))
    } else {
        // Handle FASTA file (original logic)
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Split content into words and join from index 1 onwards
        let words: Vec<&str> = content.split_whitespace().collect();
        if words.len() <= 1 {
            return Err("File does not contain enough data".to_string());
        }

        let sequence: String = words[1..].iter().map(|s| s.to_string()).collect::<Vec<String>>().join("");

        analyze_sequence(&sequence)
    }
}

#[tauri::command]
fn write_file(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![greet, read_file_start, analyze_nat2, download_csv_results, write_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
