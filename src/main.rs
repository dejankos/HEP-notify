use chrono::{Duration, Local};
use clap::Parser;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use scraper::{Html, Selector};
use std::env;

#[derive(Parser, Debug)]
#[command(name = "hep-outage-checker")]
#[command(about = "Check for HEP power outages and send notifications", long_about = None)]
struct Args {
    #[arg(long, help = "Print outage data to console instead of sending email")]
    dry_run: bool,

    #[arg(long, short = 'f', help = "Filter outages by location or street (partial match). Shows all if not provided")]
    filter: Option<String>,
}

#[derive(Debug)]
struct PowerOutage {
    date: String,
    location: String,
    street: String,
    time: String,
    note: String,
}

fn fetch_page(date: &str, city: &str, office: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "https://www.hep.hr/ods/bez-struje/19?dp={}&el={}&datum={}",
        city, office, date
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let response = client.get(&url).send()?;
    let html = response.text()?;

    Ok(html)
}

fn parse_outages(html: &str) -> Result<Vec<PowerOutage>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    let mut outages = Vec::new();

    // Find the date from the heading
    let date_selector = Selector::parse("h3").unwrap();
    let date = document
        .select(&date_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();

    // Each outage is in a section with hr separators
    // Look for sections with "Mjesto:" pattern
    let text = document.root_element().text().collect::<Vec<_>>().join(" ");

    // Split by horizontal rules or look for "Mjesto:" patterns
    let lines: Vec<&str> = text.split('\n').map(|s| s.trim()).collect();

    let mut current_outage: Option<PowerOutage> = None;
    let mut expect_time_next = false;

    for line in lines {
        if line.starts_with("Mjesto:") {
            if let Some(outage) = current_outage.take() {
                outages.push(outage);
            }
            current_outage = Some(PowerOutage {
                date: date.clone(),
                location: line.replace("Mjesto:", "").trim().to_string(),
                street: String::new(),
                time: String::new(),
                note: String::new(),
            });
            expect_time_next = false;
        } else if line.starts_with("Ulica:") {
            if let Some(ref mut outage) = current_outage {
                outage.street = line.replace("Ulica:", "").trim().to_string();
            }
            expect_time_next = false;
        } else if line.starts_with("Očekivano trajanje:") {
            // The time might be on the same line or the next line
            let time_on_same_line = line.replace("Očekivano trajanje:", "").trim().to_string();
            if !time_on_same_line.is_empty() {
                if let Some(ref mut outage) = current_outage {
                    outage.time = time_on_same_line;
                }
                expect_time_next = false;
            } else {
                // Time is on the next line
                expect_time_next = true;
            }
        } else if line.starts_with("Napomena:") {
            if let Some(ref mut outage) = current_outage {
                outage.note = line.replace("Napomena:", "").trim().to_string();
            }
            expect_time_next = false;
        } else if expect_time_next && !line.is_empty() && line.contains("-") {
            // This should be the time line (e.g., "09:00 - 11:30")
            if let Some(ref mut outage) = current_outage {
                outage.time = line.to_string();
            }
            expect_time_next = false;
        }
    }

    // Don't forget the last one
    if let Some(outage) = current_outage {
        outages.push(outage);
    }

    Ok(outages)
}

fn send_email(
    outages: &[PowerOutage],
    to_email: &str,
    from_email: &str,
    smtp_username: &str,
    smtp_password: &str,
    smtp_server: &str,
    filter: &Option<String>,
    city: &str,
    office: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut body = String::from("⚡ PLANNED POWER OUTAGES IN YOUR AREA ⚡\n\n");
    body.push_str(&format!("Found {} scheduled outage(s):\n\n", outages.len()));
    
    for (i, outage) in outages.iter().enumerate() {
        body.push_str(&format!("━━━ OUTAGE {} ━━━\n", i + 1));
        body.push_str(&format!("📅 Date: {}\n", outage.date));
        body.push_str(&format!("📍 Location: {}\n", outage.location));
        body.push_str(&format!("🛣️  Street: {}\n", outage.street));
        body.push_str(&format!("⏰ Time: {}\n", outage.time));
        if !outage.note.is_empty() {
            body.push_str(&format!("📝 Note: {}\n", outage.note));
        }
        body.push('\n');
    }
    
    body.push_str("\n---\n");
    body.push_str("This is an automated notification from HEP Outage Checker\n");
    body.push_str(&format!(
        "Source: https://www.hep.hr/ods/bez-struje/19?dp={}&el={}\n",
        city, office
    ));

    let subject = match filter {
        Some(location) => format!("⚡ Power Outage Alert - {}", location),
        None => "⚡ Power Outage Alert".to_string(),
    };

    let email = Message::builder()
        .from(from_email.parse()?)
        .to(to_email.parse()?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body)?;
    
    let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
    
    let mailer = SmtpTransport::relay(smtp_server)?
        .credentials(creds)
        .build();
    
    mailer.send(&email)?;
    
    Ok(())
}

fn filter_outages<'a>(outages: &'a [PowerOutage], filter: &Option<String>) -> Vec<&'a PowerOutage> {
    match filter {
        Some(filter_text) => {
            let filter_lower = filter_text.to_lowercase();
            outages
                .iter()
                .filter(|outage| {
                    outage.location.to_lowercase().contains(&filter_lower)
                        || outage.street.to_lowercase().contains(&filter_lower)
                })
                .collect()
        }
        None => outages.iter().collect(),
    }
}

fn print_outages_detailed(outages: &[PowerOutage], city: &str, office: &str) {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║        ⚡ DRY RUN - POWER OUTAGE DATA (NO EMAIL SENT) ⚡        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    if outages.is_empty() {
        println!("✅ No outages found in the next 7 days.\n");
        return;
    }

    println!("Found {} scheduled outage(s):\n", outages.len());

    for (i, outage) in outages.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━ OUTAGE {} ━━━━━━━━━━━━━━━━━", i + 1);
        println!("📅 Date:     {}", outage.date);
        println!("📍 Location: {}", outage.location);
        println!("🛣️  Street:   {}", outage.street);
        println!("⏰ Time:     {}", outage.time);
        if !outage.note.is_empty() {
            println!("📝 Note:     {}", outage.note);
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "Source: https://www.hep.hr/ods/bez-struje/19?dp={}&el={}",
        city, office
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Get environment variables (only required if not in dry-run mode)
    let to_email = if args.dry_run {
        String::new()
    } else {
        env::var("TO_EMAIL").expect("TO_EMAIL must be set")
    };
    let from_email = if args.dry_run {
        String::new()
    } else {
        env::var("FROM_EMAIL").expect("FROM_EMAIL must be set")
    };
    let smtp_username = if args.dry_run {
        String::new()
    } else {
        env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set")
    };
    let smtp_password = if args.dry_run {
        String::new()
    } else {
        env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set")
    };
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".to_string());

    // Get HEP location parameters
    let hep_city = env::var("HEP_CITY").expect("HEP_CITY must be set");
    let hep_office = env::var("HEP_OFFICE").expect("HEP_OFFICE must be set");

    println!("🔍 HEP Outage Checker starting...");
    if args.dry_run {
        println!("🔍 Mode: DRY RUN (no email will be sent)");
    } else {
        println!("📧 Will notify: {}", to_email);
    }
    
    // Check today and the next 7 days
    let today = Local::now();
    let mut all_outages = Vec::new();
    
    for days_ahead in 0..=7 {
        let check_date = today + Duration::days(days_ahead);
        let date_str = check_date.format("%d.%m.%Y").to_string();
        
        println!("\n📅 Checking date: {}", date_str);

        match fetch_page(&date_str, &hep_city, &hep_office) {
            Ok(html) => {
                match parse_outages(&html) {
                    Ok(outages) => {
                        if !outages.is_empty() {
                            println!("   ⚠️  Found {} outage(s)", outages.len());
                            for outage in &outages {
                                println!("      - {}: {}", outage.location, outage.time);
                            }
                            all_outages.extend(outages);
                        } else {
                            println!("   ✅ No outages scheduled");
                        }
                    }
                    Err(e) => {
                        eprintln!("   ❌ Error parsing outages: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("   ❌ Error fetching page: {}", e);
            }
        }
        
        // Small delay to be nice to the server
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    // Apply filter if provided
    let filtered_outages = filter_outages(&all_outages, &args.filter);

    if args.filter.is_some() {
        println!(
            "\n🔍 Filter applied: '{}' - {} of {} outage(s) match",
            args.filter.as_ref().unwrap(),
            filtered_outages.len(),
            all_outages.len()
        );
    }

    if args.dry_run {
        // Convert Vec<&PowerOutage> to Vec<PowerOutage> for printing
        let outages_to_print: Vec<PowerOutage> = filtered_outages
            .iter()
            .map(|&outage| PowerOutage {
                date: outage.date.clone(),
                location: outage.location.clone(),
                street: outage.street.clone(),
                time: outage.time.clone(),
                note: outage.note.clone(),
            })
            .collect();
        print_outages_detailed(&outages_to_print, &hep_city, &hep_office);
    } else {
        if !filtered_outages.is_empty() {
            println!("\n📧 Sending email notification...");
            let outages_to_send: Vec<PowerOutage> = filtered_outages
                .iter()
                .map(|&outage| PowerOutage {
                    date: outage.date.clone(),
                    location: outage.location.clone(),
                    street: outage.street.clone(),
                    time: outage.time.clone(),
                    note: outage.note.clone(),
                })
                .collect();
            match send_email(
                &outages_to_send,
                &to_email,
                &from_email,
                &smtp_username,
                &smtp_password,
                &smtp_server,
                &args.filter,
                &hep_city,
                &hep_office,
            ) {
                Ok(_) => println!("✅ Email sent successfully!"),
                Err(e) => eprintln!("❌ Failed to send email: {}", e),
            }
        } else {
            if args.filter.is_some() {
                println!("\n✅ No matching outages found. No email sent.");
            } else {
                println!("\n✅ No outages found in the next 7 days. No email sent.");
            }
        }
    }
    
    Ok(())
}
