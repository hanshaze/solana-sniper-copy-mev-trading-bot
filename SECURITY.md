# 🔐 Security Policy

## 📆 Supported Versions

| Version         | Supported |
|------------------|-----------|
| `main` (latest)  | ✅ Yes     |
| older versions   | ❌ No      |

We only support the latest `main` branch for active development and security updates.

---

## 📣 Reporting a Vulnerability

If you discover a security vulnerability, **please report it responsibly**.  
**Do not** create a public GitHub issue.

### 🔒 How to Report

Send a private disclosure message to:

- **Telegram (Preferred):** [@hanshaze007](https://t.me/hanshaze007)  


We will acknowledge your report within **48 hours** and work with you on a timely resolution.

---

## 📌 What to Report

Please report vulnerabilities such as:

- Private key leakage
- Transaction spoofing or unauthorized trade execution
- Unsafe default config behavior (e.g. unsafe slippage)
- MEV or sniper logic bugs causing unintended trades
- Telegram command injection / exploits
- Dependency vulnerabilities (e.g. `node_modules` packages with CVEs)

---

## ❌ Out of Scope

The following are **not considered security issues**:

- Losing funds due to poor strategy configuration
- Market losses (slippage, impermanent loss, front-running)
- Insecure user environments (e.g. leaked `.env`)
- User misconfiguration or misuse

---

## 🛡️ Security Best Practices for Users

- Always use a **burner wallet** during development and testing
- Never commit your `.env` or `PRIVATE_KEY` to GitHub
- Set proper file permissions for `.env`
- Use strong passwords and 2FA on GitHub and Telegram
- Run the bot on a secure, trusted VPS or local machine
- Review PRs and third-party code before merging

---

## 🤝 Disclosure Process

1. Report vulnerability privately (Telegram or email)
2. We'll confirm receipt within 48 hours
3. We'll investigate and patch within 7–14 days
4. Optional: Public CVE disclosure with your credit

---

Thanks for making **Solana Sniper Copy MEV Trading Bot** safer for the entire Solana community.  
Security is a shared responsibility — and we appreciate your help.

*Maintained by [@hanshaze007](https://github.com/hanshaze007)*
