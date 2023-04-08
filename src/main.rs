use kube::{Client};
use tabled::{Table, Style};

#[cfg(test)]
mod utils_test;
mod utils;
mod kubernetes;

#[tokio::main]
async fn main() {
    let client = match Client::try_default().await {
        Err(e) => {
            eprintln!("Error creating kubernetes client {:?}", e);
            return;
        },
        Ok(client) => client,
    };

    let mut resource_req = Vec::new();

    kubernetes::collect_node_info(client.clone(), &mut resource_req).await;

    let data = utils::parse_resource_data(&resource_req);
    let mut table = Table::new(&data);

    table.with(Style::rounded());

    println!("{}", table);
}