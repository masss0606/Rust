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

// 定义常量
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
        self.edges.get(vertex).cloned().unwrap_or(HashSet::new()) // 返回 HashSet<String>
    }
}

// 第一阶段: 数据获取与预处理
async fn download_and_extract_data() -> Result<(), Box<dyn Error>> {
    // 下载数据集
    let data = download_data().await?;
    // 解压缩数据
    extract_data(data)?;

    Ok(())
}

async fn download_data() -> Result<Vec<u8>, Box<dyn Error>> {
    // 下载数据集
    let response = reqwest::get(DATA_URL).await?;
    // 检查请求是否成功
    if !response.status().is_success() {
        return Err("无法下载数据集".into());
    }

    // 读取响应体
    let mut buf = Vec::new();
    let bytes = response.bytes().await?;
    buf.extend_from_slice(&bytes); // 使用 extend_from_slice 方法将字节追加到向量中

    Ok(buf)
}

fn extract_data(data: Vec<u8>) -> Result<(), Box<dyn Error>> {
    // 解压缩数据
    let reader = GzDecoder::new(data.as_slice());
    let mut tar = Archive::new(reader); // 将 tar 声明为可变的

    // 创建一个向量来保存抽样的电子邮件
    let mut sampled_emails = Vec::new();

    // 在存档中迭代每个条目
    for entry in tar.entries()? {
        let mut entry = entry?;

        // 检查条目是否为文件
        if entry.header().entry_type().is_file() {
            // 读取电子邮件内容
            let mut content = String::new();
            entry.read_to_string(&mut content)?;

            // 将电子邮件内容推入向量中
            sampled_emails.push(content);

            // 当抽样的电子邮件数量达到样本大小时停止
            if sampled_emails.len() >= SAMPLE_SIZE {
                break;
            }
        }
    }

    // 随机打乱抽样的电子邮件
    let mut rng = rand::thread_rng();
    sampled_emails.shuffle(&mut rng);

    // 打印前 10 封抽样的电子邮件以供验证
    for email in sampled_emails.iter().take(10) {
        println!("{}", email);
    }

    Ok(())
}

// 第二阶段: 平均距离计算
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
    let mut queue: VecDeque<(String, usize)> = VecDeque::new(); // 修改队列类型为包含距离的元组

    distances.insert(start_vertex.to_string(), 0);
    visited.insert(start_vertex.to_string());
    queue.push_back((start_vertex.to_string(), 0)); // 初始化起始顶点距离为 0

    while let Some((current_vertex, current_distance)) = queue.pop_front() { // 修改迭代变量为元组
        for neighbor in graph.get_neighbors(&current_vertex) {
            if !visited.contains(neighbor.as_str()) {
                let new_distance = current_distance + 1; // 计算新的距离
                distances.insert(neighbor.clone(), new_distance);
                visited.insert(neighbor.clone());
                queue.push_back((neighbor.clone(), new_distance)); // 将邻居和新距离加入队列
            }
        }
    }

    distances
}

// 第三阶段: 度分布分析
fn degree_distribution_analysis(graph: &Graph) {
    let mut degrees = HashMap::new();

    for vertex in graph.vertices.iter() {
        let degree = graph.get_neighbors(vertex).len();
        let count = degrees.entry(degree).or_insert(0);
        *count += 1;
    }

    let mut degree_counts: Vec<(usize, usize)> = degrees.into_iter().collect();
    degree_counts.sort_by_key(|&(degree, _)| degree);

    println!("顶点度分布:");
    for (degree, count) in degree_counts.iter() {
        println!("度 {}: {}", degree, count);
    }
}

// 第四阶段: 社区检测和中心度量
fn community_detection_and_centrality_measures(graph: &Graph) {
    // 使用 Louvain 算法检测社区
    let communities = louvain(graph);

    println!("社区检测结果:");
    for (community_id, vertices) in communities.iter() {
        println!("社区 {}: {:?}", community_id, vertices);
    }

    // 计算中心度量
    let centrality_measures = calculate_centrality(graph);

    println!("中心度量:");
    for (vertex, centrality) in centrality_measures.iter() {
        println!("{}: {}", vertex, centrality);
    }
}

// Louvain 算法的实现
fn louvain(graph: &Graph) -> HashMap<usize, HashSet<String>> {
    // 实现 Louvain 算法
    // 返回社区
    HashMap::new()
}

// 计算中心度量的实现
fn calculate_centrality(graph: &Graph) -> HashMap<String, f64> {
    // 计算中心度量
    // 返回中心度量的 HashMap
    HashMap::new()
}

#[tokio::main]
async fn main() {
    // 第一阶段: 数据获取与预处理
    if let Err(err) = download_and_extract_data().await {
        eprintln!("Error: {}", err);
        process::exit(1);
    }

    // 创建一个图表示（可以根据实际需求修改）
    let graph = Graph::new();

    // 第二阶段: 平均距离计算
    let avg_distance = calculate_average_distance(&graph);
    println!("平均距离: {}", avg_distance);

    // 第三阶段: 度分布分析
    degree_distribution_analysis(&graph);

    // 第四阶段: 社区检测和中心度量
    community_detection_and_centrality_measures(&graph);
}
