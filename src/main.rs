use clap::{Arg, Command};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

struct Fox2Web {
    client: Client,
    base_url: Url,
    project_name: String,
    downloaded_urls: HashSet<String>,
}

impl Fox2Web {
    fn new(target_url: &str, project_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .user_agent("Fox2Web/1.0 (Website Downloader)")
            .build()?;
        
        let base_url = Url::parse(target_url)?;
        
        Ok(Fox2Web {
            client,
            base_url,
            project_name: project_name.to_string(),
            downloaded_urls: HashSet::new(),
        })
    }

    fn create_project_directory(&self) -> Result<(), Box<dyn std::error::Error>> {
        let project_path = Path::new(&self.project_name);
        if !project_path.exists() {
            fs::create_dir_all(project_path)?;
            fs::create_dir_all(project_path.join("assets"))?;
            fs::create_dir_all(project_path.join("css"))?;
            fs::create_dir_all(project_path.join("js"))?;
            fs::create_dir_all(project_path.join("images"))?;
        }
        Ok(())
    }

    fn download_page(&mut self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        if self.downloaded_urls.contains(url) {
            return Ok(String::new());
        }

        println!("{} {}", "📥 Downloading:".green().bold(), url.cyan());
        
        let response = self.client.get(url).send()?;
        let content = response.text()?;
        
        self.downloaded_urls.insert(url.to_string());
        Ok(content)
    }

    fn extract_assets(&self, html: &str) -> Vec<String> {
        let document = Html::parse_document(html);
        let mut assets = Vec::new();

        // CSS files
        let css_selector = Selector::parse("link[rel='stylesheet']").unwrap();
        for element in document.select(&css_selector) {
            if let Some(href) = element.value().attr("href") {
                assets.push(href.to_string());
            }
        }

        // JavaScript files
        let js_selector = Selector::parse("script[src]").unwrap();
        for element in document.select(&js_selector) {
            if let Some(src) = element.value().attr("src") {
                assets.push(src.to_string());
            }
        }

        // Images
        let img_selector = Selector::parse("img[src]").unwrap();
        for element in document.select(&img_selector) {
            if let Some(src) = element.value().attr("src") {
                assets.push(src.to_string());
            }
        }

        assets
    }

    fn resolve_url(&self, relative_url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let resolved = self.base_url.join(relative_url)?;
        Ok(resolved.to_string())
    }

    fn get_file_extension(&self, url: &str) -> String {
        let parsed_url = Url::parse(url).unwrap();
        let path = parsed_url.path();
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("html")
            .to_string()
    }

    fn get_asset_directory(&self, url: &str) -> &str {
        let ext = self.get_file_extension(url);
        match ext.as_str() {
            "css" => "css",
            "js" => "js",
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "images",
            _ => "assets",
        }
    }

    fn download_asset(&self, asset_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let full_url = self.resolve_url(asset_url)?;
        
        println!("{} {}", "📦 Asset:".yellow().bold(), asset_url.blue());
        
        let response = self.client.get(&full_url).send()?;
        let content = response.bytes()?;
        
        let asset_dir = self.get_asset_directory(&full_url);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let default_name = format!("asset_{}", timestamp);
        let filename = Path::new(asset_url)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(&default_name))
            .to_str()
            .unwrap();
        
        let asset_path = Path::new(&self.project_name)
            .join(asset_dir)
            .join(filename);
        
        if let Some(parent) = asset_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(asset_path, content)?;
        Ok(())
    }

    fn process_html(&self, mut html: String) -> String {
        // Update asset paths to local paths
        let css_regex = Regex::new(r#"href=["']([^"']+\.css[^"']*)["']"#).unwrap();
        html = css_regex.replace_all(&html, |caps: &regex::Captures| {
            let original = &caps[1];
            let filename = Path::new(original).file_name().unwrap().to_str().unwrap();
            format!(r#"href="css/{}""#, filename)
        }).to_string();

        let js_regex = Regex::new(r#"src=["']([^"']+\.js[^"']*)["']"#).unwrap();
        html = js_regex.replace_all(&html, |caps: &regex::Captures| {
            let original = &caps[1];
            let filename = Path::new(original).file_name().unwrap().to_str().unwrap();
            format!(r#"src="js/{}""#, filename)
        }).to_string();

        let img_regex = Regex::new(r#"src=["']([^"']+\.(png|jpg|jpeg|gif|svg|webp)[^"']*)["']"#).unwrap();
        html = img_regex.replace_all(&html, |caps: &regex::Captures| {
            let original = &caps[1];
            let filename = Path::new(original).file_name().unwrap().to_str().unwrap();
            format!(r#"src="images/{}""#, filename)
        }).to_string();

        html
    }

    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "🦊 Fox2Web - Website Downloader".red().bold());
        println!("{} {}", "🎯 Target:".green().bold(), self.base_url.as_str().cyan());
        println!("{} {}", "📁 Project:".green().bold(), self.project_name.yellow());
        println!();

        self.create_project_directory()?;

        // Download main page
        let base_url_str = self.base_url.as_str().to_string();
        let html_content = self.download_page(&base_url_str)?;
        
        // Extract assets
        println!("{}", "🔍 Extracting assets...".blue().bold());
        let assets = self.extract_assets(&html_content);
        
        println!("{} {} assets found", "✨".green(), assets.len().to_string().yellow().bold());
        
        // Download assets with progress bar
        let pb = ProgressBar::new(assets.len() as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"));
        
        for asset in &assets {
            pb.set_message(format!("Downloading: {}", asset));
            if let Err(e) = self.download_asset(asset) {
                println!("{} Failed to download {}: {}", "❌".red(), asset, e);
            }
            pb.inc(1);
        }
        pb.finish_with_message("Assets downloaded!");

        // Process and save HTML
        println!("{}", "🔧 Processing HTML...".blue().bold());
        let processed_html = self.process_html(html_content);
        let index_file = "index.html";
        let index_path = Path::new(&self.project_name).join(index_file);
        fs::write(index_path, processed_html)?;

        println!();
        println!("{}", "✅ Download completed successfully!".green().bold());
        println!("{} {}", "📂 Files saved to:".blue(), self.project_name.yellow().bold());
        
        Ok(())
    }
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt.cyan().bold());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_name = "Fox2Web";
    let app_version = "1.0.0";
    let app_author = "Hossein Pira";
    let app_description = "🦊 Fox2Web - Terminal Website Downloader";
    let project_help = "Project name (folder name for downloaded files)";
    let target_help = "Target website URL to download";
    let project_prompt = "📁 Enter project name: ";
    let target_prompt = "🌐 Enter target URL: ";
    let error_msg = "❌ Project name and target URL are required!";
    
    let matches = Command::new(app_name)
        .version(app_version)
        .author(app_author)
        .about(app_description)
        .arg(
            Arg::new("project")
                .short('p')
                .long("project")
                .value_name("PROJECT_NAME")
                .help(project_help)
        )
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .value_name("URL")
                .help(target_help)
        )
        .get_matches();

    let project_name = match matches.get_one::<String>("project") {
        Some(name) => name.clone(),
        None => get_user_input(project_prompt),
    };

    let target_url = match matches.get_one::<String>("target") {
        Some(url) => url.clone(),
        None => get_user_input(target_prompt),
    };

    if project_name.is_empty() || target_url.is_empty() {
        println!("{}", error_msg.red().bold());
        return Ok(());
    }

    // Ensure URL has protocol
    let full_url = if target_url.starts_with("http://") || target_url.starts_with("https://") {
        target_url
    } else {
        format!("https://{}", target_url)
    };

    let mut fox2web = Fox2Web::new(&full_url, &project_name)?;
    fox2web.run()?;

    Ok(())
}
