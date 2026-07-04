use std::{env, fs, path::Path, time::{SystemTime, UNIX_EPOCH}};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let Some(env_path) = find_dotenv(Path::new(&manifest_dir)) else { return };
    // Tell cargo to re-run this script whenever .env changes
    println!("cargo:rerun-if-changed={}", env_path.display());
    let Ok(content) = fs::read_to_string(&env_path) else { return };

    let mut days: i64 = 30;
    let mut hours: i64 = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
        let Some((key, val)) = trimmed.split_once('=') else { continue };
        let key = key.trim();
        let val = val.trim().trim_matches('"').trim_matches('\'');
        match key {
            "SAMGRAHA_EXPIRY_DAYS"  => { if let Ok(v) = val.parse() { days = v; } }
            "SAMGRAHA_EXPIRY_HOURS" => { if let Ok(v) = val.parse() { hours = v; } }
            _ => {}
        }
    }

    if days == -1 {
        // No expiry: option_env!("SAMGRAHA_EXPIRY") returns None, check_expiry() is a no-op
        return;
    }
    let hours = hours.max(0);

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let expiry_unix = now_secs + days * 86400 + hours * 3600;
    let expiry_rfc = unix_to_rfc3339(expiry_unix);
    println!("cargo:rustc-env=SAMGRAHA_EXPIRY={expiry_rfc}");
}

fn find_dotenv(start: &Path) -> Option<std::path::PathBuf> {
    let mut dir = start;
    loop {
        let candidate = dir.join(".env");
        if candidate.exists() { return Some(candidate); }
        dir = dir.parent()?;
    }
}

fn unix_to_rfc3339(unix_secs: i64) -> String {
    let secs_in_day = 86400i64;
    let time = ((unix_secs % secs_in_day) + secs_in_day) % secs_in_day;
    let day_num = (unix_secs - time) / secs_in_day;
    let h = time / 3600;
    let m = (time % 3600) / 60;
    let s = time % 60;
    let (year, month, day) = civil_from_days(day_num);
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

// Howard Hinnant's civil_from_days — correct for all proleptic Gregorian dates
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}
