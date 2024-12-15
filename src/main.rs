use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use csv::Writer;
use rusqlite::Connection;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug)]
struct VisitedUrl {
    url: String,
    title: String,
    visit_time: DateTime<Utc>,
    history_file: String,
    browser: String,
}

fn find_chrome_history_files() -> Vec<PathBuf> {
    let home = std::env::var("HOME").expect("Could not find HOME directory");
    let chrome_dir = PathBuf::from(home).join("Library/Application Support/Google/Chrome");
    
    let mut history_files = Vec::new();
    
    for entry in WalkDir::new(chrome_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "History" {
            history_files.push(entry.path().to_owned());
        }
    }
    
    history_files
}

fn find_safari_history_file() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let history_path = PathBuf::from(home)
        .join("Library/Safari/History.db");
    
    if history_path.exists() && history_path.metadata().is_ok() {
        Some(history_path)
    } else {
        None
    }
}

fn find_firefox_history_files() -> Vec<PathBuf> {
    let home = std::env::var("HOME").expect("Could not find HOME directory");
    let firefox_dir = PathBuf::from(home).join("Library/Application Support/Firefox/Profiles");
    
    let mut history_files = Vec::new();
    
    for entry in WalkDir::new(firefox_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "places.sqlite" {
            history_files.push(entry.path().to_owned());
        }
    }
    
    history_files
}

fn read_chrome_history(db_path: &PathBuf) -> Result<Vec<VisitedUrl>> {
    let temp_path = std::env::temp_dir().join("chrome_history_temp.db");
    std::fs::copy(db_path, &temp_path)?;
    
    let conn = Connection::open(&temp_path)
        .with_context(|| format!("Failed to open database at {:?}", temp_path))?;

    let mut stmt = conn.prepare(
        "SELECT urls.url, urls.title, visits.visit_time 
         FROM urls JOIN visits ON urls.id = visits.url 
         ORDER BY visits.visit_time DESC"
    )?;

    let history_path = db_path.to_string_lossy().to_string();
    
    let visited_urls = stmt.query_map([], |row| {
        let timestamp: i64 = row.get(2)?;
        let chrome_epoch = Utc.with_ymd_and_hms(1601, 1, 1, 0, 0, 0)
            .earliest()
            .expect("Invalid chrome epoch date");
        let duration = chrono::Duration::microseconds(timestamp);
        let visit_time = chrome_epoch + duration;

        Ok(VisitedUrl {
            url: row.get(0)?,
            title: row.get(1)?,
            visit_time,
            history_file: history_path.clone(),
            browser: "Chrome".to_string(),
        })
    })?;

    let results: Result<Vec<_>, _> = visited_urls.collect();
    let urls = results.with_context(|| "Failed to read Chrome history entries")?;
    
    std::fs::remove_file(temp_path)?;
    
    Ok(urls)
}

fn read_safari_history(db_path: &PathBuf) -> Result<Vec<VisitedUrl>> {
    let temp_path = std::env::temp_dir().join("safari_history_temp.db");
    std::fs::copy(db_path, &temp_path)?;
    
    let conn = Connection::open(&temp_path)
        .with_context(|| format!("Failed to open Safari database at {:?}", temp_path))?;

    let mut stmt = conn.prepare(
        "SELECT history_items.url, history_visits.visit_time, 
                history_visits.title
         FROM history_items 
         JOIN history_visits ON history_items.id = history_visits.history_item
         ORDER BY history_visits.visit_time DESC"
    )?;

    let history_path = db_path.to_string_lossy().to_string();
    
    let visited_urls = stmt.query_map([], |row| {
        let timestamp: f64 = row.get(1)?;
        let safari_epoch = Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0)
            .earliest()
            .expect("Invalid safari epoch date");
        let seconds = timestamp as i64;
        let visit_time = safari_epoch + chrono::Duration::seconds(seconds);

        Ok(VisitedUrl {
            url: row.get(0)?,
            title: row.get(3).unwrap_or_default(),
            visit_time,
            history_file: history_path.clone(),
            browser: "Safari".to_string(),
        })
    })?;

    let results: Result<Vec<_>, _> = visited_urls.collect();
    let urls = results.with_context(|| "Failed to read Safari history entries")?;
    
    std::fs::remove_file(temp_path)?;
    
    Ok(urls)
}

fn read_firefox_history(db_path: &PathBuf) -> Result<Vec<VisitedUrl>> {
    let temp_path = std::env::temp_dir().join("firefox_history_temp.db");
    std::fs::copy(db_path, &temp_path)?;
    
    let conn = Connection::open(&temp_path)
        .with_context(|| format!("Failed to open Firefox database at {:?}", temp_path))?;

    let mut stmt = conn.prepare(
        "SELECT p.url, COALESCE(p.title, p.url) as title, h.visit_date 
         FROM moz_places p
         JOIN moz_historyvisits h ON p.id = h.place_id
         WHERE p.url NOT LIKE 'about:%'
         AND p.url NOT LIKE 'place:%'
         ORDER BY h.visit_date DESC"
    )?;

    let history_path = db_path.to_string_lossy().to_string();
    
    let visited_urls = stmt.query_map([], |row| {
        let timestamp: i64 = row.get(2)?;
        let firefox_epoch = Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0)
            .earliest()
            .expect("Invalid firefox epoch date");
        let duration = chrono::Duration::microseconds(timestamp);
        let visit_time = firefox_epoch + duration;

        Ok(VisitedUrl {
            url: row.get(0)?,
            title: row.get(1)?,
            visit_time,
            history_file: history_path.clone(),
            browser: "Firefox".to_string(),
        })
    })?;

    let results: Result<Vec<_>, _> = visited_urls.collect();
    let urls = results.with_context(|| "Failed to read Firefox history entries")?;
    
    std::fs::remove_file(temp_path)?;
    
    Ok(urls)
}

fn export_to_csv(all_visits: &[VisitedUrl], output_path: &str) -> Result<()> {
    let mut writer = Writer::from_path(output_path)?;
    
    writer.write_record(&["Timestamp", "URL", "Title", "History File", "Browser"])?;
    
    for visit in all_visits {
        writer.write_record(&[
            &visit.visit_time.to_rfc3339(),
            &visit.url,
            &visit.title,
            &visit.history_file,
            &visit.browser,
        ])?;
    }
    
    writer.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let mut all_visits = Vec::new();

    // Collect Chrome history
    let chrome_files = find_chrome_history_files();
    for history_file in chrome_files {
        println!("\nReading Chrome history from: {:?}", history_file);
        match read_chrome_history(&history_file) {
            Ok(urls) => {
                println!("Found {} Chrome URLs", urls.len());
                all_visits.extend(urls);
            }
            Err(err) => {
                eprintln!("Error reading Chrome history: {}", err);
            }
        }
    }

    // Collect Firefox history
    let firefox_files = find_firefox_history_files();
    for history_file in firefox_files {
        println!("\nReading Firefox history from: {:?}", history_file);
        match read_firefox_history(&history_file) {
            Ok(urls) => {
                println!("Found {} Firefox URLs", urls.len());
                all_visits.extend(urls);
            }
            Err(err) => {
                eprintln!("Error reading Firefox history: {}", err);
            }
        }
    }

    // Collect Safari history
    if let Some(safari_history) = find_safari_history_file() {
        println!("\nReading Safari history from: {:?}", safari_history);
        match read_safari_history(&safari_history) {
            Ok(urls) => {
                println!("Found {} Safari URLs", urls.len());
                all_visits.extend(urls);
            }
            Err(err) => {
                eprintln!("Error reading Safari history: {}.", err);
                eprintln!("Note: Reading Safari history requires Full Disk Access permission.");
                eprintln!("To grant access: System Settings -> Privacy & Security -> Full Disk Access");
            }
        }
    } else {
        println!("\nSafari history file not accessible. Skipping Safari history.");
        println!("Note: Reading Safari history requires Full Disk Access permission.");
        println!("To grant access: System Settings -> Privacy & Security -> Full Disk Access");
    }

    if all_visits.is_empty() {
        println!("No browser history found!");
        return Ok(());
    }

    // Sort all visits by timestamp
    all_visits.sort_by(|a, b| b.visit_time.cmp(&a.visit_time));

    // Generate filename with timestamp
    let now = chrono::Local::now();
    let output_path = format!("browser_history_{}.csv", 
        now.format("%Y-%m-%d_%H-%M-%S"));
    
    export_to_csv(&all_visits, &output_path)?;
    println!("\nExported {} visits to {}", all_visits.len(), output_path);

    Ok(())
} 