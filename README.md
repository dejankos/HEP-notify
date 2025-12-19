# HEP Power Outage Checker

Automated checker for HEP ODS (Croatian electricity company) planned power outages in Poreč area. Runs twice daily on GitHub Actions and sends email notifications when outages are detected.

## Features

- 🔍 Checks HEP website for planned outages
- 📅 Looks ahead 7 days
- 📧 Email notifications only when outages are found
- ⏰ Runs automatically twice per day (7 AM and 7 PM CET)
- 🆓 Completely free using GitHub Actions
- 🦀 Written in Rust for reliability and speed

## Setup Instructions

### 1. Fork or Create Repository

1. Create a new repository on GitHub (can be private)
2. Upload these files:
   - `Cargo.toml`
   - `src/main.rs`
   - `.github/workflows/check-outages.yml`

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

### 3. Add GitHub Secrets

1. Go to your repository on GitHub
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret** and add each of these:

   | Name | Value | Example |
   |------|-------|---------|
   | `TO_EMAIL` | Your notification email | `your-email@gmail.com` |
   | `FROM_EMAIL` | Sender email | `your-email@gmail.com` |
   | `SMTP_USERNAME` | SMTP username | `your-email@gmail.com` |
   | `SMTP_PASSWORD` | App password or API key | `abcd efgh ijkl mnop` |
   | `SMTP_SERVER` | SMTP server address | `smtp.gmail.com` |

### 4. Enable GitHub Actions

1. Go to **Actions** tab in your repository
2. If prompted, click "I understand my workflows, go ahead and enable them"
3. GitHub Actions is now enabled

### 5. Test the Setup

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

### Change Location

If you want to monitor a different area, modify the URL in `src/main.rs`:

```rust
let url = format!(
    "https://www.hep.hr/ods/bez-struje/19?dp=pula&el=128&datum={}",
    date
);
```

Visit the HEP website, select your area, and copy the URL parameters.

### Change Look-ahead Days

In `src/main.rs`, change this line:

```rust
for days_ahead in 0..=7 {  // Currently checks 7 days ahead
```

Change `7` to however many days you want to check.

## How It Works

1. **GitHub Actions** triggers the workflow on schedule
2. **Rust program** runs and:
   - Fetches HEP website for next 7 days
   - Parses HTML to extract outage information
   - If outages found, sends email via SMTP
   - If no outages, just logs and exits
3. **Email** is sent only when there are outages

## Troubleshooting

### No emails received?

1. Check GitHub Actions logs for errors
2. Verify all secrets are set correctly
3. Check your spam folder
4. Test Gmail app password by sending a test email manually
5. Ensure 2FA is enabled on Gmail

### Workflow not running?

1. Ensure GitHub Actions is enabled in repository settings
2. Check if the workflow file is in `.github/workflows/` directory
3. Verify cron syntax is correct
4. Remember: free GitHub accounts may have delayed schedule triggers (up to 15 min)

### Build errors?

1. Make sure `Cargo.toml` and `src/main.rs` are in the correct locations
2. Check GitHub Actions logs for specific error messages

## Cost

**100% Free!**
- GitHub Actions: 2,000 minutes/month free (this uses ~1 minute/day = ~30 min/month)
- Gmail: Free (within Gmail's sending limits)
- SendGrid: Free tier (100 emails/day)

## Local Testing

To test locally before pushing to GitHub:

```bash
# Set environment variables
export TO_EMAIL="your-email@gmail.com"
export FROM_EMAIL="your-email@gmail.com"
export SMTP_USERNAME="your-email@gmail.com"
export SMTP_PASSWORD="your-app-password"
export SMTP_SERVER="smtp.gmail.com"

# Build and run
cargo build --release
cargo run --release
```

## Security Notes

- Never commit secrets to your repository
- Always use GitHub Secrets for sensitive data
- Use Gmail App Passwords, not your actual password
- Consider using a dedicated email account for notifications

## License

Free to use and modify as needed.
