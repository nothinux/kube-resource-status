use super::utils::*;

#[test]
fn test_parse_cpu_requests() {
    assert_eq!(parse_cpu_requests(String::from("100m")), 100);
    assert_eq!(parse_cpu_requests(String::from("500m")), 500);
    assert_eq!(parse_cpu_requests(String::from("1")), 1000);
    assert_eq!(parse_cpu_requests(String::from("2")), 2000);
    assert_eq!(parse_cpu_requests(String::from("2.5")), 2500);
    assert_eq!(parse_cpu_requests(String::from("12.5")), 12500);
}

#[test]
fn test_parse_capacity_requests() {
    assert_eq!(parse_capacity_requests(String::from("1000Ki")), 0.9765625);
    assert_eq!(parse_capacity_requests(String::from("1Mi")), 1.0);
    assert_eq!(parse_capacity_requests(String::from("1000Mi")), 1000.0);
    assert_eq!(parse_capacity_requests(String::from("1Gi")), 1024.0);
    assert_eq!(parse_capacity_requests(String::from("10Gi")), 10240.0);
    assert_eq!(parse_capacity_requests(String::from("3Ti")), 3145728.0);
    assert_eq!(parse_capacity_requests(String::from("1.5Gi")), 1536.0);
    assert_eq!(parse_capacity_requests(String::from("1.3Ti")), 1363148.8);
    assert_eq!(parse_capacity_requests(String::from("10.5Mi")), 10.5);
    assert_eq!(parse_capacity_requests(String::from("53M")), 50.544724);
    assert_eq!(parse_capacity_requests(String::from("20k")), 0.01907348);
    assert_eq!(parse_capacity_requests(String::from("10G")), 9536.74);
    assert_eq!(parse_capacity_requests(String::from("1M")), 0.953674);
    assert_eq!(parse_capacity_requests(String::from("1T")), 953674.0);

}