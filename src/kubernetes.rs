use std::str::FromStr;

use kube::{Client, Api, api::ListParams};
use k8s_openapi::api::core::v1::{Node, Pod, Namespace};
use tabled::{Tabled};

use super::utils;

pub struct ResouceRequests {
    pub name: String,
    pub cpu_requests: u32,
    pub cpu_total: u32,
    pub mem_requests: f32,
    pub mem_total: f32,
    pub storage_requests: f32,
    pub storage_total: f32,
    pub pods: usize,
    pub pods_total: usize,
}

#[derive(Tabled)]
pub struct ResourceStatus {
    node_name: String,
    cpu: String,
    mem: String,
    storage: String,
    pods: String,
}

impl ResouceRequests {
    pub fn new(name: String, cpu_requests: u32, cpu_total: u32, mem_requests: f32, mem_total: f32, storage_requests: f32, storage_total: f32, pods: usize, pods_total: usize) -> Self {
        Self {
            name,
            cpu_requests,
            cpu_total,
            mem_requests,
            mem_total,
            storage_requests,
            storage_total,
            pods,
            pods_total,
        }
    }
}

impl ResourceStatus {
    pub fn new(node_name: String, cpu: String, mem: String, storage: String, pods: String) -> Self {
        Self {
            node_name,
            cpu,
            mem,
            storage,
            pods,
        }
    }
}

pub enum ResourceType {
    Node,
    Namespace,
}

impl FromStr for ResourceType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "node" => Ok(ResourceType::Node),
            "namespace" => Ok(ResourceType::Namespace),
            _ => Err(()),
        }
    }
}

pub async fn get_pods_resources_req(client: Client, resource_type: &ResourceType, resource_name: &String) -> (u32, f32, f32, usize) {
    let api: Api<Pod> = Api::all(client);

    let field_selector = match resource_type {
        ResourceType::Node => format!("spec.nodeName={}", resource_name),
        ResourceType::Namespace => format!("metadata.namespace={}", resource_name),
    };

    let lp = ListParams::default().fields(&field_selector.as_str());

    let pods = match api.list(&lp).await {
        Ok(pods) => pods,
        Err(e) => {
            eprint!("Error listing pods {:?}", e);
            return (0, 0.0, 0.0, 0);
        }
    };

    let mut cpu_requested: u32 = 0;
    let mut mem_requested: f32 = 0.0;
    let mut storage_requested: f32 = 0.0;

    for pod in pods.items.clone() {
        if let Some(spec) = pod.spec {
            for container in spec.containers {
                if let Some(resources) = container.resources {
                    if let Some(requests) = resources.requests {
                        if let Some(cpu) = requests.get("cpu") {
                            cpu_requested += utils::parse_cpu_requests(cpu.0.to_string())
                        }
                        if let Some(mem) = requests.get("memory") {
                            mem_requested += utils::parse_capacity_requests(mem.0.to_string())
                        }
                        if let Some(storage) = requests.get("ephemeral-storage") {
                            storage_requested += utils::parse_capacity_requests(storage.0.to_string())
                        }
                    }
                }
            }
        }
    }

    return (cpu_requested, mem_requested, storage_requested, pods.items.len());
}

async fn get_node_info(client: Client, node_name: &String) -> (u32, f32, f32, usize) {
    let api: Api<Node> = Api::all(client);

    let node = match api.get(node_name).await {
        Ok(node) => node,
        Err(_) => {
            // eprintln!("Error get node information {}", e);
            return (0, 0.0, 0.0, 0);
        }
    };

    let mut total_cpu: u32 = 0;
    let mut total_mem: f32 = 0.0;
    let mut total_storage: f32 = 0.0;
    let mut total_pods: usize =  0;


    if let Some(node_status) = node.status {
        if let Some(allocatable) = node_status.allocatable {
            if let Some(cpu) = allocatable.get("cpu") {
                total_cpu += utils::parse_cpu_requests(cpu.0.to_string())
            }
            if let Some(mem) = allocatable.get("memory") {
                total_mem += utils::parse_capacity_requests(mem.0.to_string())
            }
            if let Some(storage) = allocatable.get("ephemeral-storage") {
                total_storage += utils::parse_capacity_requests(storage.0.to_string())
            }
            if let Some(pods) = allocatable.get("pods") {
                total_pods += pods.0.parse::<usize>().unwrap()
            }
        }
    }

    return (total_cpu, total_mem, total_storage, total_pods)
}

pub async fn collect_info(client: Client, rrs: &mut Vec<ResouceRequests>, resource_type: ResourceType, selector: Option<String>) {
    let mut lp = ListParams::default();
    let mut resource_names: Vec<String> = Vec::new();

    match &resource_type {
        ResourceType::Node => {
            if let Some(node_lebels) = selector {
                lp = ListParams::default().labels(&node_lebels)
            }

            let api: Api<Node> = Api::all(client.clone());

            let nodes = match api.list(&lp).await {
                Ok(nodes) => nodes,
                Err(e) => {
                    eprintln!("Error listing nodes {:?}", e);
                    return;
                }
            };

            for node in nodes.items {
                resource_names.push(node.metadata.name.unwrap());
            }

        },
        ResourceType::Namespace => {
            if let Some(ns_labels) = selector {
                lp = ListParams::default().labels(&ns_labels)
            }

            let api: Api<Namespace> = Api::all(client.clone());

            let namespaces = match api.list(&lp).await {
                Ok(namespaces) => namespaces,
                Err(e) => {
                    eprintln!("Error listing namespaces {:?}", e);
                    return;
                }
            };

            for namespace in namespaces.items {
                resource_names.push(namespace.metadata.name.unwrap());
            }
        },
    };

    let mut cluster_cpu_req: u32 = 0;
    let mut cluster_cpu_total: u32 = 0;
    let mut cluster_mem_req: f32 = 0.0;
    let mut cluster_mem_total: f32 = 0.0;
    let mut cluster_storage_req: f32 = 0.0;
    let mut cluster_storage_total: f32 = 0.0;
    let mut cluster_pods_req: usize = 0;
    let mut cluster_pods_total: usize = 0;


    for name in resource_names {
        let (cpu_requests, mem_requests, storage_requests, pods) = get_pods_resources_req(client.clone(), &resource_type, &name).await;
        let (cpu_total, mem_total, storage_total, pods_total) = get_node_info(client.clone(), &name).await;

        utils::add_data(name.clone(), cpu_requests, cpu_total, mem_requests, mem_total, storage_requests, storage_total, pods, pods_total, rrs).await;

        cluster_cpu_req += cpu_requests;
        cluster_cpu_total += cpu_total;
        cluster_mem_req += mem_requests;
        cluster_mem_total += mem_total;
        cluster_storage_req += storage_requests;
        cluster_storage_total += storage_total;
        cluster_pods_req += pods;
        cluster_pods_total += pods_total;
    }

    utils::add_data(
        String::from("*"),
        cluster_cpu_req,
        cluster_cpu_total,
        cluster_mem_req,
        cluster_mem_total,
        cluster_storage_req,
        cluster_storage_total,
        cluster_pods_req,
        cluster_pods_total,
        rrs
    ).await;
}