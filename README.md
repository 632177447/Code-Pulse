# CodePulse

CodePulse is a cross-platform desktop application for AI-powered code generation and analysis, built with Tauri, Vue 3, and TypeScript.

## Features

- **AI Code Generation**: Generate code snippets, functions, and components using AI
- **Code Analysis**: Analyze code for potential issues and improvements
- **Cross-Platform**: Runs on Windows, macOS, and Linux
- **Modern UI**: Clean and intuitive user interface with Vue 3
- **Desktop Integration**: Native desktop application with Tauri

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite
- **Desktop Framework**: Tauri
- **Styling**: Tailwind CSS
- **Icons**: Lucide React
- **State Management**: Pinia
- **AI Integration**: OpenAI API (configurable)

## Prerequisites

- Node.js (v18 or higher)
- Rust (v1.65 or higher)
- Cargo (Rust package manager)

## Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd CodePulse
   ```

2. Install frontend dependencies:
   ```bash
   cd src-tauri
   npm install
   ```

3. Install backend dependencies:
   ```bash
   cd ..
   npm install
   ```

4. Configure AI API Key:
   Create a `.env` file in the `src-tauri` directory with your OpenAI API key:
   ```env
   OPENAI_API_KEY=your_openai_api_key_here
   ```

## Development

### Start Development Server

```bash
npm run dev
```

This will start both the frontend development server and the Tauri backend. You can access the application at `http://localhost:5173`.

### Run Tauri Commands

```bash
# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build

# Run specific Tauri commands
npm run tauri dev -- --target x86_64-pc-windows-msvc
```

## Project Structure

```
CodePulse/
├── src-tauri/          # Tauri backend and Rust code
│   ├── src/             # Rust source code
│   ├── Cargo.toml       # Rust dependencies
│   └── ...
├── src/                 # Vue frontend
│   ├── components/      # Vue components
│   ├── stores/          # Pinia stores
│   ├── views/           # Page components
│   ├── App.vue          # Root component
│   ├── main.ts          # Entry point
│   └── ...
├── public/              # Static assets
├── .env                 # Environment variables (in src-tauri/)
├── package.json         # Frontend dependencies
└── README.md            # Project documentation
```

## Configuration

### AI Settings

You can configure AI settings in the frontend:

1. Open `src/stores/aiSettings.ts`
2. Update the `model` and `temperature` values as needed
3. The API key is loaded from the environment variable `OPENAI_API_KEY`

### Tauri Settings

For Tauri-specific configuration, edit `src-tauri/tauri.conf.json`:

```json
{
  "package": {
    "productName": "CodePulse",
    "version": "1.0.0"
  },
  "tauri": {
    "bundle": {
      "targets": null,
      "windows": {
        "webviewInstallMode": {
          "type": "downloadBootstrapper"
        }
      }
    }
  }
}
```

## Development Commands

| Command | Description |
|---------|-------------|
| `npm run dev` | Start frontend dev server + Tauri backend |
| `npm run tauri dev` | Start Tauri backend only |
| `npm run tauri build` | Build for production |
| `npm run tauri build -- --target x86_64-pc-windows-msvc` | Build for specific target |
| `npm run tauri dev -- --target x86_64-pc-windows-msvc` | Run on specific target |

## Building for Production

To create a production build:

```bash
npm run tauri build
```

This will create executables for your platform in the `src-tauri/target/release/` directory.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

For issues or questions, please open an issue on the repository.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
