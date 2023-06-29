mod progress;

use regex::Regex;
use progress::{ProgressBar};
use futures_util::StreamExt;
use std::{fs::File, io::BufWriter};
use scraper::{Html, Selector};
use std::io::Write;
use std::io;
use tokio;
extern crate pretty_bytes;
use reqwest::Client;

use pretty_bytes::converter::convert;

const MEDIAFIRE: &str =  "https://www.mediafire.com/file/.*/";
const ANONFILES: &str = "https://anonfiles.com/.*";
const PIXELDRAIN: &str = "https://pixeldrain.com/.*";

fn read_input(prompt: &str) -> String {
    let mut buffer: String = String::new();
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut buffer).unwrap();
    buffer.trim().to_owned()
}

#[tokio::main]
async fn main() {
    let mut args: Vec<_> = std::env::args().collect();
    let mut arguments: Args = handleargs(args.as_mut_slice());
    if args.len() > 1 {
        arguments = handleargs(args.as_mut_slice());
    } else {
        let input = read_input("Provide the link to the file on hoster: ");
        if try_parse(input.clone()) {
            if input.split(" ").count() > 1 {
                for item in input.split(" ") {
                    arguments.queue.add_to_queue(item.to_string());
                }
            } else {
                arguments.queue.add_to_queue(input.to_string());
            }
        } else {
            std::process::exit(1);
        }
    }
    let mut downloader = Downloader::new(arguments, 0.0);
    match downloader.download().await {
        Ok(_finished) => {

        },
        Err(err) => {
            panic!("{}", err);
        }
    }
}

struct Downloader {
    arguments: Args,
    current_progress: f64,
}

impl Downloader {
    fn new(arguments: Args, current_progress: f64) -> Downloader {
        return Downloader { arguments: arguments, current_progress: current_progress};
    }

    async fn download(&mut self) -> io::Result<bool> {
        loop {
            if self.arguments.queue.completed() {
                break;
            } else {
                let item = self.arguments.queue.get_current_item();
                match self.download_from_url_host(item.to_string().as_str()).await {
                    Ok(_finished) => {
                        if !self.arguments.quiet {println!("\nDownload finished.");}
                    },
                    Err(err) => println!("{}", err),
                }
                self.arguments.queue.next();
            }
        }
        loop {
            if self.arguments.directqueue.completed() {
                break;
            } else {
                let item = self.arguments.directqueue.get_current_item();
                match self.download_from_url(item.to_string().as_str(), None).await {
                    Ok(_finished) => {
                        if !self.arguments.quiet {println!("\nDownload finished.");}
                    },
                    Err(err) => println!("{}", err),
                }
                self.arguments.directqueue.next();
            }
        }
        return Ok(true);
    }

    async fn download_from_url_host(&mut self, url: &str) -> io::Result<bool> {
        if Regex::new(ANONFILES).unwrap().is_match(url) {
            let r = self.anonfiles_download(url).await;
            match r {
                Ok(_) => return Ok(true),
                Err(_e) => return Ok(false),
            }
        } else if Regex::new(MEDIAFIRE).unwrap().is_match(url) {
            let r = self.mediafire_download(url).await;
            match r {
                Ok(_) => return Ok(true),
                Err(_e) => return Ok(false),
            }
        } else if Regex::new(PIXELDRAIN).unwrap().is_match(url) {
            let r = self.pixeldrain_download(url).await;
            match r {
                Ok(_) => return Ok(true),
                Err(_e) => return Ok(false),
            }
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid URL"));
        }
    }

    async fn pixeldrain_download(&mut self, url: &str) -> Result<bool, reqwest::Error> {
        
        let filekey = url.split('/').last().unwrap();

        let download_link = "https://pixeldrain.com/api/file/".to_string() +  filekey;

        let response = reqwest::get(url).await?;
        let body = response.text().await?;

        let result = body.lines()
        .filter_map(|s| {
            if s.contains("window.viewer_data = ") {
                s.splitn(2, "=").nth(1)
            } else {
                None
            }
        }).map(ToString::to_string);
        let results: Vec<_> = result.collect();
        let jsonstring = results[0].strip_suffix(";").unwrap();
        let filename_map = jsonstring.lines().filter_map(|s| {
            if s.contains("\"name\":\"") {
                s.split("\"name\":\"").nth(1)?.split("\"").nth(0)
            } else {
                None
            }
        }).map(ToString::to_string);

        let fn_results: Vec<_> = filename_map.collect();
        let filename = fn_results[0].as_str();

        let path = r#".\"#.to_string() + filename;
        match self.download_from_url(&download_link, Some(path)).await {
            Ok(_) => return Ok(true),
            Err(err) => panic!("Error downloading from {} with Error: {}", url, err),
        }
    }

    async fn anonfiles_download(&mut self, url: &str) -> Result<bool, reqwest::Error> {
        let response = reqwest::get(url).await?;
        let body = response.text().await?;

        let document = Html::parse_document(&body);
        let selector = Selector::parse(r#"#download-url"#).unwrap();
        let mut download_link = String::new();
        for element in document.select(&selector) {
            download_link = element
            .value()
            .attr("href")
            .expect("href not found")
            .to_string();
            if !self.arguments.quiet {println!("Download link found!");}
        }
        match self.download_from_url(&download_link, None).await {
            Ok(_) => return Ok(true),
            Err(err) => panic!("Error downloading from {} with Error: {}", url, err),
        }
    }


    async fn mediafire_download(&mut self, url: &str) -> Result<bool, reqwest::Error> {

        let response = reqwest::get(url).await?;
        let body = response.text().await?;

        let document = Html::parse_document(&body);
        let selector = Selector::parse(r#"#downloadButton"#).unwrap();
        let mut download_link = String::new();
        for element in document.select(&selector) {
            download_link = element
            .value()
            .attr("href")
            .expect("href not found")
            .to_string();
            if !self.arguments.quiet {println!("Download link found!");}
        }
        match self.download_from_url(&download_link, None).await {
            Ok(_) => return Ok(true),
            Err(err) => panic!("Error downloading from {} with {}", url, err),
        }
        
    }

    async fn download_from_url(&mut self, url: &str, _path: Option<String>) -> Result<bool, String> {
        let client = Client::new();
        let response = client.get(url).send().await
            .or(Err(format!("Failed to get response from URL: {}", url)))?;
        let download_link = &response.url().to_string().split("/").last().expect("File name not found.").to_string();

        let mut path = r#".\"#.to_string() + download_link;
        if let Some(p) = _path {
            path = p;
        } 
        let total_size = response.content_length().ok_or("Failed to get total size from response")?;
        let suffix = "]".to_string();
        let temp_path = path.to_string() + ".temp";
    
        let mut progress = ProgressBar::new("[".to_string(), suffix.as_str().to_string(), "#".to_string(), "~".to_string(), 0.0, total_size as f64);
    
        let mut file = BufWriter::new(File::create(&temp_path).map_err(|e| format!("Failed to create file {}: {}", temp_path, e))?);
        let mut stream = response.bytes_stream();
    
        if !self.arguments.quiet {
            println!("Starting download of {}", path.split("\\").last().unwrap());
        }
    
        use std::time::Instant;
        let now = Instant::now();
    
        while let Some(bytes) = stream.next().await {
            let chunk = bytes.map_err(|e| format!("Failed to read chunk from stream: {}", e))?;
            file.write_all(&chunk)
                .map_err(|e| format!("Failed to write to new file {}: {}", temp_path, e))?;
            self.current_progress += chunk.len() as f64;
            match progress.show() {
                Ok(_) => {
                    progress.suffix = suffix.clone() + format!(
                        " {}/{} - {:.2}MB/s -",
                        convert(self.current_progress),
                        convert(progress.progress_obj.max_value),
                        ((self.current_progress / 1000000.0) / now.elapsed().as_secs() as f64)
                    )
                    .as_str();
                    progress.update_progress(self.current_progress);
                }
                Err(err) => panic!("{}", err),
            }
        }
    
        progress.show().unwrap();
    
        file.flush().map_err(|e| format!("Failed to flush buffer to file {}: {}", temp_path, e))?;
        drop(file);
    
        std::fs::rename(&temp_path, &path).map_err(|e| format!("Failed to rename file from {} to {}: {}", temp_path, path, e))?;
    
        Ok(true)
    }
}

struct Queue {
    files: Vec<String>,
    index: usize,
    quiet: bool,
}

impl Queue {
    fn new(files: Vec<String>, quiet:bool) -> Queue {
        return Queue { files: files, index: 0, quiet: quiet};
    }

    fn completed(&self) -> bool {
        return self.files.len() < self.index + 1;
    }

    fn get_current_item(&self) -> &str {
        return self.files[self.index].as_str();
    }

    fn next(&mut self) {
        if self.index + 1 >= self.files.len() {
            if !self.quiet { println!("Reached end of queue."); }
            std::process::exit(1);
        } else {
            self.index = self.index + 1;
        }
    }

    fn add_to_queue_str(&mut self, file: &str) {
        self.files.push(file.to_string());
    }

    fn add_to_queue(&mut self, file: String) {
        self.files.push(file);
    }
}

struct Args {
    directqueue: Queue,
    queue: Queue,
    quiet: bool,
}

impl Args {
    fn new(queue: Queue, quiet: bool) -> Args {
        return Args { queue: queue, quiet: quiet, directqueue: Queue::new(Vec::new(), false) };
    }
}

fn handleargs(args: &[String]) -> Args {
    let mut arguments = Args::new(Queue::new(Vec::new(), false), false);
    let newargs = &args[1..];
    for arg in newargs {
        if arg == "-q" {
            arguments.quiet = true;
            arguments.queue.quiet = true;
            arguments.directqueue.quiet = true;
        } else if Regex::new(MEDIAFIRE).unwrap().is_match(arg.as_str()) || Regex::new(ANONFILES).unwrap().is_match(arg.as_str()) ||  Regex::new(PIXELDRAIN).unwrap().is_match(arg.as_str()) {
            arguments.queue.add_to_queue_str(&arg);
        } else {
            arguments.directqueue.add_to_queue_str(&arg);
        }
    }
    return arguments;
}

fn try_parse(input: String) -> bool {

    if Regex::new(MEDIAFIRE).unwrap().is_match(input.as_str()) || Regex::new(ANONFILES).unwrap().is_match(input.as_str()) || Regex::new(PIXELDRAIN).unwrap().is_match(input.as_str()) {
        return true;
    } else {
        println!("File hoster could not be detected...");
        return false;
    }
}