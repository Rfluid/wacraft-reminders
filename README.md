# Wacraft Reminders CLI

**`wacraft-reminders`** is an open-source command-line tool developed to work with [Astervia's wacraft](https://wacraft.astervia.tech) to automate the process of re-engaging inactive users. Built with Rust, this CLI connects to the Wacraft API to identify contacts who haven't communicated in a while and sends them customized reminders through various channels.

It's designed to be run as a background service (daemon) on your server, ensuring your user engagement strategy runs smoothly and automatically.

---

## ✨ Features

- **Automated Inactivity Tracking**: Automatically fetches conversations from the Wacraft server to identify inactive contacts.
- **Configurable Rules**: Define multiple reminder rules based on inactivity periods (e.g., send a message after 3 days, an email after 7 days).
- **Multi-Channel Reminders**:
    - Send a WhatsApp message directly via the Wacraft API.
    - Send a customized email via SMTP.
    - Trigger a webhook or external API with a generic HTTP request.
- **Daemon Mode**: Run the tool as a persistent background service that periodically checks for and sends reminders.
- **Secure Configuration**: Keeps your credentials and rules in local configuration files, separate from the application logic.
- **Built with Rust**: Fast, reliable, and memory-efficient, perfect for a long-running background service.

## 📦 Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) toolchain (for building from source).
- Access to a Wacraft server instance.

### From Source (Recommended)

1.  Clone the repository:
    ```bash
    git clone https://github.com/Rfluid/wacraft-reminders.git
    cd wacraft-reminders
    ```
2.  Build and install the binary:
    ```bash
    cargo install --path .
    ```
3.  Verify the installation:
    ```bash
    wacraft-reminders --version
    ```

### From Crates.io

_Coming soon\! Once published, you will be able to install it with:_

```bash
cargo install wacraft-reminders
```

## 🚀 Getting Started

### 1. Initialize Configuration

The first step is to create the necessary configuration files. Run the `init` command:

```bash
wacraft-reminders config init
```

This will create a `wacraft-reminders` directory in your system's default config location (e.g., `~/.config/wacraft-reminders` on Linux) with two files:

- `settings.json`: For your service credentials (Wacraft and SMTP).
- `reminders.json`: For your inactivity rules.

### 2. Edit `settings.json`

Open the `settings.json` file and fill in your credentials:

```json
{
    "wacraft": {
        "base_url": "https://your-wacraft-api-url.com",
        "email": "your-wacraft-login-email@example.com",
        "password": "your-wacraft-login-password",
        "access_token": null,
        "refresh_token": null,
        "token_expires_at": null
    },
    "email": {
        "smtp_server": "smtp.example.com",
        "smtp_port": 587,
        "smtp_user": "your-smtp-user",
        "smtp_password": "your-smtp-password",
        "from_address": "no-reply@yourcompany.com"
    }
}
```

### 3. Define Rules in `reminders.json`

Open `reminders.json` and add your reminder rules. Here are a few examples:

```json
[
    {
        "name": "12-Hour WhatsApp Nudge",
        "inactive_for_hours": 12,
        "action": {
            "type": "wacraft_message",
            "sender_data": {
                "recipient_type": "individual",
                "messaging_product": "whatsapp",
                "type": "template",
                "template": {
                    "name": "hello_world",
                    "language": {
                        "code": "en_US"
                    }
                }
            }
        }
    },
    {
        "name": "24-Hour Email Follow-up",
        "inactive_for_hours": 24,
        "action": {
            "type": "email",
            "subject": "We miss you, {contact_name}!",
            "template": "/path/to/your/templates/email_7_days.html"
        }
    },
    {
        "name": "48-Hour CRM Webhook",
        "inactive_for_hours": 48,
        "action": {
            "type": "http_request",
            "method": "POST",
            "url": "https://your-crm.com/api/webhook/inactive-contact",
            "headers": {
                "Authorization": "Bearer your-secret-token"
            },
            "body": {
                "contact_id": "{contact_id}",
                "name": "{contact_name}",
                "email": "{contact_email}"
            }
        }
    },
    {
        "name": "72-Hour Do Nothing",
        "inactive_for_hours": 72
    }
]
```

## 🧰 Usage

### Command Structure

```bash
wacraft-reminders <COMMAND>
```

### `config` Commands

- `wacraft-reminders config init [--force]`: Creates default configuration files.
- `wacraft-reminders config view`: Displays the content of your configuration files.
- `wacraft-reminders config path`: Shows the path to the configuration directory.

### `reminders` Commands

- `wacraft-reminders reminders send --contact-id <CONTACT_ID>`: Manually triggers a reminder check for a single contact. The tool will evaluate the rules and send the appropriate reminder.

### `daemon` Commands

- `wacraft-reminders daemon run [--interval <SECONDS>] [--batch-size <SIZE>]`: Starts the daemon in the foreground. It will check all contacts at the specified interval.
- `wacraft-reminders daemon run --detached`: Starts the daemon as a background process.
- `wacraft-reminders daemon stop`: Stops the background daemon process.
- `wacraft-reminders daemon logs`: Shows the log file for the daemon.

## 🤝 Contributing

This is an open-source project designed to work with [Astervia's wacraft](https://wacraft.astervia.tech). Contributions are welcome. If you'd like to contribute, please feel free to fork the repository, make your changes, and submit a pull request.

## 📜 License

This project is licensed under the MIT License. See the [LICENSE](https://mit-license.org/) file for details.

## 📧 Contact

- Email: [ruy.vieiraneto@gmail.com](mailto:ruy.vieiraneto@gmail.com)
- GitHub: [https://github.com/Rfluid/wacraft-reminders](https://github.com/Rfluid/wacraft-reminders)
