use super::kubernetes;

pub fn parse_resource_data(rrs: &Vec<kubernetes::ResouceRequests>) -> Vec<kubernetes::ResourceStatus> {
    let mut rss = Vec::new();

    for rr in rrs {
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
        return m.parse::<f32>().unwrap() / 1000.0;
    } else if mem.contains("Mi") {
        let m = mem.replace("Mi", "");
        return m.parse::<f32>().unwrap();
    } else if mem.contains("Gi") {
        let m = mem.replace("Gi", "");
        return m.parse::<f32>().unwrap() * 1000.0;
    } else if mem.contains("Ti") {
        let m = mem.replace("Ti", "");
        return m.parse::<f32>().unwrap() * 1000.0 * 1000.0;
    } else {
        return 0.0;
    }
}