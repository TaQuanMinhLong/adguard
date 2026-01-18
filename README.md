# AdGuard Manager

A modern desktop application built with Tauri 2 for managing ad-blocking using the hosts file. Provides an intuitive interface to view, add, and remove blocked domains with a comprehensive backup and history system.

## Features

### 🚫 Domain Management
- **View Blocked Domains**: See all currently blocked domains in a clean, organized interface
- **Add Domains**: Easily add new domains to block
- **Remove Domains**: Remove domains from the block list with a single click
- **Save Changes**: Commit changes to the hosts file with automatic DNS cache flushing

### 📚 History & Backup
- **Automatic Backups**: Every save operation creates a backup snapshot
- **History Management**: View all backup history entries with timestamps, entry counts, and file sizes
- **Rollback Support**: Restore your hosts file to any previous backup
- **Multiple Selection**: Select and delete multiple history entries at once
- **Configurable Limits**: Set maximum number of history entries to keep

### ⚙️ Configuration
- **Custom Hosts File Path**: Configure the path to your hosts file (defaults to platform-specific location)
- **History Directory**: Set custom location for backup history files
- **Theme Support**: Switch between dark and light themes
- **Platform-Specific Defaults**: Automatic detection of hosts file location based on your OS

### 🔒 Security & Permissions
- **Admin Privileges Check**: Automatically detects if the app is running with administrator privileges
- **Warning System**: Clear alerts when admin privileges are not available
- **Safe Operations**: Prevents accidental modifications when not running as administrator

### 🎨 User Interface
- **Modern Design**: Clean, modern interface with dark mode first approach
- **Responsive Layout**: Grid-based layout that adapts to container size
- **Smooth Transitions**: Elegant animations for tab switching and interactions
- **Toast Notifications**: Rich toast notifications for user feedback
- **Accessible**: Friendly light mode for better accessibility

## Tech Stack

### Frontend
- **Vue 3**: Progressive JavaScript framework
- **TypeScript**: Type-safe JavaScript
- **TailwindCSS v4**: Utility-first CSS framework with custom theming
- **Vite**: Fast build tool and dev server
- **vue-sonner**: Toast notification library
- **lucide-vue-next**: Icon library
- **reka-ui**: UI component primitives

### Backend
- **Rust**: Systems programming language
- **Tauri 2**: Framework for building desktop applications
- **pest**: Parsing Expression Grammar (PEG) library for parsing hosts and config files
- **notify**: Cross-platform file system notifications
- **parking_lot**: Efficient synchronization primitives
- **chrono**: Date and time library
- **tokio**: Async runtime

## Prerequisites

- **Rust**: Latest stable version ([Install Rust](https://www.rust-lang.org/tools/install))
- **Node.js**: v18 or higher ([Install Node.js](https://nodejs.org/))
- **Package Manager**: npm, yarn, pnpm, or bun
- **System Requirements**:
  - Windows 10/11 (currently supported)
  - Linux (planned)
  - macOS (planned)

## Installation

### Development Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd adguard
   ```

2. **Install frontend dependencies**
   ```bash
   npm install
   # or
   bun install
   # or
   pnpm install
   ```

3. **Run the development server**
   ```bash
   npm run tauri dev
   # or
   bun run tauri dev
   ```

### Building for Production

1. **Build the application**
   ```bash
   npm run tauri build
   # or
   bun run tauri build
   ```

2. **Output location**
   - The built application will be in `src-tauri/target/release/`
   - Installers will be in `src-tauri/target/release/bundle/`

## Usage

### First Run

1. **Launch the application** (requires administrator privileges on Windows)
2. The app will automatically:
   - Detect your hosts file location
   - Load existing blocked domains
   - Create a default history directory in App Data

### Managing Blocked Domains

1. **View Blocked Domains**
   - Navigate to the "Blocked Domains" tab
   - See all currently blocked domains in a responsive grid

2. **Add a Domain**
   - Click the "Add Domain" button
   - Enter the domain name (e.g., `example.com`)
   - Click "Add"
   - Click "Save Changes" to commit to the hosts file

3. **Remove a Domain**
   - Click the "Remove" button next to any domain
   - Click "Save Changes" to commit the removal

### History & Backups

1. **View History**
   - Navigate to the "History" tab
   - See all backup snapshots with details

2. **Restore a Backup**
   - Click "Restore" on any history entry
   - Confirm the rollback operation
   - The hosts file will be restored to that state

3. **Delete History Entries**
   - Select one or more history entries using checkboxes
   - Click "Delete" to remove selected entries

### Settings

1. **Configure Paths**
   - Set custom hosts file path (leave empty for default)
   - Set custom history directory (leave empty for default)

2. **History Settings**
   - Set maximum number of history entries to keep (1-1000)

3. **Appearance**
   - Switch between dark and light themes

## Project Structure

```
adguard/
├── src/                          # Frontend source code
│   ├── components/               # Vue components
│   │   ├── BlockedList.vue     # Main domain management interface
│   │   ├── History.vue          # Backup history interface
│   │   ├── Settings.vue         # Settings interface
│   │   └── ui/                  # UI component library
│   ├── composables/             # Vue composables
│   ├── state.ts                 # Global state management
│   └── tailwind.css             # TailwindCSS configuration
├── src-tauri/                   # Backend source code
│   ├── src/
│   │   ├── lib.rs               # Application entry point
│   │   ├── commands.rs          # Tauri commands
│   │   ├── state.rs             # Application state management
│   │   ├── parser.rs            # Hosts file parser
│   │   ├── config.rs            # Configuration management
│   │   ├── history.rs           # Backup history management
│   │   ├── platform.rs          # Platform-specific utilities
│   │   ├── watcher.rs           # File system watcher
│   │   └── commit.rs            # Commit operations
│   ├── grammar/                 # Pest grammar files
│   │   ├── hosts.pest           # Hosts file grammar
│   │   └── config.pest          # Config file grammar
│   └── Cargo.toml               # Rust dependencies
├── package.json                 # Frontend dependencies
└── README.md                    # This file
```

## Configuration

The application uses an INI-like configuration file stored in platform-specific App Data directories:

- **Windows**: `%APPDATA%\adguard\config.ini`
- **Linux**: `~/.config/adguard/config.ini` (planned)
- **macOS**: `~/Library/Application Support/adguard/config.ini` (planned)

### Configuration Format

```ini
[paths]
host_file_path = C:\Windows\System32\drivers\etc\hosts
history_dir = C:\Users\YourName\AppData\Roaming\adguard\history

[appearance]
theme = dark

[history]
max_history_entries = 50
```

## Development

### Running Tests

```bash
cd src-tauri
cargo test
```

### Code Style

- **Rust**: Follows standard Rust formatting (`cargo fmt`)
- **TypeScript/Vue**: Uses Biome for formatting and linting

### Key Design Decisions

1. **Hosts File Parsing**: Uses Pest (PEG) for robust parsing that preserves comments and non-localhost entries
2. **State Management**: Uses Tauri's built-in `AppState` with `parking_lot` for efficient synchronization
3. **File Watching**: Monitors hosts file for external changes and automatically updates state
4. **History System**: Complete file snapshots for reliable rollback functionality
5. **Theme System**: CSS variables with TailwindCSS for easy theme switching

## Platform Support

### Currently Supported
- ✅ Windows 10/11

### Planned Support
- 🔲 Linux
- 🔲 macOS

## Security Considerations

- **Administrator Privileges**: Required to modify the hosts file
- **File Validation**: History files are verified before rollback
- **Atomic Writes**: Hosts file updates use atomic file operations
- **Safe Parsing**: Robust parsing prevents corruption of existing hosts file entries

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Add your license here]

## Acknowledgments

- Built with [Tauri](https://tauri.app/)
- Icons from [Lucide](https://lucide.dev/)
- UI components inspired by [shadcn/ui](https://ui.shadcn.com/)
