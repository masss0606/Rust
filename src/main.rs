use std::io::{self, BufReader, Read};
use std::path::Path;
use std::process;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use rand::seq::SliceRandom;
use tar::Archive;
use flate2::read::GzDecoder;
use reqwest::Url;
use tokio::runtime::Runtime;

// Define a constant
const DATA_URL: &str = "http://zoo.cs.yale.edu/classes/cs458/lectures/sklearn/ud/ud120-projects-master/enron_mail_20150507.tgz";
const SAMPLE_SIZE: usize = 10_000;

struct Graph {
    vertices: HashSet<String>,
    edges: HashMap<String, HashSet<String>>,
}

impl Graph {
    fn new() -> Self {
        Self {
            vertices: HashSet::new(),
            edges: HashMap::new(),
        }
    }

    fn add_edge(&mut self, source: &str, target: &str) {
        self.vertices.insert(source.to_string());
        self.vertices.insert(target.to_string());

        let source_edges = self.edges.entry(source.to_string()).or_insert(HashSet::new());
        source_edges.insert(target.to_string());

        let target_edges = self.edges.entry(target.to_string()).or_insert(HashSet::new());
        target_edges.insert(source.to_string());
    }

    fn get_neighbors(&self, vertex: &str) -> HashSet<String> {
        self.edges.get(vertex).cloned().unwrap_or(HashSet::new()) // return  HashSet<String>
    }
}

// The first stage: data acquisition and preprocessing
async fn download_and_extract_data() -> Result<(), Box<dyn Error>> {
    // 下载数据集
    let data = download_data().await?;
    // 解压缩数据
    extract_data(data)?;

    Ok(())
}

async fn download_data() -> Result<Vec<u8>, Box<dyn Error>> {
// Download the data set
    let response = reqwest::get(DATA_URL).await?;
// Check whether the request is successful
    if !response.status().is_success() {
        return Err("Unable to download data set".into());
    }

// Read the response body
    let mut buf = Vec::new();
    let bytes = response.bytes().await?;
    buf.extend_from_slice(&bytes); // Append bytes to the vector using the extend_from_slice method

    Ok(buf)
}

fn extract_data(data: Vec<u8>) -> Result<(), Box<dyn Error>> {
// Decompress the data
    let reader = GzDecoder::new(data.as_slice());
    let mut tar = Archive::new(reader); // Declare tar as mutable

    // Create a vector to hold the sampled emails
    let mut sampled_emails = Vec::new();

    // Iterate over each entry in the archive
    for entry in tar.entries()? {
        let mut entry = entry?;

        // Check whether the entry is a file
        if entry.header().entry_type().is_file() {
            // Read the email content
            let mut content = String::new();
            entry.read_to_string(&mut content)?;

            // Push the email content into the vector
            sampled_emails.push(content);

            // Stop when the number of sampled emails reaches the sample size
            if sampled_emails.len() >= SAMPLE_SIZE {
                break;
            }
        }
    }

    // Random scrambled sampling of emails
    let mut rng = rand::thread_rng();
    sampled_emails.shuffle(&mut rng);

    // Print the first 10 sampled emails for verification
    for email in sampled_emails.iter().take(10) {
        println!("{}", email);
    }

    Ok(())
}

// Second stage: Average distance calculation
fn calculate_average_distance(graph: &Graph) -> f64 {
    let mut total_distance = 0;
    let mut total_pairs = 0;

    for vertex in graph.vertices.iter() {
        let distances = bfs(graph, vertex);

        for (_, distance) in distances.iter() {
            total_distance += distance;
            total_pairs += 1;
        }
    }

    total_distance as f64 / total_pairs as f64
}

fn bfs(graph: &Graph, start_vertex: &str) -> HashMap<String, usize> {
    let mut distances: HashMap<String, usize> = HashMap::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new(); // Change the queue type to a tuple containing distance

    distances.insert(start_vertex.to_string(), 0);
    visited.insert(start_vertex.to_string());
    queue.push_back((start_vertex.to_string(), 0)); // Initialize the starting vertex distance to 0

    while let Some((current_vertex, current_distance)) = queue.pop_front() { // Modify the iteration variable to a tuple
        for neighbor in graph.get_neighbors(&current_vertex) {
            if !visited.contains(neighbor.as_str()) {
                let new_distance = current_distance + 1; // Calculate the new distance
                distances.insert(neighbor.clone(), new_distance);
                visited.insert(neighbor.clone());
                queue.push_back((neighbor.clone(), new_distance)); // Adds the neighbor and new distance to the queue
            }
        }
    }

    distances
}

// Stage 3: Degree distribution analysis
fn degree_distribution_analysis(graph: &Graph) {
    let mut degrees = HashMap::new();

    for vertex in graph.vertices.iter() {
        let degree = graph.get_neighbors(vertex).len();
        let count = degrees.entry(degree).or_insert(0);
        *count += 1;
    }

    let mut degree_counts: Vec<(usize, usize)> = degrees.into_iter().collect();
    degree_counts.sort_by_key(|&(degree, _)| degree);

    println!("Vertex degree distribution:");
    for (degree, count) in degree_counts.iter() {
        println!("degree {}: {}", degree, count);
    }
}

#[tokio::main]
async fn main() {
// The first stage: data acquisition and preprocessing
    if let Err(err) = download_and_extract_data().await {
        eprintln!("Error: {}", err);
        process::exit(1);
    }

// Create a graph representation
    let graph = Graph::new();

// Second stage: Average distance calculation
    let avg_distance = calculate_average_distance(&graph);
    println!("平均距离: {}", avg_distance);

// Stage 3: Degree distribution analysis
    degree_distribution_analysis(&graph);
}
