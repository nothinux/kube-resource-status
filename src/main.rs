use std::str::FromStr;

use bpaf::Bpaf;
use kube::{Client};
use tabled::{Table, Style};

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
/// a tool that provide kubernetes cluster resource information, including cpu, memory, storage and number of pods.
struct Options {
    #[bpaf(short('l'), long)]
    /// filter spesific node using it's label
    selector: Option<String>,
    #[bpaf(short('t'), long("type"))]
    /// filter based on resource type (eg: node, namespace), default: node
    resource_type: Option<String>,
}

mod utils;
mod kubernetes;

#[cfg(test)]
mod utils_test;

#[tokio::main]
async fn main() {
    let opts = options().run();
    let mut resource_type: kubernetes::ResourceType = kubernetes::ResourceType::Node;

    if let Some(rt) = opts.resource_type {
        resource_type =  kubernetes::ResourceType::from_str(&rt).unwrap();
    }

    let client = match Client::try_default().await {
        Err(e) => {
            eprintln!("Error creating kubernetes client {:?}", e);
            return;
        },
        Ok(client) => client,
    };

    let mut resource_req = Vec::new();

    kubernetes::collect_info(client.clone(), &mut resource_req, resource_type, opts.selector).await;

    let data = utils::parse_resource_data(&resource_req);
    let mut table = Table::new(&data);

    table.with(Style::rounded());

    println!("{}", table);
}