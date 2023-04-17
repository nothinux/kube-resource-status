use std::cmp::Reverse;
use std::{str::FromStr};

use super::kubernetes;

pub enum Filter {
    Cpu,
    Mem,
    Storage,
    Pods,
    None,
}

impl FromStr for Filter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "" => Ok(Filter::None),
            "cpu" => Ok(Filter::Cpu),
            "mem" => Ok(Filter::Mem),
            "storage" => Ok(Filter::Storage),
            "pods" => Ok(Filter::Pods),
            _ => Err(format!("invalid filter {}", s))
        }
    }
}

pub fn parse_resource_data(rrs: Vec<kubernetes::ResouceRequests>, sort_by: Filter) -> Vec<kubernetes::ResourceStatus> {
    let mut rss = Vec::new();
    let mut data: Vec<kubernetes::ResouceRequests> = rrs.clone();

    match sort_by {
        Filter::Cpu => data.sort_by_key(|r| Reverse(r.cpu_requests)),
        Filter::Mem => {
            data.sort_by(|a, b| {
                b.mem_requests.partial_cmp(&a.mem_requests).unwrap()
            })
        },
        Filter::Storage => {
            data.sort_by(|a, b| {
                b.storage_requests.partial_cmp(&a.storage_requests).unwrap()
            })
        },
        Filter::Pods => data.sort_by_key(|r| Reverse(r.pods)),
        _ => (),
    }

    for rr in data {
        let cpu_total = (rr.cpu_requests as f32 / rr.cpu_total as f32) * 100.0;
        let mem_total = (rr.mem_requests / rr.mem_total) * 100.0;
        let storage_total = (rr.storage_requests / rr.storage_total) * 100.0;

        let rs = kubernetes::ResourceStatus::new(
            format!("{}", rr.name),
            format!("{}m ({:.2}%)", rr.cpu_requests, cpu_total),
            format!("{}Mi ({:.2}%)", rr.mem_requests, mem_total),
            format!("{}Mi ({:.2}%)", rr.storage_requests, storage_total),
            format!("{} / {}", rr.pods, rr.pods_total),
        );
        rss.push(rs);


    }

    return rss;
}

pub async fn add_data(
    node_name: String, cpu_requests: u32, cpu_total: u32, mem_requests: f32,
    mem_total: f32, storage_requests: f32, storage_total: f32, pods: usize,
    pods_total: usize, rrs: &mut Vec<kubernetes::ResouceRequests>
) {
    rrs.push(kubernetes::ResouceRequests::new(
        node_name,
        cpu_requests,
        cpu_total,
        mem_requests,
        mem_total,
        storage_requests,
        storage_total,
        pods,
        pods_total,
    ));
}

pub fn parse_cpu_requests(cpu: String) -> u32 {
    if cpu.contains("m") {
        let m = cpu.replace("m", "");
        return m.parse::<u32>().unwrap();
    } else if cpu.contains(".") {
        let m = cpu.replace(".", "");
        return m.parse::<u32>().unwrap() * 100;
    } else {
        return cpu.parse::<u32>().unwrap() * 1000;
    }
}

pub fn parse_capacity_requests(mem: String) -> f32 {
    if mem.contains("Ki") {
        let m = mem.replace("Ki", "");
        return m.parse::<f32>().unwrap() / 1024.0;
    } else if mem.contains("Mi") {
        let m = mem.replace("Mi", "");
        return m.parse::<f32>().unwrap();
    } else if mem.contains("Gi") {
        let m = mem.replace("Gi", "");
        return m.parse::<f32>().unwrap() * 1024.0;
    } else if mem.contains("Ti") {
        let m = mem.replace("Ti", "");
        return m.parse::<f32>().unwrap() * 1024.0 * 1024.0;
    } else {
        return mem.parse::<f32>().unwrap() / 1024.0 / 1024.0;
    }
}