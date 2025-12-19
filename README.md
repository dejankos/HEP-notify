# HEP Power Outage Checker

Automated checker for HEP ODS (Croatian electricity company) planned power outages. Runs twice daily on GitHub Actions and sends email notifications when outages are detected in your area.

## Features

- 🔍 Checks HEP website for planned outages in your configured area
- 📅 Looks ahead 7 days
- 🎯 **Location filtering** - Get notified only for specific streets or areas
- 📧 Email notifications only when outages are found
- ⏰ Runs automatically twice per day (7 AM and 7 PM CET)
- 🧪 Dry-run mode for testing without sending emails
- 🆓 Completely free using GitHub Actions
- 🦀 Written in Rust for reliability and speed

## Setup Instructions

### 1. Fork or Create Repository

1. Fork this repository or create a new repository on GitHub (can be private)
2. If creating new, upload all files from this project

### 2. Setup Email (Gmail Example)

#### Option A: Gmail with App Password (Recommended)

1. Go to your Google Account settings
2. Enable 2-Factor Authentication if not already enabled
3. Go to Security → 2-Step Verification → App passwords
4. Generate a new app password for "Mail"
5. Save this 16-character password

Your settings will be:
- `SMTP_SERVER`: `smtp.gmail.com`
- `SMTP_USERNAME`: your full Gmail address (e.g., `youremail@gmail.com`)
- `SMTP_PASSWORD`: the 16-character app password
- `FROM_EMAIL`: your Gmail address
- `TO_EMAIL`: where you want to receive notifications

#### Option B: SendGrid (Alternative)

1. Sign up for free SendGrid account (100 emails/day)
2. Create an API key
3. Your settings:
   - `SMTP_SERVER`: `smtp.sendgrid.net`
   - `SMTP_USERNAME`: `apikey` (literally the word "apikey")
   - `SMTP_PASSWORD`: your SendGrid API key
   - `FROM_EMAIL`: your verified sender email
   - `TO_EMAIL`: where you want notifications

### 3. Find Your HEP Area Parameters

1. Visit [HEP ODS Outages Page](https://www.hep.hr/ods/bez-struje/19)
2. Select your distribution area (e.g., Pula, Zagreb, Split)
3. Select your electricity office
4. Look at the URL - it will be something like:
   ```
   https://www.hep.hr/ods/bez-struje/19?dp=pula&el=128&datum=...
   ```
5. Note the values:
   - `dp=pula` → your `HEP_CITY` is `pula`
   - `el=128` → your `HEP_OFFICE` is `128`

### 4. Add GitHub Secrets

1. Go to your repository on GitHub
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret** and add each of these:

#### Required Secrets

| Name | Value | Example |
|------|-------|---------|
| `TO_EMAIL` | Your notification email | `your-email@gmail.com` |
| `FROM_EMAIL` | Sender email | `your-email@gmail.com` |
| `SMTP_USERNAME` | SMTP username | `your-email@gmail.com` |
| `SMTP_PASSWORD` | App password or API key | `abcd efgh ijkl mnop` |
| `SMTP_SERVER` | SMTP server address | `smtp.gmail.com` |
| `HEP_CITY` | Distribution area code | `pula` |
| `HEP_OFFICE` | Electricity office number | `128` |

#### Optional Secrets

| Name | Value | Example | Description |
|------|-------|---------|-------------|
| `FILTER_LOCATION` | Street or area name | `Valentići` | Only get notified for outages matching this location. Leave empty for all outages. |

### 5. Enable GitHub Actions

1. Go to **Actions** tab in your repository
2. If prompted, click "I understand my workflows, go ahead and enable them"
3. GitHub Actions is now enabled

### 6. Test the Setup

#### Manual Test
1. Go to **Actions** tab
2. Click on "Check HEP Power Outages" workflow
3. Click **Run workflow** → **Run workflow**
4. Wait for it to complete (~30 seconds)
5. Check the logs to see if it worked
6. Check your email for notifications (if any outages were found)

#### Check the Schedule
- The workflow will now run automatically at:
  - 7:00 AM CET (Central European Time)
  - 7:00 PM CET
- You can adjust the schedule in `.github/workflows/check-outages.yml`

## Location Filtering

The `FILTER_LOCATION` secret allows you to receive notifications only for specific areas or streets:

- **Partial matching**: `Valen` matches "Valentići", "Valentin", etc.
- **Case-insensitive**: `valen`, `VALEN`, or `Valen` all work the same
- **Searches both location and street fields**
- **Leave empty for all outages** in your configured area

### Example Email (With Filter)

When `FILTER_LOCATION` is set to `Valentići`, you'll receive emails like this:

```
Subject: ⚡ Power Outage Alert - Valentići

⚡ PLANNED POWER OUTAGES IN YOUR AREA ⚡

Found 2 scheduled outage(s):

━━━ OUTAGE 1 ━━━
📅 Date: 20.12.2025., petak
📍 Location: Valentići
🛣️  Street: cijela naselja
⏰ Time: 09:00 - 11:30
📝 Note: u slučaju lošeg vremena radovi se mogu odgoditi

━━━ OUTAGE 2 ━━━
📅 Date: 22.12.2025., nedjelja
📍 Location: Valentići
🛣️  Street: Istarska ulica 12-45
⏰ Time: 08:00 - 12:00


---
This is an automated notification from HEP Outage Checker
Source: https://www.hep.hr/ods/bez-struje/19?dp=pula&el=128
```

## Local Testing

### Dry-Run Mode (No Email Sent)

Test the checker without sending emails:

```bash
# Set required environment variables
export HEP_CITY="pula"
export HEP_OFFICE="128"

# Run in dry-run mode (prints to console, no email)
cargo run -- --dry-run

# Test with filtering
cargo run -- --dry-run --filter "Valentići"
```

### Normal Mode (Sends Email)

```bash
# Set all environment variables
export TO_EMAIL="your-email@gmail.com"
export FROM_EMAIL="your-email@gmail.com"
export SMTP_USERNAME="your-email@gmail.com"
export SMTP_PASSWORD="your-app-password"
export SMTP_SERVER="smtp.gmail.com"
export HEP_CITY="pula"
export HEP_OFFICE="128"

# Run normally (sends email if outages found)
cargo run --release

# With filtering
cargo run --release -- --filter "Valentići"
```

### Available CLI Options

```bash
# Show help
cargo run -- --help

# Dry-run mode (no email sent, prints to console)
cargo run -- --dry-run

# Filter by location/street
cargo run -- --filter "Valentići"

# Combine options
cargo run -- --dry-run --filter "Valen"
```

## Customization

### Change Check Frequency

Edit `.github/workflows/check-outages.yml`:

```yaml
schedule:
  - cron: '0 6 * * *'   # 7 AM CET (6 AM UTC)
  - cron: '0 18 * * *'  # 7 PM CET (6 PM UTC)
```

Cron syntax: `minute hour day month weekday`

Examples:
- `'0 8 * * *'` - Once daily at 9 AM CET
- `'0 */6 * * *'` - Every 6 hours
- `'0 8,20 * * *'` - 9 AM and 9 PM CET

### Change Look-ahead Days

In `src/main.rs`, change this line:

```rust
for days_ahead in 0..=7 {  // Currently checks 7 days ahead
```

Change `7` to however many days you want to check.

### Enable/Disable Email Sending

In `.github/workflows/check-outages.yml`, the workflow currently runs in normal mode.

To temporarily disable emails and only log results:
- Change line 37 from `run: cargo run --release` to `run: cargo run --release -- --dry-run`

To re-enable emails:
- Change it back to `run: cargo run --release`

## How It Works

1. **GitHub Actions** triggers the workflow on schedule
2. **Rust program** runs and:
   - Fetches HEP website for your configured area for next 7 days
   - Parses HTML to extract outage information
   - Filters by location if `FILTER_LOCATION` is set
   - If matching outages found, sends email via SMTP
   - If no outages, just logs and exits
3. **Email** is sent only when there are matching outages

## Troubleshooting

### No emails received?

1. Check GitHub Actions logs for errors
2. Verify all required secrets are set correctly
3. Check your spam folder
4. Test Gmail app password by sending a test email manually
5. Ensure 2FA is enabled on Gmail
6. Try running locally with `--dry-run` to see if outages are being found

### Wrong area being checked?

1. Verify `HEP_CITY` and `HEP_OFFICE` secrets are correct
2. Visit the HEP website and double-check the URL parameters for your area
3. Run locally with `--dry-run` and check the source URL in the output

### Not receiving filtered results?

1. Check the `FILTER_LOCATION` secret value
2. Run locally: `cargo run -- --dry-run --filter "YourFilter"` to test
3. Remember: filtering is case-insensitive and does partial matching
4. The filter searches both the location AND street fields

### Workflow not running?

1. Ensure GitHub Actions is enabled in repository settings
2. Check if the workflow file is in `.github/workflows/` directory
3. Verify cron syntax is correct
4. Remember: free GitHub accounts may have delayed schedule triggers (up to 15 min)

### Build errors?

1. Make sure `Cargo.toml` and `src/main.rs` are in the correct locations
2. Check GitHub Actions logs for specific error messages
3. Ensure all dependencies are properly specified in `Cargo.toml`

## Cost

**100% Free!**
- GitHub Actions: 2,000 minutes/month free (this uses ~1 minute/day = ~30 min/month)
- Gmail: Free (within Gmail's sending limits)
- SendGrid: Free tier (100 emails/day)

## Security Notes

- Never commit secrets to your repository
- Always use GitHub Secrets for sensitive data
- Use Gmail App Passwords, not your actual password
- Consider using a dedicated email account for notifications
- Keep your repository private if you prefer to keep your area configuration private

## Dependencies

This project uses the following Rust crates:
- `reqwest` - HTTP client for fetching HEP website
- `scraper` - HTML parsing
- `lettre` - Email sending via SMTP
- `chrono` - Date/time handling
- `clap` - Command-line argument parsing

## License

Free to use and modify as needed.
