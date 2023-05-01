use std::str::FromStr;

use bpaf::Bpaf;
use tabled::{Table, Style, Disable, locator::ByColumnName};

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
/// a tool that provide kubernetes cluster resource information, including cpu, memory, storage and number of pods.
struct Options {
    #[bpaf(short('u'), long)]
    /// show the real utilization
    utilization: bool,
    #[bpaf(short('l'), long)]
    /// filter spesific node using it's label
    selector: Option<String>,
    #[bpaf(short('t'), long("type"))]
    /// filter based on resource type (eg: node, namespace), default: node
    resource_type: Option<String>,
    #[bpaf(short('s'), long)]
    /// filter by cpu, mem, storage or pods
    sort_by: Option<String>
}

mod utils;
mod kubernetes;

#[cfg(test)]
mod utils_test;

#[tokio::main]
async fn main() {
    let opts = options().run();
    let mut sort_by = utils::Filter::None;
    let mut resource_type = kubernetes::ResourceType::Node;

    if let Some(rt) = opts.resource_type {
        resource_type = match kubernetes::ResourceType::from_str(&rt) {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };
    }

    if let Some(s) = opts.sort_by {
        sort_by = match utils::Filter::from_str(&s) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        }
    }

    let mut resource_req = Vec::new();
    let client = kubernetes::connect().await;

    kubernetes::collect_info(client.clone(), &mut resource_req, resource_type, opts.utilization, opts.selector).await;

    let data = utils::parse_resource_data(resource_req, sort_by);
    let mut table = Table::new(&data);
        
    table.with(Style::rounded());
    if !opts.utilization {
        table.with(Disable::column(ByColumnName::new("cpu usage")));
        table.with(Disable::column(ByColumnName::new("mem usage")));
    }

    println!("{}", table);
}