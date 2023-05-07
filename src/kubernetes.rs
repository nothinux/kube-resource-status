use std::{str::FromStr};

use kube::{Client, Config, Api, api::ListParams, client::ConfigExt, core::ObjectMeta};
use k8s_openapi::{api::core::v1::{Node, Pod, Namespace, Container}, apimachinery::pkg::api::resource::Quantity};
use tabled::{Tabled};

use crate::utils::{parse_cpu_requests, parse_capacity_requests};

use super::utils;

#[derive(serde::Deserialize, Clone, Debug)]
struct Usage {
    cpu: Quantity,
    memory: Quantity,
}

#[derive(serde::Deserialize, Clone, Debug)]
struct NodeMetrics {
    metadata: ObjectMeta,
    usage: Usage,
}

#[derive(serde::Deserialize, Clone, Debug)]
struct ContainerMetrics {
    usage: Usage,
}

#[derive(serde::Deserialize, Clone, Debug)]
struct PodMetrics {
    metadata: ObjectMeta,
    containers: Vec<ContainerMetrics>,
}

impl k8s_openapi::Resource for NodeMetrics {
    const GROUP: &'static str = "metrics.k8s.io";
    const KIND: &'static str = "node";
    const VERSION: &'static str = "v1beta1";
    const API_VERSION: &'static str = "metrics.k8s.io/v1beta1";
    const URL_PATH_SEGMENT: &'static str = "nodes";

    type Scope = k8s_openapi::ClusterResourceScope;
}

impl k8s_openapi::Resource for PodMetrics {
    const GROUP: &'static str = "metrics.k8s.io";
    const KIND: &'static str = "pod";
    const VERSION: &'static str = "v1beta1";
    const API_VERSION: &'static str = "metrics.k8s.io/v1beta1";
    const URL_PATH_SEGMENT: &'static str = "pods";

    type Scope = k8s_openapi::NamespaceResourceScope;
}

impl k8s_openapi::Metadata for NodeMetrics {
    type Ty = ObjectMeta;

    fn metadata(&self) -> &<Self as k8s_openapi::Metadata>::Ty {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut<Self as k8s_openapi::Metadata>::Ty {
        &mut self.metadata
    }
}

impl k8s_openapi::Metadata for PodMetrics {
    type Ty = ObjectMeta;

    fn metadata(&self) -> &<Self as k8s_openapi::Metadata>::Ty {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut<Self as k8s_openapi::Metadata>::Ty {
        &mut self.metadata
    }
}

#[derive(Clone)]
pub struct ResouceRequests {
    pub name: String,
    pub cpu_requests: u32,
    pub cpu_total: u32,
    pub cpu_usage: u32,
    pub mem_requests: f32,
    pub mem_total: f32,
    pub mem_usage: f32,
    pub storage_requests: f32,
    pub storage_total: f32,
    pub pods: usize,
    pub pods_total: usize,
}

#[derive(Tabled)]
pub struct ResourceStatus {
    name: String,
    cpu: String,
    #[tabled(rename = "cpu usage")]
    cpu_usage: String,
    mem: String,
    #[tabled(rename = "mem usage")]
    mem_usage: String,
    storage: String,
    pods: String,
}

impl ResouceRequests {
    pub fn new(
        name: String, cpu_requests: u32, cpu_total: u32, cpu_usage: u32,
        mem_requests: f32, mem_total: f32, mem_usage: f32, storage_requests: f32,
        storage_total: f32, pods: usize, pods_total: usize) -> Self {
        Self {
            name,
            cpu_requests,
            cpu_total,
            cpu_usage,
            mem_requests,
            mem_total,
            mem_usage,
            storage_requests,
            storage_total,
            pods,
            pods_total,
        }
    }
}

impl ResourceStatus {
    pub fn new(name: String, cpu: String, cpu_usage: String, mem: String, mem_usage: String, storage: String, pods: String) -> Self {
        Self {
            name,
            cpu,
            cpu_usage,
            mem,
            mem_usage,
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
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "node" => Ok(ResourceType::Node),
            "namespace" => Ok(ResourceType::Namespace),
            _ => Err(format!("invalid resource type {}", s)),
        }
    }
}

pub async fn connect() -> Client {
    let c = Config::infer().await.unwrap();

    let client = if c.tls_server_name.is_some() {
        let https = c.rustls_https_connector().unwrap();
        let service = tower::ServiceBuilder::new().layer(c.base_uri_layer()).service(hyper::Client::builder().build(https));
        Client::new(service, c.default_namespace)
    } else {
        let https = c.openssl_https_connector().unwrap();
        let service = tower::ServiceBuilder::new().layer(c.base_uri_layer()).service(hyper::Client::builder().build(https));
        Client::new(service, c.default_namespace)
    };

    return client;
}

pub async fn get_node_utilization(client: Client, node_name: &String) -> (u32, f32) {
    let api = Api::<NodeMetrics>::all(client);
    let node_metrics = match api.get(node_name).await {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error getting node utilization information {}", e);
            return (0, 0.0);
        }
    };

    let cpu_usage = parse_cpu_requests(node_metrics.usage.cpu.0.to_string());
    let mem_usage = parse_capacity_requests(node_metrics.usage.memory.0.to_string());

    return (cpu_usage, mem_usage);
}

pub async fn get_pod_utilization(client: Client, namespace: &String) -> (u32, f32) {
    let api = Api::<Pod>::namespaced(client.clone(), &namespace);
    let lp = ListParams::default();

    let pods = match api.list(&lp).await {
        Ok(pods) => pods,
        Err(e) => {
            eprintln!("Error listing pods {:?}", e);
            return (0, 0.0);
        },
    };

    let mut cpu_usage: u32 = 0;
    let mut mem_usage: f32 = 0.0;

    if pods.items.len() != 0 {
        for pod in pods.items {
            let api = Api::<PodMetrics>::namespaced(client.clone(), namespace);
            let pod_metrics = match api.get(pod.metadata.name.unwrap().as_str()).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error getting pod utilization information {}", e);
                    return (0, 0.0);
                }
            };


            for container in pod_metrics.containers {
                cpu_usage += parse_cpu_requests(container.usage.cpu.0.to_string());
                mem_usage += parse_capacity_requests(container.usage.memory.0.to_string());
            }
        }

        return (cpu_usage, mem_usage)
    }

    return (0, 0.0)

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
            eprintln!("Error listing pods {:?}", e);
            return (0, 0.0, 0.0, 0);
        }
    };

    let mut cpu_requested: u32 = 0;
    let mut mem_requested: f32 = 0.0;
    let mut storage_requested: f32 = 0.0;

    for pod in pods.items.clone() {
        if let Some(status) = pod.status {
            if let Some(phase) = status.phase {
                if phase == "Failed" || phase == "Completed" || phase == "Succeeded" {
                    continue;
                }
            }
        }

        if let Some(spec) = pod.spec {
            let mut init_cpu_requested: u32 = 0;
            let mut init_mem_requested: f32 = 0.0;
            let mut init_storage_requested: f32 = 0.0;

            if let Some(init_containers) = spec.init_containers {
                (init_cpu_requested, init_mem_requested, init_storage_requested) = get_containers_resources_req(init_containers).await;
            }

            let (cpu_req, mem_req, storage_req) = get_containers_resources_req(spec.containers).await;

            cpu_requested += cpu_req.max(init_cpu_requested);
            mem_requested += mem_req.max(init_mem_requested);
            storage_requested += storage_req.max(init_storage_requested);
        }
    }

    return (cpu_requested, mem_requested, storage_requested, pods.items.len());
}

async fn get_containers_resources_req(containers: Vec<Container>) -> (u32, f32, f32) {
    let mut cpu_requested: u32 = 0;
    let mut mem_requested: f32 = 0.0;
    let mut storage_requested: f32 = 0.0;

    for container in containers {
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

    return (cpu_requested, mem_requested, storage_requested)
}

async fn get_cluster_node_info(client: Client) -> (u32, f32, f32, usize) {
    let api: Api<Node> = Api::all(client.clone());
    let lp = ListParams::default();

    let nodes = match api.list(&lp).await {
        Ok(nodes) => nodes,
        Err(e) => {
            eprintln!("Error getting nodes information {}", e);
            return (0, 0.0, 0.0, 0);
        }
    };

    let mut cluster_total_cpu: u32 = 0;
    let mut cluster_total_mem: f32 = 0.0;
    let mut cluster_total_storage: f32 = 0.0;
    let mut cluster_total_pods: usize =  0;

    for node in nodes {
        if let Some(node_name) = node.metadata.name {
            let (total_cpu, total_mem, total_storage, total_pods) = get_node_info(client.clone(), &node_name).await;
            cluster_total_cpu += total_cpu;
            cluster_total_mem += total_mem;
            cluster_total_storage += total_storage;
            cluster_total_pods += total_pods;
        }
    }

    return (cluster_total_cpu, cluster_total_mem, cluster_total_storage, cluster_total_pods)
}

async fn get_node_info(client: Client, node_name: &String) -> (u32, f32, f32, usize) {
    let api: Api<Node> = Api::all(client);

    let node = match api.get(node_name).await {
        Ok(node) => node,
        Err(e) => {
            eprintln!("Error get node information {}", e);
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

pub async fn collect_info(client: Client, rrs: &mut Vec<ResouceRequests>, resource_type: ResourceType, utilization: bool, selector: Option<String>) {
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
    let mut cluster_cpu_usage: u32 = 0;
    let mut cluster_mem_req: f32 = 0.0;
    let mut cluster_mem_total: f32 = 0.0;
    let mut cluster_mem_usage: f32 = 0.0;
    let mut cluster_storage_req: f32 = 0.0;
    let mut cluster_storage_total: f32 = 0.0;
    let mut cluster_pods_req: usize = 0;
    let mut cluster_pods_total: usize = 0;

    for name in resource_names {
        let (cpu_requests, mem_requests, storage_requests, pods) = get_pods_resources_req(client.clone(), &resource_type, &name).await;

        let mut cpu_usage: u32 = 0;
        let mut mem_usage: f32 = 0.0;
        let mut cpu_total: u32 = 0;
        let mut mem_total: f32 = 0.0;
        let mut storage_total: f32 = 0.0;
        let mut pods_total: usize = 0;

        if utilization {
            match &resource_type {
                ResourceType::Node => {
                    (cpu_usage, mem_usage) = get_node_utilization(client.clone(), &name).await;
                },
                ResourceType::Namespace => {
                    (cpu_usage, mem_usage) = get_pod_utilization(client.clone(), &name).await;
                }
            }
        }

        cluster_cpu_req += cpu_requests;
        cluster_cpu_usage += cpu_usage;
        cluster_mem_req += mem_requests;
        cluster_mem_usage += mem_usage;
        cluster_storage_req += storage_requests;
        cluster_pods_req += pods;

        match &resource_type {
            ResourceType::Namespace => {
                if (cluster_cpu_total, cluster_mem_total, cluster_storage_total, cluster_pods_total) == (0, 0.0, 0.0, 0) {
                    (cluster_cpu_total, cluster_mem_total, cluster_storage_total, cluster_pods_total) = get_cluster_node_info(client.clone()).await;
                }
                (cpu_total, mem_total, storage_total, pods_total) = (cluster_cpu_total, cluster_mem_total, cluster_storage_total, cluster_pods_total)
            },
            ResourceType::Node => {
                (cpu_total, mem_total, storage_total, pods_total) = get_node_info(client.clone(), &name).await;
                cluster_cpu_total += cpu_total;
                cluster_mem_total += mem_total;
                cluster_storage_total += storage_total;
                cluster_pods_total += pods_total;
            }
        }

        utils::add_data(name.clone(), cpu_requests, cpu_total, cpu_usage, mem_requests, mem_total, mem_usage, storage_requests, storage_total, pods, pods_total, rrs).await;
    }

    utils::add_data(
        String::from("*"),
        cluster_cpu_req,
        cluster_cpu_total,
        cluster_cpu_usage,
        cluster_mem_req,
        cluster_mem_total,
        cluster_mem_usage,
        cluster_storage_req,
        cluster_storage_total,
        cluster_pods_req,
        cluster_pods_total,
        rrs
    ).await;
}