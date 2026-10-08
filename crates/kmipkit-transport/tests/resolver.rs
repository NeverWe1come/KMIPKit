//! Resolver behavior tests against the real private resolver and loopback DNS.
//!
//! The Hickory 0.26.3 system-config loaders use the platform resolver API,
//! hosts file, and global search configuration. They flatten DNS servers and
//! do not preserve per-interface split-DNS routing; this is an explicit
//! platform limitation, not a fallback to an alternate endpoint.

use std::collections::BTreeMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use hickory_resolver::config::{ConnectionConfig, NameServerConfig, ResolverConfig};
use hickory_resolver::hosts::Hosts;
use hickory_resolver::proto::rr::Name;
use kmipkit_test_support::{DnsQueryType, LocalDnsFixture};
use tokio::task::JoinSet;

// Include the production module rather than a test resolver so runtime tests
// observe Hickory traffic and the production bounds.
#[allow(dead_code)]
#[path = "../src/resolver.rs"]
mod resolver;

use resolver::{ResolveFailure, Resolver, ResolverLimits};

const RESOLUTION_TIMEOUT: Duration = Duration::from_secs(12);

fn resolver_config(servers: &[SocketAddr], search: &[&str]) -> ResolverConfig {
    resolver_config_with_transport(servers, search, false)
}

fn resolver_config_with_transport(
    servers: &[SocketAddr],
    search: &[&str],
    tcp: bool,
) -> ResolverConfig {
    let mut nameservers = Vec::with_capacity(servers.len());
    for address in servers {
        let mut connection = if tcp {
            ConnectionConfig::tcp()
        } else {
            ConnectionConfig::udp()
        };
        connection.port = address.port();
        nameservers.push(NameServerConfig::new(address.ip(), true, vec![connection]));
    }

    let search_domains = search
        .iter()
        .map(|domain| Name::from_str(domain).expect("test search domain parses"))
        .collect();
    ResolverConfig::from_parts(None, search_domains, nameservers)
}

fn local_resolver(fixture: &LocalDnsFixture, search: &[&str]) -> Resolver {
    Resolver::from_config(resolver_config(&[fixture.local_addr()], search), None)
        .expect("the loopback DNS configuration builds")
}

fn fixture(records: &[(&str, Vec<IpAddr>)]) -> LocalDnsFixture {
    let records = records
        .iter()
        .map(|(name, addresses)| ((*name).to_owned(), addresses.clone()))
        .collect::<BTreeMap<_, _>>();
    LocalDnsFixture::bind(records).expect("the fixture accepts loopback records")
}

fn record_a(last_octet: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, last_octet))
}

async fn wait_for_questions(
    fixture: &LocalDnsFixture,
    name: &str,
    expected_a: usize,
    expected_aaaa: usize,
) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if fixture.query_count(name, DnsQueryType::A) >= expected_a
                && fixture.query_count(name, DnsQueryType::Aaaa) >= expected_aaaa
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("the expected DNS questions reach the loopback fixture");
}

fn assert_resolver_limits(limits: ResolverLimits) {
    assert_eq!(limits.retries_after_initial_attempt, 1);
    assert_eq!(limits.concurrent_nameserver_requests_per_query, 2);
    assert_eq!(limits.max_active_requests_per_upstream_connection, 32);
    assert_eq!(limits.response_cache_entries_per_client, 128);
    assert_eq!(limits.max_address_candidates, 16);
}

#[tokio::test]
async fn system_resolver_initializes_from_the_platform_configuration() {
    // This runs on each supported CI OS: Hickory selects Unix resolv.conf,
    // Windows adapter/registry configuration, or Apple System Configuration.
    let resolver = Resolver::from_system_config()
        .expect("the current platform exposes a usable system DNS configuration");

    assert_resolver_limits(resolver.limits());
}

#[test]
fn resolver_records_the_system_split_dns_limitation() {
    assert!(
        resolver::SYSTEM_DNS_LIMITATIONS
            .contains("does not preserve per-interface split-DNS routing")
    );
}

#[tokio::test]
async fn injected_hosts_and_search_configuration_are_used_before_upstream_queries() {
    let dns_name = "service.search.kmipkit.test";
    let fixture = fixture(&[(dns_name, vec![record_a(21)])]);
    let config = resolver_config(&[fixture.local_addr()], &["search.kmipkit.test"]);
    let mut hosts = Hosts::default();
    hosts
        .read_hosts_conf(Cursor::new(
            b"127.0.0.22 local-only.kmipkit.test local-alias.kmipkit.test\n\
              ::1 local-only.kmipkit.test local-alias.kmipkit.test\n",
        ))
        .expect("the test hosts mapping parses");
    let resolver = Resolver::from_config(config, Some(hosts))
        .expect("the injected resolver configuration builds");

    let hosts_result = resolver
        .lookup_candidates(
            "local-only.kmipkit.test",
            Instant::now() + RESOLUTION_TIMEOUT,
        )
        .await
        .expect("hosts-file names resolve locally");
    assert!(hosts_result.contains(&record_a(22)));
    assert!(hosts_result.contains(&IpAddr::V6(Ipv6Addr::LOCALHOST)));
    assert_eq!(
        fixture.query_count("local-only.kmipkit.test", DnsQueryType::A),
        0,
        "hosts results do not issue an upstream query"
    );
    assert_eq!(
        fixture.query_count("local-only.kmipkit.test", DnsQueryType::Aaaa),
        0,
        "hosts results do not issue an upstream query for the other address family"
    );

    let searched_result = resolver
        .lookup_candidates("service", Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("the configured search suffix resolves the short name");
    assert_eq!(searched_result, vec![record_a(21)]);
    assert!(
        fixture.query_count(dns_name, DnsQueryType::A) > 0,
        "the configured search suffix reaches the local DNS fixture"
    );
}

#[tokio::test]
async fn one_retry_means_no_more_than_two_attempts_for_each_question() {
    let name = "retry.kmipkit.test";
    let fixture = fixture(&[(name, vec![record_a(23)])]);
    fixture.drop_next_questions(name, DnsQueryType::A, 1);
    let resolver = local_resolver(&fixture, &[]);

    let resolved = resolver
        .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("the second A attempt receives the local answer");

    assert_eq!(resolved, vec![record_a(23)]);
    assert_eq!(fixture.query_count(name, DnsQueryType::A), 2);
    assert!(fixture.query_count(name, DnsQueryType::Aaaa) <= 2);
}

#[tokio::test]
async fn one_lookup_never_has_more_than_two_concurrent_nameserver_requests() {
    let name = "parallel.kmipkit.test";
    let records = [(name, vec![record_a(24), IpAddr::V6(Ipv6Addr::LOCALHOST)])];
    let first = fixture(&records);
    let second = fixture(&records);
    let third = fixture(&records);
    first.hold_responses();
    second.hold_responses();
    third.hold_responses();
    let resolver = Arc::new(
        Resolver::from_config(
            resolver_config(
                &[first.local_addr(), second.local_addr(), third.local_addr()],
                &[],
            ),
            None,
        )
        .expect("three local nameservers build"),
    );
    assert_resolver_limits(resolver.limits());
    let lookup_resolver = Arc::clone(&resolver);
    let lookup_name = name.to_owned();
    let lookup = tokio::spawn(async move {
        lookup_resolver
            .lookup_candidates(&lookup_name, Instant::now() + RESOLUTION_TIMEOUT)
            .await
    });

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let a_queries = first.query_count(name, DnsQueryType::A)
                + second.query_count(name, DnsQueryType::A)
                + third.query_count(name, DnsQueryType::A);
            let aaaa_queries = first.query_count(name, DnsQueryType::Aaaa)
                + second.query_count(name, DnsQueryType::Aaaa)
                + third.query_count(name, DnsQueryType::Aaaa);
            if a_queries >= 1 && aaaa_queries >= 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("each address question reaches a loopback nameserver");
    for query_type in [DnsQueryType::A, DnsQueryType::Aaaa] {
        let per_nameserver = [
            first.query_count(name, query_type),
            second.query_count(name, query_type),
            third.query_count(name, query_type),
        ];
        assert!(per_nameserver.iter().all(|count| *count <= 1));
        let total = per_nameserver.iter().sum::<usize>();
        assert!((1..=2).contains(&total));
        assert!(
            per_nameserver.iter().any(|count| *count == 0),
            "at least one of three nameservers remains unqueried while responses are held"
        );
    }
    first.release_responses();
    second.release_responses();
    third.release_responses();
    lookup
        .await
        .expect("the resolver task completes")
        .expect("the released DNS responses resolve");
}

#[tokio::test]
async fn active_request_limit_applies_per_upstream_connection_not_per_client() {
    let first_name = "upstream-a.kmipkit.test";
    let second_name = "upstream-b.kmipkit.test";
    let first = fixture(&[(first_name, vec![record_a(25)])]);
    let second = fixture(&[(second_name, vec![record_a(26)])]);
    first.hold_responses();
    second.hold_responses();
    let resolver = Arc::new(
        Resolver::from_config(
            resolver_config_with_transport(&[first.local_addr(), second.local_addr()], &[], true),
            None,
        )
        .expect("two local upstreams build"),
    );
    let mut lookups = JoinSet::new();
    for index in 0..48 {
        let current = Arc::clone(&resolver);
        let hostname = format!("host-{index}.kmipkit.test");
        lookups.spawn(async move {
            current
                .lookup_candidates(&hostname, Instant::now() + RESOLUTION_TIMEOUT)
                .await
        });
    }

    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let first_active = first.active_tcp_requests();
            let second_active = second.active_tcp_requests();
            if first_active + second_active > 32 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("more than 32 active requests can use separate upstream connections");
    assert!(first.peak_active_tcp_requests_per_connection() <= 32);
    assert!(second.peak_active_tcp_requests_per_connection() <= 32);
    assert!(
        first.active_tcp_requests() + second.active_tcp_requests() > 32,
        "the per-upstream cap is not an aggregate client cap"
    );

    first.release_responses();
    second.release_responses();
    while let Some(result) = lookups.join_next().await {
        result.expect("lookup tasks complete");
    }
}

#[tokio::test]
async fn response_cache_is_per_resolver_and_evicts_past_128_answers() {
    let names = (0..130)
        .map(|index| (format!("cache-{index}.kmipkit.test"), vec![record_a(27)]))
        .collect::<BTreeMap<_, _>>();
    let fixture = LocalDnsFixture::bind(names).expect("cache test records are valid loopback data");
    let resolver = local_resolver(&fixture, &[]);
    let first_name = "cache-0.kmipkit.test";

    resolver
        .lookup_candidates(first_name, Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("first answer resolves");
    resolver
        .lookup_candidates(first_name, Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("the cached answer resolves");
    assert_eq!(fixture.query_count(first_name, DnsQueryType::A), 1);

    for index in 1..130 {
        let name = format!("cache-{index}.kmipkit.test");
        resolver
            .lookup_candidates(&name, Instant::now() + RESOLUTION_TIMEOUT)
            .await
            .expect("each unique answer resolves");
    }
    resolver
        .lookup_candidates(first_name, Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("the evicted answer resolves again");
    assert!(fixture.query_count(first_name, DnsQueryType::A) >= 2);

    let isolated_client = local_resolver(&fixture, &[]);
    isolated_client
        .lookup_candidates(first_name, Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("a second resolver client has an independent cache");
    assert!(fixture.query_count(first_name, DnsQueryType::A) >= 3);
}

#[tokio::test]
async fn candidates_are_ordered_and_capped_across_address_families() {
    let name = "ordered.kmipkit.test";
    let addresses = (1..=16).rev().map(record_a).collect::<Vec<_>>();
    let mut answers = addresses.clone();
    answers.push(IpAddr::V6(Ipv6Addr::LOCALHOST));
    let fixture = fixture(&[(name, answers)]);
    let resolver = local_resolver(&fixture, &[]);

    let candidates = resolver
        .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("the loopback record set resolves");

    assert_eq!(candidates.len(), 16);
    assert_eq!(candidates[0], IpAddr::V6(Ipv6Addr::LOCALHOST));
    assert_eq!(candidates[1..], addresses[..15]);
}

#[tokio::test]
async fn timeout_and_cancellation_stop_the_caller_visible_lookup() {
    let name = "blocked.kmipkit.test";
    let fixture = fixture(&[(name, vec![record_a(28)])]);
    fixture.hold_responses();
    let resolver = Arc::new(local_resolver(&fixture, &[]));
    let timed_out = resolver
        .lookup_candidates(name, Instant::now() + Duration::from_millis(150))
        .await
        .expect_err("the blocked lookup reaches its caller deadline");
    assert_eq!(timed_out.failure(), ResolveFailure::Timeout);
    assert!(fixture.query_count(name, DnsQueryType::A) > 0);

    let cancellation_name = "cancelled.kmipkit.test";
    let cancellation_resolver = Arc::clone(&resolver);
    let lookup = tokio::spawn(async move {
        cancellation_resolver
            .lookup_candidates(cancellation_name, Instant::now() + RESOLUTION_TIMEOUT)
            .await
    });
    wait_for_questions(&fixture, "cancelled.kmipkit.test", 1, 1).await;
    lookup.abort();
    assert!(
        lookup
            .await
            .expect_err("the lookup task is cancelled")
            .is_cancelled()
    );
    fixture.release_responses();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        fixture.query_count("cancelled.kmipkit.test", DnsQueryType::A),
        1,
        "cancelled lookup work does not retry or dispatch later"
    );
}

#[tokio::test]
async fn nxdomain_is_reported_without_exposing_the_hostname() {
    let name = "sensitive-name.kmipkit.test";
    let fixture = fixture(&[]);
    fixture.set_nxdomain(name);
    let resolver = local_resolver(&fixture, &[]);

    let error = resolver
        .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect_err("NXDOMAIN is an error for endpoint resolution");

    assert_eq!(error.failure(), ResolveFailure::NxDomain);
    assert!(!error.to_string().contains(name));
}

#[tokio::test]
async fn lookup_returns_no_candidates_before_the_loopback_dns_response_arrives() {
    let name = "no-request-before-dns.kmipkit.test";
    let fixture = fixture(&[(name, vec![record_a(29)])]);
    fixture.hold_responses();
    let resolver = Arc::new(local_resolver(&fixture, &[]));
    let resolving = Arc::clone(&resolver);
    let (resolved_sender, mut resolved_receiver) = tokio::sync::oneshot::channel();
    let lookup = tokio::spawn(async move {
        let result = resolving
            .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
            .await;
        let _send_result = resolved_sender.send(result);
    });

    wait_for_questions(&fixture, name, 1, 1).await;
    assert!(
        matches!(
            resolved_receiver.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ),
        "the same lookup future cannot return candidates while DNS responses are held"
    );
    // Request-byte ordering is asserted by the raw-TLS and HTTPS adapter tests;
    // this resolver test proves the caller has no candidate before DNS completes.
    fixture.release_responses();
    lookup.await.expect("the resolver task completes");
    let candidates = resolved_receiver
        .await
        .expect("the lookup result is delivered")
        .expect("the released DNS answer resolves");
    assert_eq!(candidates, vec![record_a(29)]);
}
