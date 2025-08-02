# 🦊 Fox2Web - Website Downloader

A fast and efficient website downloader written in Rust.

## Features

- 📥 Download HTML pages
- 📦 Download and organize assets (CSS, JS, Images)  
- 🗂️ Automatic directory structure creation
- 🎯 URL path correction for local viewing
- 📊 Progress bar for asset downloads
- 🎨 Colorful terminal output
- ⚡ Fast and concurrent downloads

## Installation

Make sure you have Rust installed, then run:

```bash
cargo build --release
```

## Usage

### Command Line Arguments

```bash
# With arguments
./f2web -p Google -t https://google.com

# Or on Linux/Mac
./f2web -p Google -t https://google.com
```

### Interactive Mode

Simply run the executable without arguments and it will ask for:
- Project name (folder name)
- Target URL

```bash
./f2web
```

### Arguments

- `-p, --project <PROJECT_NAME>`: Project name (folder name for downloaded files)
- `-t, --target <URL>`: Target website URL to download
- `-h, --help`: Show help information
- `-V, --version`: Show version

## How it works

1. **Creates Project Structure**: Sets up organized folders:
   ```
   ProjectName/
   ├── index.html
   ├── css/
   ├── js/
   ├── images/
   └── assets/
   ```

2. **Downloads Main Page**: Fetches the HTML content from the target URL

3. **Extracts Assets**: Parses HTML to find:
   - CSS stylesheets (`<link rel="stylesheet">`) 
   - JavaScript files (`<script src="">`)
   - Images (`<img src="">`)

4. **Downloads Assets**: Downloads all found assets with progress tracking

5. **Processes HTML**: Updates asset paths to point to local files

6. **Saves Everything**: Creates a local copy ready for offline viewing

## Example

```bash
# Download Google's homepage
./f2web -p Google -t https://google.com

# Download a website interactively
./f2web
📁 Enter project name: MyWebsite
🌐 Enter target URL: https://example.com
```

## Output

The tool will create a folder with your project name containing:
- `index.html` - The main page with corrected asset paths
- `css/` - All stylesheets
- `js/` - All JavaScript files  
- `images/` - All images (PNG, JPG, GIF, SVG, WebP)
- `assets/` - Other assets

## License

Open source - feel free to use and modify!
